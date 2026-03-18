"""Shraddha Tithi (death anniversary) calculator.

Computes the annual Shraddha date by finding when the same Tithi
(in the same lunar month) as the death date recurs in a target year.
"""

from __future__ import annotations

from datetime import date

from panchang._core import py_compute_shraddha, py_init
from panchang.core.sun import _tz_offset_for_date
from panchang.types import Location, ShraddhaData


def compute_shraddha(
    death_date: date,
    target_year: int,
    location: Location,
) -> ShraddhaData | None:
    """Compute the Shraddha (death anniversary) date for a target year.

    The Shraddha falls on the same Tithi and lunar month as the death date.

    Args:
        death_date: The original date of death.
        target_year: Gregorian year to find the Shraddha in.
        location: Geographic location (sunrise times affect Tithi).

    Returns:
        ShraddhaData with the resolved date and reasoning, or None if
        the Tithi could not be resolved.
    """
    py_init(None)
    utc_offset = _tz_offset_for_date(location.tz, target_year, 6, 15)

    raw = py_compute_shraddha(
        death_date.year,
        death_date.month,
        death_date.day,
        target_year,
        location.lat,
        location.lng,
        location.altitude,
        utc_offset,
    )

    if raw is None:
        return None

    return ShraddhaData(
        death_date=death_date,
        tithi=raw["tithi"],
        lunar_month=raw["lunar_month"],
        lunar_month_name=raw["lunar_month_name"],
        shraddha_date=date(raw["year"], raw["month"], raw["day"]),
        reasoning=raw["reasoning"],
    )
