"""Regional calendar computation.

Builds regional calendars (Tamil, Telugu, Bengali, Marathi, etc.) by
combining computed Sankrantis and lunar months with regional naming
conventions loaded from data/regional_calendars.yaml.
"""

from __future__ import annotations

from datetime import date

from panchang._core import py_era_year, py_jovian_cycle_index
from panchang.calendar._data import (
    get_all_regional_calendar_ids,
    get_regional_calendar_def,
    get_sixty_year_cycle,
)
from panchang.calendar.lunar_month import compute_lunar_months
from panchang.calendar.sankranti import compute_sankrantis
from panchang.types import (
    CalendarSystem,
    Location,
    RegionalCalendarData,
    RegionalMonthInfo,
)

# Standard Sanskrit lunar month names (index 0 = Chaitra)
_STANDARD_LUNAR_MONTHS = [
    "Chaitra",
    "Vaishakha",
    "Jyeshtha",
    "Ashadha",
    "Shravana",
    "Bhadrapada",
    "Ashwin",
    "Kartik",
    "Margashirsha",
    "Pausha",
    "Magha",
    "Phalguna",
]

# Standard Rashi names (index 0 = Mesha)
_STANDARD_RASHI_NAMES = [
    "Mesha",
    "Vrishabha",
    "Mithuna",
    "Karka",
    "Simha",
    "Kanya",
    "Tula",
    "Vrischika",
    "Dhanu",
    "Makara",
    "Kumbha",
    "Meena",
]

# Sankranti index → Rashi index mapping (same as Rust SANKRANTI_RASHI_INDEX)
_SANKRANTI_RASHI_INDEX = [9, 10, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8]

# Rashi index → lunar month number (1-based)
_RASHI_TO_LUNAR_MONTH = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]


def _jd_to_date(jd: float) -> date:
    """Convert JD to a date (approximate, for month boundary display)."""
    from panchang.core.ephemeris import jd_to_datetime

    dt = jd_to_datetime(jd)
    return dt.date()


def compute_regional_calendar(
    year: int,
    location: Location,
    calendar_id: str,
) -> RegionalCalendarData:
    """Compute a regional calendar for a given year and location.

    Args:
        year: Gregorian year.
        location: Geographic location.
        calendar_id: Regional calendar ID (e.g. 'tamil', 'bengali', 'marathi').

    Returns:
        RegionalCalendarData with months, era year, and optional Jovian cycle name.

    Raises:
        ValueError: If calendar_id is not found in regional_calendars.yaml.
    """
    cal_def = get_regional_calendar_def(calendar_id)
    if cal_def is None:
        available = get_all_regional_calendar_ids()
        raise ValueError(f"Unknown calendar '{calendar_id}'. Available: {available}")

    cal_type = cal_def["type"]
    era_def = cal_def.get("era", {})

    if cal_type == "solar":
        months, new_year_date = _build_solar_months(year, location, cal_def)
    else:
        months, new_year_date = _build_lunar_months(year, location, cal_def)

    # Compute era year
    new_year_passed = new_year_date is not None and new_year_date <= date(year, 12, 31)
    era_offset = era_def.get("offset", 0)
    era_year = py_era_year(year, era_offset, new_year_passed)
    era_name = era_def.get("name", "CE")

    # Compute 60-year Jovian cycle name (used by Tamil, Telugu, Kannada)
    cycle_data = get_sixty_year_cycle()
    cycle_names = cycle_data.get("names", [])
    jovian_name = None
    if cycle_names:
        epoch = cycle_data.get("epoch_year", 1987)
        idx = py_jovian_cycle_index(year, epoch)
        if idx < len(cycle_names):
            jovian_name = cycle_names[idx]

    return RegionalCalendarData(
        id=cal_def["id"],
        name=cal_def["name"],
        language=cal_def.get("language", ""),
        calendar_type=cal_type,
        era_name=era_name,
        era_year=era_year,
        jovian_year_name=jovian_name,
        months=months,
        new_year_date=new_year_date,
    )


