"""Sankranti (solar ingress) computation."""

from __future__ import annotations

from datetime import date

from panchang._core import py_compute_sankrantis, py_init
from panchang.types import Location, SankrantiData


def compute_sankrantis(year: int, location: Location) -> list[SankrantiData]:
    """Compute all 12 Sankrantis for a year.

    A Sankranti occurs when the Sun's sidereal longitude crosses a
    multiple of 30 degrees, marking the transition between zodiac signs.

    Args:
        year: Gregorian year.
        location: Geographic location (used only for timezone).

    Returns:
        List of 12 SankrantiData, sorted chronologically.
    """
    py_init(None)
    raw_list = py_compute_sankrantis(year)

    results = []
    for raw in raw_list:
        results.append(
            SankrantiData(
                index=raw["index"],
                name=raw["name"],
                rashi=raw["rashi"],
                target_longitude=raw["target_longitude"],
                date=date(raw["year"], raw["month"], raw["day"]),
            )
        )

    return results
