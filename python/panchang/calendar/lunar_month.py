"""Lunar month computation for Amant and Purnimant systems."""

from __future__ import annotations

from panchang._core import py_compute_lunar_months, py_init
from panchang.core.ephemeris import jd_to_datetime
from panchang.types import CalendarSystem, Location, LunarMonthData


def compute_lunar_months(
    year: int,
    location: Location,
    system: CalendarSystem = CalendarSystem.AMANT,
) -> list[LunarMonthData]:
    """Compute all lunar months for a year.

    Args:
        year: Gregorian year.
        location: Geographic location (used only for timezone).
        system: Calendar system (Amant for South Bhārat, Purnimant for North Bhārat).

    Returns:
        List of 12-14 LunarMonthData, covering the Gregorian year.
    """
    py_init(None)
    raw_list = py_compute_lunar_months(year, system.value)

    results = []
    for raw in raw_list:
        results.append(
            LunarMonthData(
                number=raw["number"],
                name=raw["name"],
                is_adhik=raw["is_adhik"],
                is_kshaya=raw["is_kshaya"],
                start=jd_to_datetime(raw["start_jd"]),
                end=jd_to_datetime(raw["end_jd"]),
            )
        )

    return results
