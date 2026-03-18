"""Festival, Ekadashi, and regional calendar data loader.

Loads definitions from YAML files in the data/ directory and provides
them as typed dicts for passing to the Rust computation engine.
"""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path

import yaml


def _data_dir() -> Path:
    """Return the data/ directory at the project root."""
    return Path(__file__).resolve().parents[3] / "data"


@lru_cache(maxsize=1)
def _load_yaml() -> dict:
    """Load and cache the festivals YAML file."""
    yaml_path = _data_dir() / "festivals.yaml"
    with open(yaml_path) as f:
        return yaml.safe_load(f)


@lru_cache(maxsize=1)
def _load_regional_yaml() -> dict:
    """Load and cache the regional calendars YAML file."""
    yaml_path = _data_dir() / "regional_calendars.yaml"
    with open(yaml_path) as f:
        return yaml.safe_load(f)


def get_festival_defs() -> list[dict]:
    """Get festival definitions as list of dicts for Rust bridge.

    Each dict has: id, name, rule, lunar_month, tithi, sankranti_index, nakshatra.
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
