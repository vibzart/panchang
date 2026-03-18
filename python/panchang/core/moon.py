"""Moon position and phase calculations for Lilavati."""

from __future__ import annotations

from datetime import datetime

from panchang.core.ephemeris import (
    EphemerisEngine,
    Planet,
    datetime_to_jd,
    get_engine,
    normalize_degrees,
)
from panchang.types import MoonData


def compute_moon_data(
    dt: datetime,
    engine: EphemerisEngine | None = None,
) -> MoonData:
    """Compute Moon sidereal longitude and phase angle for a given datetime."""
    engine = engine or get_engine()
    jd = datetime_to_jd(dt)

    moon_sid = engine.get_sidereal_longitude(jd, Planet.MOON)
    sun_trop = engine.get_tropical_longitude(jd, Planet.SUN)
    moon_trop = engine.get_tropical_longitude(jd, Planet.MOON)
    phase_angle = normalize_degrees(moon_trop - sun_trop)

    return MoonData(
        longitude=round(moon_sid, 4),
        phase_angle=round(phase_angle, 4),
    )
