"""Swiss Ephemeris wrapper for Lilavati.

Provides sidereal planetary positions using the Lahiri/Chitrapaksha Ayanamsa,
the standard used by the Government of Bhārat's Rashtriya Panchang.

This module delegates all computation to the Rust _core extension.
"""

from __future__ import annotations

from datetime import datetime, timezone
from enum import IntEnum
from typing import Optional

from panchang._core import (
    PLANET_JUPITER,
    PLANET_KETU,
    PLANET_MARS,
    PLANET_MERCURY,
    PLANET_MOON,
    PLANET_RAHU,
    PLANET_SATURN,
    PLANET_SUN,
    PLANET_VENUS,
    py_ayanamsa,
    py_close,
    py_datetime_to_jd,
    py_init,
    py_jd_to_datetime,
    py_planet_speed,
    py_sidereal_longitude,
    py_tropical_longitude,
)


class Planet(IntEnum):
    """Planet identifiers matching Swiss Ephemeris constants."""

    SUN = PLANET_SUN
    MOON = PLANET_MOON
    MARS = PLANET_MARS
    MERCURY = PLANET_MERCURY
    JUPITER = PLANET_JUPITER
    VENUS = PLANET_VENUS
    SATURN = PLANET_SATURN
    RAHU = PLANET_RAHU
    KETU = PLANET_KETU


class EphemerisEngine:
    """Core ephemeris engine backed by Rust.

    Handles initialization, ayanamsa configuration, and planetary position queries.
    """

    def __init__(
        self,
        ayanamsa: int = 1,  # SE_SIDM_LAHIRI
        ephe_path: Optional[str] = None,
    ):
        self._ayanamsa = ayanamsa
        py_init(ephe_path)

    def close(self) -> None:
        """Release Swiss Ephemeris resources."""
        py_close()

    # --- Julian Day conversion ---

    @staticmethod
    def datetime_to_jd(dt: datetime) -> float:
        """Convert a datetime to Julian Day (UT)."""
        if dt.tzinfo is not None:
            dt = dt.astimezone(timezone.utc)
        sec = dt.second + dt.microsecond / 1_000_000.0
        return py_datetime_to_jd(dt.year, dt.month, dt.day, dt.hour, dt.minute, sec)

    @staticmethod
    def jd_to_datetime(jd: float) -> datetime:
        """Convert Julian Day (UT) to a UTC datetime."""
        year, month, day, hour, minute, second, microsecond = py_jd_to_datetime(jd)
        return datetime(year, month, day, hour, minute, second, microsecond, tzinfo=timezone.utc)

    # --- Ayanamsa ---

    def get_ayanamsa(self, jd: float) -> float:
        """Get the ayanamsa value for a Julian Day."""
        return py_ayanamsa(jd)

    # --- Planetary positions ---

    def get_tropical_longitude(self, jd: float, planet: Planet) -> float:
        """Get tropical longitude of a planet in degrees [0, 360)."""
        return py_tropical_longitude(jd, int(planet))

    def get_sidereal_longitude(self, jd: float, planet: Planet) -> float:
        """Get sidereal longitude of a planet in degrees [0, 360)."""
        return py_sidereal_longitude(jd, int(planet))

    def get_planet_speed(self, jd: float, planet: Planet) -> float:
        """Get the speed of a planet in degrees per day."""
        return py_planet_speed(jd, int(planet))

    # --- Internal helpers (kept for backward compatibility) ---

    def _calc_longitude(self, jd: float, planet: Planet) -> float:
        return py_tropical_longitude(jd, int(planet))

    def _calc_speed(self, jd: float, planet: Planet) -> float:
        return py_planet_speed(jd, int(planet))


# Module-level default engine (lazy initialization)
_default_engine: Optional[EphemerisEngine] = None


def get_engine(
    ayanamsa: int = 1,
    ephe_path: Optional[str] = None,
) -> EphemerisEngine:
    """Get or create the default ephemeris engine."""
    global _default_engine
    if _default_engine is None:
        _default_engine = EphemerisEngine(ayanamsa=ayanamsa, ephe_path=ephe_path)
    return _default_engine


def datetime_to_jd(dt: datetime) -> float:
    """Convert datetime to Julian Day (UT). Convenience function."""
    return EphemerisEngine.datetime_to_jd(dt)


def jd_to_datetime(jd: float) -> datetime:
    """Convert Julian Day (UT) to UTC datetime. Convenience function."""
    return EphemerisEngine.jd_to_datetime(jd)


def normalize_degrees(degrees: float) -> float:
    """Normalize an angle to [0, 360)."""
    return degrees % 360.0
