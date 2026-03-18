"""Festival, Ekadashi, and Vrat date computation."""

from __future__ import annotations

from datetime import date

from panchang._core import (
    py_compute_ekadashis,
    py_compute_festivals,
    py_compute_vrat_dates,
    py_init,
)
from panchang.calendar._data import get_ekadashi_defs, get_festival_defs
from panchang.core.ephemeris import jd_to_datetime
from panchang.core.sun import _tz_offset_for_date
from panchang.types import (
    EkadashiInfo,
    FestivalInfo,
    Location,
    Paksha,
    VratInfo,
)


def compute_festivals(year: int, location: Location) -> list[FestivalInfo]:
    """Compute all festival dates for a year.

    Festival definitions are loaded from data/festivals.yaml.

    Args:
        year: Gregorian year.
        location: Geographic location (sunrise times affect festival dates).

    Returns:
        List of FestivalInfo, sorted chronologically.
    """
    py_init(None)
    utc_offset = _tz_offset_for_date(location.tz, year, 6, 15)
    defs = get_festival_defs()

    raw_list = py_compute_festivals(
        defs,
        year,
        location.lat,
        location.lng,
        location.altitude,
        utc_offset,
    )

    results = []
    for raw in raw_list:
        sunrise_dt = jd_to_datetime(raw["sunrise_jd"]) if raw["sunrise_jd"] else None
        results.append(
            FestivalInfo(
                id=raw["festival_id"],
                name=raw["festival_name"],
                date=date(raw["year"], raw["month"], raw["day"]),
                sunrise=sunrise_dt,
                tithi_at_sunrise=raw["tithi_at_sunrise"],
                lunar_month=raw["lunar_month_name"],
                is_adhik_month=raw["is_adhik_month"],
                reasoning=raw["reasoning"],
            )
        )

    return results


def compute_ekadashis(year: int, location: Location) -> list[EkadashiInfo]:
    """Compute all 24 Ekadashis for a year.

    Each Ekadashi has both Smartha and Vaishnava dates.
    The Vaishnava date may differ by one day if Dashami persists at Arunodaya.

    Args:
        year: Gregorian year.
        location: Geographic location.

    Returns:
        List of EkadashiInfo, sorted chronologically.
    """
    py_init(None)
    utc_offset = _tz_offset_for_date(location.tz, year, 6, 15)
    defs = get_ekadashi_defs()

    raw_list = py_compute_ekadashis(
        defs,
        year,
        location.lat,
        location.lng,
        location.altitude,
        utc_offset,
    )

    results = []
    for raw in raw_list:
        results.append(
            EkadashiInfo(
                name=raw["name"],
                lunar_month=raw["lunar_month"],
                lunar_month_name=raw["lunar_month_name"],
                paksha=Paksha(raw["paksha"]),
                smartha_date=date(raw["smartha_year"], raw["smartha_month"], raw["smartha_day"]),
                vaishnava_date=date(
                    raw["vaishnava_year"],
                    raw["vaishnava_month"],
                    raw["vaishnava_day"],
                ),
                reasoning=raw["reasoning"],
            )
        )

    return results


def compute_vrat_dates(year: int, location: Location) -> list[VratInfo]:
    """Compute monthly Vrat (fasting) dates for a year.

    Includes: Pradosh Vrat, Sankashti Chaturthi, Amavasya, Purnima.

    Args:
        year: Gregorian year.
        location: Geographic location.

    Returns:
        List of VratInfo, sorted chronologically.
    """
    py_init(None)
    utc_offset = _tz_offset_for_date(location.tz, year, 6, 15)

    raw_list = py_compute_vrat_dates(
        year,
        location.lat,
        location.lng,
        location.altitude,
        utc_offset,
    )

    results = []
    for raw in raw_list:
        results.append(
            VratInfo(
                vrat_type=raw["vrat_type"],
                name=raw["name"],
                date=date(raw["year"], raw["month"], raw["day"]),
                lunar_month=raw["lunar_month_name"],
                paksha=Paksha(raw["paksha"]),
            )
        )

    return results
