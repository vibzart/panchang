"""Festival, Ekadashi, and regional calendar data loader.

Loads definitions from YAML files in the data/ directory and provides
them as typed dicts for passing to the Rust computation engine.
"""

from __future__ import annotations

from functools import lru_cache
from importlib import resources

import yaml


def _read_data_file(filename: str) -> str:
    """Read a data file from the panchang.data package using importlib.resources."""
    return resources.files("panchang.data").joinpath(filename).read_text(encoding="utf-8")


@lru_cache(maxsize=1)
def _load_yaml() -> dict:
    """Load and cache the festivals YAML file."""
    return yaml.safe_load(_read_data_file("festivals.yaml"))


@lru_cache(maxsize=1)
def _load_regional_yaml() -> dict:
    """Load and cache the regional calendars YAML file."""
    return yaml.safe_load(_read_data_file("regional_calendars.yaml"))


def get_festival_defs() -> list[dict]:
    """Get festival definitions as list of dicts for Rust bridge.

    Each dict has:
        - id, name, rule              (required)
        - lunar_month, tithi          (for tithi_at_sunrise rule)
        - sankranti_index             (for sankranti rule / nakshatra anchor)
        - nakshatra                   (for nakshatra_at_sunrise rule)
        - priority                    (optional: paraviddha | puurvaviddha | vyapti)
        - kaala                       (optional: sunrise | aparahna | ... — for vyapti)
        - vyapti_tie                  (optional: purva | para — both-days vyapti tie-break)
        - adhika_maasa                (optional: nija | adhika | adhika_and_nija | adhika_if_exists)

    Unknown values for the optional observance fields fall through to Rust defaults
    (paraviddha / sunrise / nija), preserving backward compatibility with YAML
    entries that don't specify them.
    """
    data = _load_yaml()
    defs = []
    for f in data.get("festivals", []):
        defs.append(
            {
                "id": f["id"],
                "name": f["name"],
                "rule": f["rule"],
                "lunar_month": f.get("lunar_month", 0),
                "tithi": f.get("tithi", 0),
                "sankranti_index": f.get("sankranti_index"),
                "nakshatra": f.get("nakshatra"),
                "priority": f.get("priority"),
                "kaala": f.get("kaala"),
                "vyapti_tie": f.get("vyapti_tie"),
                "adhika_maasa": f.get("adhika_maasa"),
            }
        )
    return defs


def get_ekadashi_defs() -> list[dict]:
    """Get Ekadashi definitions as list of dicts for Rust bridge.

    Each dict has: month, shukla_name, krishna_name.
    """
    data = _load_yaml()
    defs = []
    for e in data.get("ekadashis", []):
        defs.append(
            {
                "month": e["month"],
                "shukla_name": e["shukla"],
                "krishna_name": e["krishna"],
            }
        )
    return defs


def get_regional_calendar_def(calendar_id: str) -> dict | None:
    """Get a single regional calendar definition by ID.

    Returns the raw dict from YAML, or None if not found.
    """
    data = _load_regional_yaml()
    for cal in data.get("calendars", []):
        if cal["id"] == calendar_id:
            return cal
    return None


def get_all_regional_calendar_ids() -> list[str]:
    """Get all available regional calendar IDs."""
    data = _load_regional_yaml()
    return [cal["id"] for cal in data.get("calendars", [])]


def get_sixty_year_cycle() -> dict:
    """Get the 60-year Jovian cycle definition.

    Returns dict with 'epoch_year' and 'names' (list of 60 strings).
    """
    data = _load_regional_yaml()
    return data.get("sixty_year_cycle", {"epoch_year": 1987, "names": []})
