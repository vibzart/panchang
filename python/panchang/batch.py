"""Batch computation for full-year panchang."""

from __future__ import annotations

from datetime import date, timedelta

from panchang._core import py_compute_batch_range, py_init
from panchang.core.ephemeris import jd_to_datetime
from panchang.core.sun import _tz_offset_for_date
from panchang.types import (
    BatchDayData,
    KaranaInfo,
    Location,
    NakshatraInfo,
    Paksha,
    SunData,
    TithiInfo,
    VaraInfo,
    YogaInfo,
)


def _raw_to_batch_day(raw: dict, tz_name: str) -> BatchDayData:
    """Convert a raw batch dict to a BatchDayData model."""
    from zoneinfo import ZoneInfo

    tz = ZoneInfo(tz_name)

    sun_raw = raw["sun"]
    sunrise_dt = jd_to_datetime(sun_raw["sunrise_jd"]).astimezone(tz)
    sunset_dt = jd_to_datetime(sun_raw["sunset_jd"]).astimezone(tz)

    tithi_raw = raw["tithi"]
    tithi_num = tithi_raw["number"]

    return BatchDayData(
        date=date(raw["year"], raw["month"], raw["day"]),
        vara=VaraInfo(
            number=raw["vara"]["number"],
            name=raw["vara"]["name"],
            english=raw["vara"]["english"],
        ),
        tithi=TithiInfo(
            number=tithi_num,
            name=tithi_raw["name"],
            paksha=Paksha.SHUKLA if tithi_num <= 15 else Paksha.KRISHNA,
        ),
        nakshatra=NakshatraInfo(
            number=raw["nakshatra"]["number"],
            name=raw["nakshatra"]["name"],
            pada=raw["nakshatra"]["pada"],
            lord=raw["nakshatra"]["lord"],
        ),
        yoga=YogaInfo(
            number=raw["yoga"]["number"],
            name=raw["yoga"]["name"],
        ),
        karana=KaranaInfo(
            number=raw["karana"]["number"],
            name=raw["karana"]["name"],
        ),
        sun=SunData(
            sunrise=sunrise_dt,
            sunset=sunset_dt,
            day_duration_hours=sun_raw["day_duration_hours"],
            sunrise_jd=sun_raw["sunrise_jd"],
            sunset_jd=sun_raw["sunset_jd"],
        ),
    )


def compute_year(year: int, location: Location) -> list[BatchDayData]:
    """Compute panchang for every day of a Gregorian year.

    Args:
        year: Gregorian year.
        location: Geographic location.

    Returns:
        List of 365 (or 366) BatchDayData, one per day.
    """
    return compute_range(date(year, 1, 1), date(year, 12, 31), location)


def _offset_segments(tz_name: str, start: date, end: date) -> list[tuple[date, date, int]]:
    """Split [start, end] into runs of constant UTC offset.

    Fixed-offset zones (e.g. Asia/Kolkata) yield a single segment; DST
    zones yield one segment per transition, so each day is computed with
    its own civil offset instead of the range-start offset.
    """
    segments: list[tuple[date, date, int]] = []
    seg_start = start
    seg_offset = _tz_offset_for_date(tz_name, start.year, start.month, start.day)
    d = start
    while d < end:
        nxt = d + timedelta(days=1)
        offset = _tz_offset_for_date(tz_name, nxt.year, nxt.month, nxt.day)
        if offset != seg_offset:
            segments.append((seg_start, d, seg_offset))
            seg_start = nxt
            seg_offset = offset
        d = nxt
    segments.append((seg_start, end, seg_offset))
    return segments


def compute_range(
    start: date,
    end: date,
    location: Location,
) -> list[BatchDayData]:
    """Compute panchang for a date range (inclusive).

    The range is segmented at DST transitions so every day uses its own
    UTC offset — a range crossing a transition previously computed the
    whole span with the start date's offset.

    Args:
        start: Start date.
        end: End date.
        location: Geographic location.

    Returns:
        List of BatchDayData, one per day.
    """
    py_init(None)

    results: list[BatchDayData] = []
    for seg_start, seg_end, utc_offset in _offset_segments(location.tz, start, end):
        raw_list = py_compute_batch_range(
            seg_start.year,
            seg_start.month,
            seg_start.day,
            seg_end.year,
            seg_end.month,
            seg_end.day,
            location.lat,
            location.lng,
            location.altitude,
            utc_offset,
        )
        results.extend(_raw_to_batch_day(raw, location.tz) for raw in raw_list)
    return results
