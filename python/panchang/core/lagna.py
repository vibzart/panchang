"""Lagna (sidereal ascendant) and bhāva-cusp computation.

Wraps the Rust `_core.py_compute_lagna` function — the same Swiss
Ephemeris path used for planetary positions, but applied to the
ascendant + 12 house cusps in the Lahiri sidereal frame.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import List, Tuple

from panchang._core import py_compute_lagna, py_compute_lagna_windows


class HouseSystem(str, Enum):
    """House system letters accepted by Swiss Ephemeris.

    The string values are the single-letter codes passed straight
    through to the C library (`hsys` argument of `swe_houses_ex`).
    """

    PLACIDUS = "P"
    """Placidus — modern Bhāva Chalit default."""

    WHOLE_SIGN = "W"
    """Whole-sign — each rashi is a full bhāva."""

    EQUAL = "E"
    """Equal house from the ascendant (every 30°)."""

    PORPHYRY = "O"
    """Porphyry / Sripati — equal-arc trisection between angles."""


@dataclass(frozen=True)
class LagnaInfo:
    """Result of a Lagna computation."""

    ascendant_longitude: float
    """Sidereal longitude of the Lagna in degrees, [0, 360)."""

    rashi: int
    """Rashi index 0..=11 (0 = Mesha, 11 = Meena)."""

    rashi_name: str
    """Sanskrit rashi name (e.g. ``"Vrishabha"``)."""

    degree_in_rashi: float
    """Degrees within the rashi, [0, 30)."""

    mc_longitude: float
    """Sidereal Midheaven longitude, [0, 360)."""

    bhava_cusps: Tuple[float, ...]
    """12 sidereal house-cusp degrees, indexed 0..=11
    where index 0 is the 1st bhāva (Lagna bhava)."""

    house_system: HouseSystem
    """Which house system was used to compute the cusps."""

    ayanamsa: float
    """Lahiri ayanamsa (degrees) at this jd. Surfaced for the
    ``/transparency`` page so readers can see exactly which
    ayanamsa epoch their chart was computed against."""


@dataclass(frozen=True)
class LagnaWindow:
    """A single rashi-rising window during a day."""

    rashi: int
    """Rashi index 0..=11."""

    rashi_name: str
    """Sanskrit rashi name."""

    start_jd: float
    """JD (UT) when this rashi began rising."""

    end_jd: float
    """JD (UT) when the next rashi began rising."""


def compute_lagna(
    jd: float,
    lat: float,
    lng: float,
    system: HouseSystem | str = HouseSystem.PLACIDUS,
) -> LagnaInfo:
    """Compute the sidereal Lagna and 12 bhāva cusps.

    Args:
        jd: Julian Day in UT (already adjusted for timezone).
        lat: Geographic latitude in degrees, north positive.
        lng: Geographic longitude in degrees, east positive.
        system: House system to use for the cusps. Defaults to
            Placidus, the most common modern Bhāva Chalit definition.

    Returns:
        A :class:`LagnaInfo` with ascendant, rashi placement, MC, and
        all 12 cusps in the Lahiri sidereal frame.
    """
    if isinstance(system, HouseSystem):
        sys_str = system.value
    else:
        sys_str = system

    raw = py_compute_lagna(jd, lat, lng, sys_str)
    return LagnaInfo(
        ascendant_longitude=raw["ascendant_longitude"],
        rashi=raw["rashi"],
        rashi_name=raw["rashi_name"],
        degree_in_rashi=raw["degree_in_rashi"],
        mc_longitude=raw["mc_longitude"],
        bhava_cusps=tuple(raw["bhava_cusps"]),
        house_system=HouseSystem(raw["house_system"]),
        ayanamsa=raw["ayanamsa"],
    )


def compute_lagna_windows(jd_start: float, lat: float, lng: float) -> List[LagnaWindow]:
    """Compute the 12 Lagna-rising windows over the 24h starting at ``jd_start``.

    Used by the Khona intake flow when birth time is approximate. If the
    user knows their birth only "in the morning", we display which lagnas
    were rising during that window so the chart can be appropriately gated.

    Args:
        jd_start: Julian Day (UT) marking the start of the 24-hour span.
            Typically local sunrise.
        lat: Geographic latitude in degrees, north positive.
        lng: Geographic longitude in degrees, east positive.

    Returns:
        A list of :class:`LagnaWindow` objects sorted by ``start_jd``,
        spanning exactly 24 hours from ``jd_start``.
    """
    raw = py_compute_lagna_windows(jd_start, lat, lng)
    return [
        LagnaWindow(
            rashi=w["rashi"],
            rashi_name=w["rashi_name"],
            start_jd=w["start_jd"],
            end_jd=w["end_jd"],
        )
        for w in raw
    ]
