"""Tests for Panchang computation.

Validation data sourced from Drik Panchang (drikpanchang.com) — the de facto
reference for Bhāratīya calendar computations.
"""

from datetime import date, datetime
from zoneinfo import ZoneInfo

from panchang import panchang
from panchang.types import Paksha


class TestPanchangCompute:
    """Basic Panchang computation tests."""

    def test_returns_all_five_elements(self, delhi):
        """Panchang result should contain all 5 elements."""
        result = panchang.compute(date(2026, 2, 24), delhi)

        assert result.vara is not None
        assert result.tithi is not None
        assert result.nakshatra is not None
        assert result.yoga is not None
        assert result.karana is not None

    def test_vara_is_tuesday(self, delhi):
        """2026-02-24 is a Tuesday (Mangalavara)."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert result.vara.english == "Tuesday"
        assert result.vara.name == "Mangalavara"

    def test_tithi_number_range(self, delhi):
        """Tithi number should be 1-30."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert 1 <= result.tithi.number <= 30

    def test_tithi_has_paksha(self, delhi):
        """Tithi should have a valid paksha."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert result.tithi.paksha in (Paksha.SHUKLA, Paksha.KRISHNA)

    def test_nakshatra_number_range(self, delhi):
        """Nakshatra number should be 1-27."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert 1 <= result.nakshatra.number <= 27

    def test_nakshatra_pada_range(self, delhi):
        """Nakshatra pada should be 1-4."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert 1 <= result.nakshatra.pada <= 4

    def test_yoga_number_range(self, delhi):
        """Yoga number should be 1-27."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert 1 <= result.yoga.number <= 27

    def test_karana_number_range(self, delhi):
        """Karana number should be 1-11."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert 1 <= result.karana.number <= 11

    def test_transition_times_present(self, delhi):
        """All elements should have start and end times."""
        result = panchang.compute(date(2026, 2, 24), delhi)

        assert result.tithi.start is not None
        assert result.tithi.end is not None
        assert result.nakshatra.start is not None
        assert result.nakshatra.end is not None
        assert result.yoga.start is not None
        assert result.yoga.end is not None
        assert result.karana.start is not None
        assert result.karana.end is not None

    def test_start_before_end(self, delhi):
        """Start times should be before end times for all elements."""
        result = panchang.compute(date(2026, 2, 24), delhi)

        assert result.tithi.start < result.tithi.end
        assert result.nakshatra.start < result.nakshatra.end
        assert result.yoga.start < result.yoga.end
        assert result.karana.start < result.karana.end


class TestMuhuratWindows:
    """Tests for muhurat time windows included in Panchang."""

    def test_rahu_kalam_present(self, delhi):
        """Rahu Kalam should be included by default."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert result.rahu_kalam is not None
        assert result.rahu_kalam.name == "Rahu Kalam"
        assert result.rahu_kalam.is_auspicious is False

    def test_rahu_kalam_duration(self, delhi):
        """Rahu Kalam should be approximately 1.5 hours."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        duration = (result.rahu_kalam.end - result.rahu_kalam.start).total_seconds() / 3600
        assert 1.3 < duration < 1.7

    def test_abhijit_muhurat_around_noon(self, delhi):
        """Abhijit Muhurat should be around local noon."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert result.abhijit_muhurat is not None
        assert result.abhijit_muhurat.is_auspicious is True
        # Should start around 11:45-12:15 IST
        assert 11 <= result.abhijit_muhurat.start.hour <= 12

    def test_no_muhurat_when_disabled(self, delhi):
        """Muhurat windows should not be computed when include_muhurat=False."""
        result = panchang.compute(date(2026, 2, 24), delhi, include_muhurat=False)
        assert result.rahu_kalam is None
        assert result.yama_gandam is None
        assert result.gulika_kalam is None
        assert result.abhijit_muhurat is None


