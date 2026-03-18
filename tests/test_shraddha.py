"""Tests for Shraddha Tithi calculator."""

from datetime import date

from panchang import Location, calendar

DELHI = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")


class TestShraddhaComputation:
    """Test Shraddha (death anniversary) date computation."""

    def test_returns_result(self):
        result = calendar.compute_shraddha(date(2020, 6, 15), 2026, DELHI)
        assert result is not None

    def test_shraddha_date_in_target_year(self):
        result = calendar.compute_shraddha(date(2020, 6, 15), 2026, DELHI)
        assert result is not None
        assert result.shraddha_date.year == 2026

    def test_tithi_in_valid_range(self):
        result = calendar.compute_shraddha(date(2020, 6, 15), 2026, DELHI)
        assert result is not None
        assert 1 <= result.tithi <= 30

    def test_lunar_month_in_valid_range(self):
        result = calendar.compute_shraddha(date(2020, 6, 15), 2026, DELHI)
        assert result is not None
        assert 1 <= result.lunar_month <= 12

    def test_reasoning_not_empty(self):
        result = calendar.compute_shraddha(date(2020, 6, 15), 2026, DELHI)
        assert result is not None
        assert len(result.reasoning) > 0

    def test_death_date_preserved(self):
        death = date(2015, 3, 20)
        result = calendar.compute_shraddha(death, 2026, DELHI)
        assert result is not None
        assert result.death_date == death

    def test_different_years_same_tithi(self):
        """Shraddha in different years should have the same tithi."""
        death = date(2010, 8, 10)
        r2025 = calendar.compute_shraddha(death, 2025, DELHI)
        r2026 = calendar.compute_shraddha(death, 2026, DELHI)
        assert r2025 is not None
        assert r2026 is not None
        assert r2025.tithi == r2026.tithi
        assert r2025.lunar_month == r2026.lunar_month

    def test_different_years_different_dates(self):
        """Shraddha in different years should fall on different Gregorian dates."""
        death = date(2010, 8, 10)
        r2025 = calendar.compute_shraddha(death, 2025, DELHI)
        r2026 = calendar.compute_shraddha(death, 2026, DELHI)
        assert r2025 is not None
        assert r2026 is not None
        assert r2025.shraddha_date != r2026.shraddha_date

    def test_lunar_month_name_valid(self):
        result = calendar.compute_shraddha(date(2018, 1, 15), 2026, DELHI)
        assert result is not None
        valid_months = [
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
        assert result.lunar_month_name in valid_months
