"""Muhurat time window calculations for Panchang.

Computes daily inauspicious/auspicious time windows based on sunrise and sunset:
  - Rahu Kalam: 1.5-hour inauspicious window (position varies by weekday)
  - Yama Gandam: 1.5-hour inauspicious window (position varies by weekday)
  - Gulika Kalam: 1.5-hour inauspicious window (position varies by weekday)
  - Abhijit Muhurat: Auspicious midday window (~48 minutes around local noon)
  - Choghadiya: 8 daytime + 8 nighttime windows of equal duration

All computation is delegated to the Rust _core extension.
"""

from __future__ import annotations

from datetime import datetime
from zoneinfo import ZoneInfo

from panchang._core import py_compute_choghadiya, py_compute_muhurat, py_init
from panchang.core.ephemeris import EphemerisEngine, jd_to_datetime
from panchang.core.sun import compute_sun_data
from panchang.types import Location, SunData, TimeWindow


def _jd_to_local(jd: float, tz: ZoneInfo) -> datetime:
    """Convert JD to timezone-aware local datetime."""
    return jd_to_datetime(jd).astimezone(tz)


def compute_rahu_kalam(
    dt: datetime,
    location: Location,
    sun_data: SunData | None = None,
    engine: EphemerisEngine | None = None,
) -> TimeWindow:
    """Compute Rahu Kalam for a given date and location."""
    py_init(None)
    sun_data = sun_data or compute_sun_data(dt, location, engine)
    tz = ZoneInfo(location.tz)
    weekday = (dt.weekday() + 1) % 7

    raw = py_compute_muhurat(weekday, sun_data.sunrise_jd, sun_data.day_duration_hours)
    rk = raw["rahu_kalam"]
    return TimeWindow(
        name=rk["name"],
        start=_jd_to_local(rk["start_jd"], tz),
        end=_jd_to_local(rk["end_jd"], tz),
        is_auspicious=rk["is_auspicious"],
    )


def compute_yama_gandam(
    dt: datetime,
    location: Location,
    sun_data: SunData | None = None,
    engine: EphemerisEngine | None = None,
) -> TimeWindow:
    """Compute Yama Gandam for a given date and location."""
    py_init(None)
    sun_data = sun_data or compute_sun_data(dt, location, engine)
    tz = ZoneInfo(location.tz)
    weekday = (dt.weekday() + 1) % 7

    raw = py_compute_muhurat(weekday, sun_data.sunrise_jd, sun_data.day_duration_hours)
    yg = raw["yama_gandam"]
    return TimeWindow(
        name=yg["name"],
        start=_jd_to_local(yg["start_jd"], tz),
        end=_jd_to_local(yg["end_jd"], tz),
        is_auspicious=yg["is_auspicious"],
    )


def compute_gulika_kalam(
    dt: datetime,
    location: Location,
    sun_data: SunData | None = None,
    engine: EphemerisEngine | None = None,
) -> TimeWindow:
    """Compute Gulika Kalam for a given date and location."""
    py_init(None)
    sun_data = sun_data or compute_sun_data(dt, location, engine)
    tz = ZoneInfo(location.tz)
    weekday = (dt.weekday() + 1) % 7

    raw = py_compute_muhurat(weekday, sun_data.sunrise_jd, sun_data.day_duration_hours)
    gk = raw["gulika_kalam"]
    return TimeWindow(
        name=gk["name"],
        start=_jd_to_local(gk["start_jd"], tz),
        end=_jd_to_local(gk["end_jd"], tz),
        is_auspicious=gk["is_auspicious"],
    )


def compute_abhijit_muhurat(
    dt: datetime,
    location: Location,
    sun_data: SunData | None = None,
    engine: EphemerisEngine | None = None,
) -> TimeWindow:
    """Compute Abhijit Muhurat — the universally auspicious midday window."""
    py_init(None)
    sun_data = sun_data or compute_sun_data(dt, location, engine)
    tz = ZoneInfo(location.tz)

    raw = py_compute_muhurat(
        (dt.weekday() + 1) % 7,
        sun_data.sunrise_jd,
        sun_data.day_duration_hours,
    )
    am = raw["abhijit_muhurat"]
    return TimeWindow(
        name=am["name"],
        start=_jd_to_local(am["start_jd"], tz),
        end=_jd_to_local(am["end_jd"], tz),
        is_auspicious=am["is_auspicious"],
    )


def compute_choghadiya(
    dt: datetime,
    location: Location,
    sun_data: SunData | None = None,
    engine: EphemerisEngine | None = None,
) -> list[TimeWindow]:
    """Compute all Choghadiya windows (8 day + 8 night) for a date."""
    py_init(None)
    sun_data = sun_data or compute_sun_data(dt, location, engine)
    tz = ZoneInfo(location.tz)
    weekday = (dt.weekday() + 1) % 7

    raw_windows = py_compute_choghadiya(
        weekday,
        sun_data.sunrise_jd,
        sun_data.sunset_jd,
        sun_data.day_duration_hours,
    )

    return [
        TimeWindow(
            name=w["name"],
            start=_jd_to_local(w["start_jd"], tz),
            end=_jd_to_local(w["end_jd"], tz),
            is_auspicious=w["is_auspicious"],
        )
        for w in raw_windows
    ]
