"""Batch computation for full-year panchang."""

from __future__ import annotations

from datetime import date

from panchang._core import py_compute_batch_range, py_compute_batch_year, py_init
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
    py_init(None)
    utc_offset = _tz_offset_for_date(location.tz, year, 6, 15)

    raw_list = py_compute_batch_year(
        year,
        location.lat,
        location.lng,
        location.altitude,
        utc_offset,
    )

    return [_raw_to_batch_day(raw, location.tz) for raw in raw_list]


def compute_range(
    start: date,
    end: date,
    location: Location,
) -> list[BatchDayData]:
    """Compute panchang for a date range (inclusive).

    Args:
        start: Start date.
        end: End date.
        location: Geographic location.

    Returns:
        List of BatchDayData, one per day.
    """
    py_init(None)
    utc_offset = _tz_offset_for_date(location.tz, start.year, start.month, start.day)

    raw_list = py_compute_batch_range(
        start.year,
        start.month,
        start.day,
        end.year,
        end.month,
        end.day,
        location.lat,
        location.lng,
        location.altitude,
        utc_offset,
    )

    return [_raw_to_batch_day(raw, location.tz) for raw in raw_list]
