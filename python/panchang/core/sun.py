"""Sunrise, sunset, and solar calculations for Lilavati.

Uses Rust _core extension which wraps Swiss Ephemeris swe_rise_trans()
for precise geometric sunrise/sunset with Hindu rising method.
"""

from __future__ import annotations

from datetime import datetime
from zoneinfo import ZoneInfo

from panchang._core import py_compute_sun_data, py_init
from panchang.core.ephemeris import EphemerisEngine, jd_to_datetime
from panchang.types import Location, SunData


def _tz_offset_for_date(tz_name: str, year: int, month: int, day: int) -> int:
    """Get UTC offset in seconds for a specific date (handles DST)."""
    tz = ZoneInfo(tz_name)
    dt = datetime(year, month, day, 12, 0, 0, tzinfo=tz)
    return int(dt.utcoffset().total_seconds())


def _get_sun_raw(dt: datetime, location: Location) -> dict:
    """Call Rust core to get raw sun data."""
    py_init(None)
    tz = ZoneInfo(location.tz)

    if dt.tzinfo is None:
        local_dt = dt.replace(tzinfo=tz)
    else:
        local_dt = dt.astimezone(tz)

    utc_offset = _tz_offset_for_date(location.tz, local_dt.year, local_dt.month, local_dt.day)

    return py_compute_sun_data(
        local_dt.year,
        local_dt.month,
        local_dt.day,
        location.lat,
        location.lng,
        location.altitude,
        utc_offset,
    )


def compute_sunrise(
    dt: datetime,
    location: Location,
    engine: EphemerisEngine | None = None,
) -> datetime:
    """Compute sunrise for a given date and location.

    Returns sunrise as a timezone-aware datetime in the location's timezone.
    """
    raw = _get_sun_raw(dt, location)
    sunrise_utc = jd_to_datetime(raw["sunrise_jd"])
    return sunrise_utc.astimezone(ZoneInfo(location.tz))


def compute_sunset(
    dt: datetime,
    location: Location,
    engine: EphemerisEngine | None = None,
) -> datetime:
    """Compute sunset for a given date and location.

    Returns sunset as a timezone-aware datetime in the location's timezone.
    """
    raw = _get_sun_raw(dt, location)
    sunset_utc = jd_to_datetime(raw["sunset_jd"])
    return sunset_utc.astimezone(ZoneInfo(location.tz))


def compute_sun_data(
    dt: datetime,
    location: Location,
    engine: EphemerisEngine | None = None,
) -> SunData:
    """Compute complete solar data for a date and location.

    Returns sunrise, sunset, and day duration.
    """
    raw = _get_sun_raw(dt, location)
    tz = ZoneInfo(location.tz)

    sunrise = jd_to_datetime(raw["sunrise_jd"]).astimezone(tz)
    sunset = jd_to_datetime(raw["sunset_jd"]).astimezone(tz)

    return SunData(
        sunrise=sunrise,
        sunset=sunset,
        day_duration_hours=raw["day_duration_hours"],
        sunrise_jd=raw["sunrise_jd"],
        sunset_jd=raw["sunset_jd"],
    )
