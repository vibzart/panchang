"""Tests for lunar month computation."""

from panchang import Location, calendar
from panchang.types import CalendarSystem

DELHI = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")


class TestLunarMonths:
    def test_amant_month_count(self):
        months = calendar.compute_lunar_months(2026, DELHI, CalendarSystem.AMANT)
        assert 12 <= len(months) <= 14, f"Got {len(months)} months"

    def test_purnimant_month_count(self):
        months = calendar.compute_lunar_months(2026, DELHI, CalendarSystem.PURNIMANT)
        assert 12 <= len(months) <= 14, f"Got {len(months)} months"

    def test_all_month_names_valid(self):
        valid = {
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
        }
        months = calendar.compute_lunar_months(2026, DELHI)
        for m in months:
            assert m.name in valid, f"Invalid month name: {m.name}"

    def test_month_boundaries_are_datetimes(self):
        months = calendar.compute_lunar_months(2026, DELHI)
        for m in months:
            assert m.start is not None
            assert m.end is not None
            assert m.end > m.start

    def test_adhik_maas_detected(self):
        """2026 has an Adhik Maas in the Amant system."""
        months = calendar.compute_lunar_months(2026, DELHI, CalendarSystem.AMANT)
        adhik_months = [m for m in months if m.is_adhik]
        assert len(adhik_months) >= 1, "Expected at least one Adhik Maas in 2026"
