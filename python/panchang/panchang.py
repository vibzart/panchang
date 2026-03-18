"""Public API for Panchang computation.

Usage:
    from panchang import panchang, Location
    from datetime import date

    delhi = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")
    today = panchang.compute(date.today(), delhi)

    print(today.tithi.name)
    print(today.nakshatra.name)
    print(today.sunrise)
"""

from __future__ import annotations

from datetime import date, datetime

from panchang._core import py_init
from panchang.core.ephemeris import EphemerisEngine, get_engine
from panchang.core.panchang import compute_panchang
from panchang.muhurat.windows import (
    compute_abhijit_muhurat,
    compute_choghadiya,
    compute_gulika_kalam,
    compute_rahu_kalam,
    compute_yama_gandam,
)
from panchang.types import Location, PanchangData, TimeWindow


def compute(
    dt: date | datetime,
    location: Location,
    engine: EphemerisEngine | None = None,
    include_muhurat: bool = True,
) -> PanchangData:
    """Compute complete Panchang for a date and location.

    Args:
        dt: Date or datetime.
        location: Geographic location with timezone.
        engine: Optional custom ephemeris engine.
        include_muhurat: Whether to include Rahu Kalam, Yama Gandam, etc.

    Returns:
        PanchangData with all 5 Panchang elements, sunrise/sunset, and time windows.
    """
    py_init(None)
    engine = engine or get_engine()

    if isinstance(dt, date) and not isinstance(dt, datetime):
        dt = datetime(dt.year, dt.month, dt.day)

    result = compute_panchang(dt, location, engine)

    if include_muhurat:
        result.rahu_kalam = compute_rahu_kalam(dt, location, result.sun, engine)
        result.yama_gandam = compute_yama_gandam(dt, location, result.sun, engine)
        result.gulika_kalam = compute_gulika_kalam(dt, location, result.sun, engine)
        result.abhijit_muhurat = compute_abhijit_muhurat(dt, location, result.sun, engine)

    return result


def choghadiya(
    dt: date | datetime,
    location: Location,
    engine: EphemerisEngine | None = None,
) -> list[TimeWindow]:
    """Compute all Choghadiya windows for a date and location.

    Returns 16 windows: 8 daytime + 8 nighttime.
    """
    py_init(None)
    engine = engine or get_engine()

    if isinstance(dt, date) and not isinstance(dt, datetime):
        dt = datetime(dt.year, dt.month, dt.day)

    return compute_choghadiya(dt, location, engine=engine)