def _build_solar_months(
    year: int,
    location: Location,
    cal_def: dict,
) -> tuple[list[RegionalMonthInfo], date | None]:
    """Build months for solar calendars (Tamil, Bengali, Malayalam, etc.).

    Solar months start when the Sun enters a new Rashi (Sankranti).
    """
    sankrantis = compute_sankrantis(year, location)
    # Also get previous and next year for boundary months
    prev_sankrantis = compute_sankrantis(year - 1, location)
    next_sankrantis = compute_sankrantis(year + 1, location)

    # Build a lookup: rashi_index → Sankranti date
    all_sankrantis = prev_sankrantis + sankrantis + next_sankrantis
    rashi_to_sankranti: dict[int, list[date]] = {}
    for s in all_sankrantis:
        rashi_idx = _SANKRANTI_RASHI_INDEX[s.index]
        rashi_to_sankranti.setdefault(rashi_idx, []).append(s.date)

    month_defs = cal_def.get("months", [])
    months = []
    new_year_date = None

    for i, m_def in enumerate(month_defs):
        rashi_idx = m_def["rashi_index"]
        regional_name = m_def["name"]
        standard_name = _STANDARD_RASHI_NAMES[rashi_idx]

        # Find the Sankranti date closest to this year
        candidates = rashi_to_sankranti.get(rashi_idx, [])
        # Pick the one in or closest to the target year
        best = None
        for d in candidates:
            if best is None or abs(d.year - year) < abs(best.year - year):
                best = d
            elif abs(d.year - year) == abs(best.year - year) and d > best:
                best = d

        if best is None:
            continue

        start_date = best

        # End date is the next month's start
        if i + 1 < len(month_defs):
            next_rashi = month_defs[i + 1]["rashi_index"]
            next_candidates = rashi_to_sankranti.get(next_rashi, [])
            end_candidates = [d for d in next_candidates if d > start_date]
            end_date = min(end_candidates) if end_candidates else start_date
        else:
            # Last month in regional order — wraps to first month of next cycle
            first_rashi = month_defs[0]["rashi_index"]
            next_candidates = rashi_to_sankranti.get(first_rashi, [])
            end_candidates = [d for d in next_candidates if d > start_date]
            end_date = min(end_candidates) if end_candidates else start_date

        months.append(
            RegionalMonthInfo(
                name=regional_name,
                standard_name=standard_name,
                start=start_date,
                end=end_date,
            )
        )

        if i == 0:
            new_year_date = start_date

    return months, new_year_date


def _build_lunar_months(
    year: int,
    location: Location,
    cal_def: dict,
) -> tuple[list[RegionalMonthInfo], date | None]:
    """Build months for lunar calendars (Hindi, Marathi, Telugu, etc.).

    Lunar months come from compute_lunar_months() with regional names applied.
    """
    system_str = cal_def.get("system", "amant")
    system = CalendarSystem.PURNIMANT if system_str == "purnimant" else CalendarSystem.AMANT

    lunar_months = compute_lunar_months(year, location, system)

    # Build name mapping: standard month number → regional name
    month_defs = cal_def.get("months", [])
    number_to_regional: dict[int, str] = {}
    regional_order: list[int] = []
    for m_def in month_defs:
        num = m_def["number"]
        number_to_regional[num] = m_def["name"]
        regional_order.append(num)

    months = []
    new_year_date = None

    # Map computed lunar months to regional names
    for lm in lunar_months:
        if lm.number == 0:
            continue
        regional_name = number_to_regional.get(lm.number, lm.name)
        standard_name = _STANDARD_LUNAR_MONTHS[lm.number - 1] if 1 <= lm.number <= 12 else lm.name
        if lm.is_adhik:
            regional_name = f"Adhik {regional_name}"

        start_date = lm.start.date()
        end_date = lm.end.date()

        months.append(
            RegionalMonthInfo(
                name=regional_name,
                standard_name=standard_name,
                start=start_date,
                end=end_date,
            )
        )

    # Determine new year date from the new_year config
    new_year_def = cal_def.get("new_year", {})
    if new_year_def.get("type") == "tithi":
        ny_month_num = new_year_def.get("lunar_month", 1)
        for lm in lunar_months:
            if lm.number == ny_month_num and not lm.is_adhik:
                new_year_date = lm.start.date()
                break

    return months, new_year_date


def list_available_calendars() -> list[str]:
    """Return all available regional calendar IDs."""
    return get_all_regional_calendar_ids()
