"""Tests for regional calendar computation."""

from panchang import Location, calendar

DELHI = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")
CHENNAI = Location(lat=13.0827, lng=80.2707, tz="Asia/Kolkata")
KOCHI = Location(lat=9.9312, lng=76.2673, tz="Asia/Kolkata")


class TestListCalendars:
    """Test calendar listing."""

    def test_available_calendars_not_empty(self):
        ids = calendar.list_available_calendars()
        assert len(ids) >= 8

    def test_known_calendars_present(self):
        ids = calendar.list_available_calendars()
        expected_ids = [
            "hindi",
            "tamil",
            "bengali",
            "marathi",
            "telugu",
            "kannada",
            "malayalam",
            "gujarati",
        ]
        for expected in expected_ids:
            assert expected in ids, f"{expected} not in available calendars"


class TestTamilCalendar:
    """Test Tamil (solar) calendar."""

    def test_returns_12_months(self):
        cal = calendar.compute_regional_calendar(2026, CHENNAI, "tamil")
        assert len(cal.months) == 12

    def test_first_month_is_chithirai(self):
        cal = calendar.compute_regional_calendar(2026, CHENNAI, "tamil")
        assert cal.months[0].name == "Chithirai"

    def test_era_name(self):
        cal = calendar.compute_regional_calendar(2026, CHENNAI, "tamil")
        assert cal.era_name == "Thiruvalluvar Aandu"
        assert cal.era_year == 2057  # 2026 + 31

    def test_calendar_type_is_solar(self):
        cal = calendar.compute_regional_calendar(2026, CHENNAI, "tamil")
        assert cal.calendar_type == "solar"

    def test_jovian_year_name_present(self):
        cal = calendar.compute_regional_calendar(2026, CHENNAI, "tamil")
        assert cal.jovian_year_name is not None
        assert len(cal.jovian_year_name) > 0

    def test_new_year_date_present(self):
        cal = calendar.compute_regional_calendar(2026, CHENNAI, "tamil")
        assert cal.new_year_date is not None
        # Tamil new year (Puthandu) is around April 14
        assert cal.new_year_date.month == 4

    def test_months_have_valid_dates(self):
        cal = calendar.compute_regional_calendar(2026, CHENNAI, "tamil")
        for m in cal.months:
            assert m.start <= m.end, f"{m.name}: start {m.start} > end {m.end}"


class TestBengaliCalendar:
    """Test Bengali (solar) calendar."""

    def test_returns_12_months(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "bengali")
        assert len(cal.months) == 12

    def test_first_month_is_boishakh(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "bengali")
        assert cal.months[0].name == "Boishakh"

    def test_era_is_bangabda(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "bengali")
        assert cal.era_name == "Bangabda"
        assert cal.era_year == 1433  # 2026 - 593


class TestMalayalamCalendar:
    """Test Malayalam (solar) calendar."""

    def test_returns_12_months(self):
        cal = calendar.compute_regional_calendar(2026, KOCHI, "malayalam")
        assert len(cal.months) == 12

    def test_first_month_is_chingam(self):
        cal = calendar.compute_regional_calendar(2026, KOCHI, "malayalam")
        assert cal.months[0].name == "Chingam"

    def test_era_is_kollavarsham(self):
        cal = calendar.compute_regional_calendar(2026, KOCHI, "malayalam")
        assert cal.era_name == "Kollavarsham"
        assert cal.era_year == 1201  # 2026 - 825


class TestHindiCalendar:
    """Test Hindi (lunar Purnimant) calendar."""

    def test_returns_months(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "hindi")
        assert len(cal.months) >= 12

    def test_era_is_vikram_samvat(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "hindi")
        assert cal.era_name == "Vikram Samvat"
        assert cal.era_year == 2083  # 2026 + 57

    def test_calendar_type_is_lunar(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "hindi")
        assert cal.calendar_type == "lunar"


class TestMarathiCalendar:
    """Test Marathi (lunar Amant) calendar."""

    def test_returns_months(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "marathi")
        assert len(cal.months) >= 12

    def test_era_is_shaka(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "marathi")
        assert cal.era_name == "Shaka Samvat"
        assert cal.era_year == 1948  # 2026 - 78


class TestTeluguCalendar:
    """Test Telugu (lunar Amant) calendar."""

    def test_returns_months(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "telugu")
        assert len(cal.months) >= 12

    def test_first_standard_month_is_chaitra(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "telugu")
        # Find the Chaitra month
        chaitra = [m for m in cal.months if m.standard_name == "Chaitra"]
        assert len(chaitra) >= 1


class TestGujaratiCalendar:
    """Test Gujarati (lunar Amant, Kartik new year) calendar."""

    def test_returns_months(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "gujarati")
        assert len(cal.months) >= 12

    def test_era_is_vikram_samvat(self):
        cal = calendar.compute_regional_calendar(2026, DELHI, "gujarati")
        assert cal.era_name == "Vikram Samvat"


class TestInvalidCalendar:
    """Test error handling."""

    def test_unknown_calendar_raises(self):
        import pytest

        with pytest.raises(ValueError, match="Unknown calendar"):
            calendar.compute_regional_calendar(2026, DELHI, "nonexistent")