class TestChoghadiya:
    """Tests for Choghadiya computation."""

    def test_returns_16_windows(self, delhi):
        """Should return 8 day + 8 night = 16 windows."""
        windows = panchang.choghadiya(date(2026, 2, 24), delhi)
        assert len(windows) == 16

    def test_day_windows_within_sunrise_sunset(self, delhi):
        """First 8 windows should be between sunrise and sunset."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        windows = panchang.choghadiya(date(2026, 2, 24), delhi)

        # First window starts at sunrise
        assert abs((windows[0].start - result.sun.sunrise).total_seconds()) < 60

        # 8th window ends at sunset
        assert abs((windows[7].end - result.sun.sunset).total_seconds()) < 60


class TestMultipleLocations:
    """Test Panchang computation across different locations."""

    def test_different_locations_same_date(self, delhi, mumbai, chennai):
        """Same date, different locations should give consistent results."""
        d = date(2026, 2, 24)
        delhi_p = panchang.compute(d, delhi)
        mumbai_p = panchang.compute(d, mumbai)
        chennai_p = panchang.compute(d, chennai)

        # Vara should be the same everywhere
        assert delhi_p.vara.english == mumbai_p.vara.english == chennai_p.vara.english

        # Sunrise should differ (different longitudes)
        assert delhi_p.sun.sunrise != mumbai_p.sun.sunrise

    def test_new_york_timezone(self, new_york):
        """Panchang should work for non-Bhāratīya locations."""
        result = panchang.compute(date(2026, 2, 24), new_york)
        assert result.tithi is not None
        assert result.nakshatra is not None
        # Sunrise should be in EST
        assert result.sun.sunrise.tzinfo is not None


class TestDateInputs:
    """Test various date input formats."""

    def test_date_object(self, delhi):
        """Should accept date objects."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        assert result.date == "2026-02-24"

    def test_datetime_object(self, delhi):
        """Should accept datetime objects."""
        dt = datetime(2026, 2, 24, 10, 30, 0)
        result = panchang.compute(dt, delhi)
        assert result.date == "2026-02-24"

    def test_timezone_aware_datetime(self, delhi):
        """Should accept timezone-aware datetimes."""
        ist = ZoneInfo("Asia/Kolkata")
        dt = datetime(2026, 2, 24, 10, 30, 0, tzinfo=ist)
        result = panchang.compute(dt, delhi)
        assert result.date == "2026-02-24"


class TestMasa:
    """Tests for lunar month (māsa) in Panchang response."""

    def test_masa_present(self, delhi):
        """Māsa should be included in panchang result."""
        result = panchang.compute(date(2026, 4, 12), delhi)
        assert result.masa is not None

    def test_masa_is_caitra_april_2026(self, delhi):
        """April 12, 2026 falls in Caitra māsa (Kṛṣṇa pakṣa)."""
        result = panchang.compute(date(2026, 4, 12), delhi)
        assert result.masa is not None
        assert result.masa.name == "Caitra"
        assert result.masa.number == 1
        assert result.masa.is_adhik is False

    def test_masa_number_range(self, delhi):
        """Māsa number should be 1-12."""
        result = panchang.compute(date(2026, 2, 24), delhi)
        if result.masa:
            assert 1 <= result.masa.number <= 12

    def test_masa_name_is_iast(self, delhi):
        """Māsa name should be in IAST (with diacritics)."""
        result = panchang.compute(date(2026, 4, 12), delhi)
        assert result.masa is not None
        # IAST forms use diacritics — Caitra has none, but Vaiśākha does
        result_may = panchang.compute(date(2026, 5, 5), delhi)
        if result_may.masa:
            assert result_may.masa.name == "Vaiśākha"

    def test_masa_paksha_matches_tithi(self, delhi):
        """Māsa pakṣa should match the tithi pakṣa."""
        result = panchang.compute(date(2026, 4, 12), delhi)
        assert result.masa is not None
        assert result.masa.paksha == result.tithi.paksha


class TestSamvat:
    """Tests for saṃvatsara (era year) in Panchang response."""

    def test_samvat_present(self, delhi):
        """Samvat should be included in panchang result."""
        result = panchang.compute(date(2026, 4, 12), delhi)
        assert result.samvat is not None

    def test_vikram_samvat_2026(self, delhi):
        """April 2026 CE = Vikram Saṃvat 2083."""
        result = panchang.compute(date(2026, 4, 12), delhi)
        assert result.samvat is not None
        assert result.samvat.vikram == 2083

    def test_shaka_samvat_2026(self, delhi):
        """April 2026 CE = Śaka Saṃvat 1948."""
        result = panchang.compute(date(2026, 4, 12), delhi)
        assert result.samvat is not None
        assert result.samvat.shaka == 1948

    def test_samvatsara_name_present(self, delhi):
        """60-year Jovian cycle name should be present."""
        result = panchang.compute(date(2026, 4, 12), delhi)
        assert result.samvat is not None
        assert result.samvat.samvatsara_name is not None
        assert len(result.samvat.samvatsara_name) > 0
