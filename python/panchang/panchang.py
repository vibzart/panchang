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

from panchang._core import py_era_year, py_init, py_jovian_cycle_index
from panchang.calendar._data import get_sixty_year_cycle
from panchang.calendar.lunar_month import compute_lunar_months
from panchang.core.ephemeris import EphemerisEngine, get_engine
from panchang.core.panchang import compute_panchang
from panchang.muhurat.windows import (
    compute_abhijit_muhurat,
    compute_choghadiya,
    compute_gulika_kalam,
    compute_rahu_kalam,
    compute_yama_gandam,
)
from panchang.types import (
    CalendarSystem,
    Location,
    MasaInfo,
    PanchangData,
    SamvatInfo,
    TimeWindow,
)


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

    # Lunar month (māsa) — determine which month this date falls into
    try:
        result.masa = _compute_masa(dt, location, result)
    except Exception:
        pass  # degrade gracefully if lunar month computation fails

    # Saṃvatsara — era years + 60-year Jovian cycle name
    try:
        result.samvat = _compute_samvat(dt)
    except Exception:
        pass

    return result


# Month name IAST lookup
_MASA_IAST: dict[str, str] = {
    "Chaitra": "Caitra",
    "Vaishakha": "Vaiśākha",
    "Jyeshtha": "Jyeṣṭha",
    "Ashadha": "Āṣāḍha",
    "Shravana": "Śrāvaṇa",
    "Bhadrapada": "Bhādrapada",
    "Ashwin": "Āśvina",
    "Kartik": "Kārttika",
    "Margashirsha": "Mārgaśīrṣa",
    "Pausha": "Pauṣa",
    "Magha": "Māgha",
    "Phalguna": "Phālguna",
}


def _compute_masa(dt: datetime, location: Location, panchang: PanchangData) -> MasaInfo | None:
    """Determine the lunar month for a given date."""
    year = dt.year
    months = compute_lunar_months(year, location, CalendarSystem.PURNIMANT)

    # Find which lunar month this date falls into (compare UTC timestamps)
    from zoneinfo import ZoneInfo

    utc = ZoneInfo("UTC")
    target_utc = dt.astimezone(utc)

    for m in months:
        start_utc = m.start.astimezone(utc) if m.start.tzinfo else m.start.replace(tzinfo=utc)
        end_utc = m.end.astimezone(utc) if m.end.tzinfo else m.end.replace(tzinfo=utc)
        if start_utc <= target_utc <= end_utc:
            return MasaInfo(
                number=m.number,
                name=_MASA_IAST.get(m.name, m.name),
                is_adhik=m.is_adhik,
                paksha=panchang.tithi.paksha,
            )

    return None


def _compute_samvat(dt: datetime) -> SamvatInfo:
    """Compute era years and 60-year Jovian cycle name."""
    year = dt.year

    # Approximate: Chaitra Shukla Pratipada usually falls in March-April.
    # Before April → previous year's samvat for Vikram; after → current.
    new_year_passed = dt.month >= 4

    vikram = py_era_year(year, 57, new_year_passed)
    shaka = py_era_year(year, -78, new_year_passed)

    # 60-year Jovian cycle
    cycle_data = get_sixty_year_cycle()
    cycle_names = cycle_data.get("names", [])
    jovian_name = None
    if cycle_names:
        epoch = cycle_data.get("epoch_year", 1987)
        idx = py_jovian_cycle_index(year, epoch)
        if idx < len(cycle_names):
            jovian_name = cycle_names[idx]

    return SamvatInfo(
        vikram=vikram,
        shaka=shaka,
        samvatsara_name=jovian_name,
    )


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
