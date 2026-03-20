"""Panchang computation engine for Panchang.

Computes the 5 elements of the Hindu Panchang:
  1. Vara (weekday)
  2. Tithi (lunar day) — based on Sun-Moon angular separation
  3. Nakshatra (lunar mansion) — based on Moon's sidereal longitude
  4. Yoga (Sun-Moon combination) — based on sum of sidereal longitudes
  5. Karana (half-tithi)

All computation is delegated to the Rust _core extension.
"""

from __future__ import annotations

from datetime import datetime
from zoneinfo import ZoneInfo

from panchang._core import py_compute_panchang, py_init
from panchang.core.ephemeris import EphemerisEngine, jd_to_datetime
from panchang.core.sun import _tz_offset_for_date, compute_sun_data
from panchang.types import (
    KaranaInfo,
    Location,
    NakshatraInfo,
    Paksha,
    PanchangData,
    TithiInfo,
    VaraInfo,
    YogaInfo,
)


def compute_panchang(
    dt: datetime,
    location: Location,
    engine: EphemerisEngine | None = None,
) -> PanchangData:
    """Compute complete Panchang for a date and location.

    Args:
        dt: Date/datetime for computation.
        location: Geographic location.
        engine: Optional ephemeris engine (uses default Lahiri if not provided).

    Returns:
        PanchangData with all 5 elements plus sunrise/sunset.
    """
    py_init(None)
    tz = ZoneInfo(location.tz)

    if dt.tzinfo is None:
        local_dt = dt.replace(tzinfo=tz)
    else:
        local_dt = dt.astimezone(tz)

    # Weekday: Python Monday=0, we need Sunday=0
    weekday = (local_dt.weekday() + 1) % 7

    utc_offset = _tz_offset_for_date(location.tz, local_dt.year, local_dt.month, local_dt.day)

    raw = py_compute_panchang(
        local_dt.year,
        local_dt.month,
        local_dt.day,
        location.lat,
        location.lng,
        location.altitude,
        utc_offset,
        weekday,
    )

    # Convert raw dict to typed models
    sun_data = compute_sun_data(local_dt, location, engine)

    vara_raw = raw["vara"]
    vara = VaraInfo(
        number=vara_raw["number"],
        name=vara_raw["name"],
        english=vara_raw["english"],
    )

    tithi_raw = raw["tithi"]
    tithi_num = tithi_raw["number"]
    paksha = Paksha.SHUKLA if tithi_num <= 15 else Paksha.KRISHNA
    tithi = TithiInfo(
        number=tithi_num,
        name=tithi_raw["name"],
        paksha=paksha,
        start=jd_to_datetime(tithi_raw["start_jd"]).astimezone(tz),
        end=jd_to_datetime(tithi_raw["end_jd"]).astimezone(tz),
    )

    nak_raw = raw["nakshatra"]
    nakshatra = NakshatraInfo(
        number=nak_raw["number"],
        name=nak_raw["name"],
        pada=nak_raw["pada"],
        lord=nak_raw["lord"],
        start=jd_to_datetime(nak_raw["start_jd"]).astimezone(tz),
        end=jd_to_datetime(nak_raw["end_jd"]).astimezone(tz),
    )

    yoga_raw = raw["yoga"]
    yoga = YogaInfo(
        number=yoga_raw["number"],
        name=yoga_raw["name"],
        start=jd_to_datetime(yoga_raw["start_jd"]).astimezone(tz),
        end=jd_to_datetime(yoga_raw["end_jd"]).astimezone(tz),
    )

    karana_raw = raw["karana"]
    karana = KaranaInfo(
        number=karana_raw["number"],
        name=karana_raw["name"],
        start=jd_to_datetime(karana_raw["start_jd"]).astimezone(tz),
        end=jd_to_datetime(karana_raw["end_jd"]).astimezone(tz),
    )

    return PanchangData(
        date=local_dt.strftime("%Y-%m-%d"),
        location=location,
        sun=sun_data,
        vara=vara,
        tithi=tithi,
        nakshatra=nakshatra,
        yoga=yoga,
        karana=karana,
    )
