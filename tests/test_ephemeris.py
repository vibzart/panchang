"""Tests for the ephemeris wrapper."""

from datetime import datetime, timezone

import pytest

from panchang.core.ephemeris import EphemerisEngine, Planet, datetime_to_jd, jd_to_datetime


class TestJulianDayConversion:
    """Tests for datetime <-> Julian Day conversion."""

    def test_known_julian_day(self):
        """J2000.0 epoch: 2000-01-01 12:00 UT = JD 2451545.0"""
        dt = datetime(2000, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        jd = datetime_to_jd(dt)
        assert abs(jd - 2451545.0) < 0.0001

    def test_roundtrip_conversion(self):
        """datetime -> JD -> datetime should preserve time to within 1 second."""
        original = datetime(2026, 2, 24, 6, 30, 0, tzinfo=timezone.utc)
        jd = datetime_to_jd(original)
        recovered = jd_to_datetime(jd)
        diff = abs((recovered - original).total_seconds())
        assert diff < 1.0

    def test_naive_datetime_treated_as_utc(self):
        """Naive datetimes should be treated as UTC."""
        naive = datetime(2026, 6, 15, 10, 0, 0)
        aware = datetime(2026, 6, 15, 10, 0, 0, tzinfo=timezone.utc)
        assert abs(datetime_to_jd(naive) - datetime_to_jd(aware)) < 0.0001

    def test_timezone_aware_conversion(self):
        """Timezone-aware datetimes should be converted to UTC first."""
        from zoneinfo import ZoneInfo

        ist = ZoneInfo("Asia/Kolkata")
        dt_ist = datetime(2026, 2, 24, 12, 0, 0, tzinfo=ist)  # Noon IST = 06:30 UTC
        dt_utc = datetime(2026, 2, 24, 6, 30, 0, tzinfo=timezone.utc)
        assert abs(datetime_to_jd(dt_ist) - datetime_to_jd(dt_utc)) < 0.0001


class TestEphemerisEngine:
    """Tests for planetary position calculations."""

    @pytest.fixture(autouse=True)
    def setup_engine(self):
        self.engine = EphemerisEngine()
        yield
        self.engine.close()

    def test_sun_longitude_range(self):
        """Sun longitude should always be in [0, 360)."""
        jd = datetime_to_jd(datetime(2026, 2, 24, 12, 0, 0, tzinfo=timezone.utc))
        sun_long = self.engine.get_tropical_longitude(jd, Planet.SUN)
        assert 0.0 <= sun_long < 360.0

    def test_moon_longitude_range(self):
        """Moon longitude should always be in [0, 360)."""
        jd = datetime_to_jd(datetime(2026, 2, 24, 12, 0, 0, tzinfo=timezone.utc))
        moon_long = self.engine.get_tropical_longitude(jd, Planet.MOON)
        assert 0.0 <= moon_long < 360.0

    def test_sidereal_less_than_tropical(self):
        """Sidereal longitude should be less than tropical (Lahiri ayanamsa ~24°)."""
        jd = datetime_to_jd(datetime(2026, 2, 24, 12, 0, 0, tzinfo=timezone.utc))
        tropical = self.engine.get_tropical_longitude(jd, Planet.SUN)
        sidereal = self.engine.get_sidereal_longitude(jd, Planet.SUN)
        ayanamsa = self.engine.get_ayanamsa(jd)
        # sidereal = (tropical - ayanamsa) % 360
        expected = (tropical - ayanamsa) % 360.0
        assert abs(sidereal - expected) < 0.001

    def test_ayanamsa_in_expected_range(self):
        """Lahiri Ayanamsa in 2026 should be approximately 24.2°."""
        jd = datetime_to_jd(datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc))
        ayanamsa = self.engine.get_ayanamsa(jd)
        assert 24.0 < ayanamsa < 24.5

    def test_ketu_opposite_rahu(self):
        """Ketu should be exactly 180° from Rahu."""
        jd = datetime_to_jd(datetime(2026, 2, 24, 12, 0, 0, tzinfo=timezone.utc))
        rahu = self.engine.get_tropical_longitude(jd, Planet.RAHU)
        ketu = self.engine.get_tropical_longitude(jd, Planet.KETU)
        # Angular distance should be 180°
        diff = abs(ketu - rahu)
        assert abs(diff - 180.0) < 0.001

    def test_all_planets_return_valid_positions(self):
        """All 9 planets should return valid longitudes."""
        jd = datetime_to_jd(datetime(2026, 6, 15, 12, 0, 0, tzinfo=timezone.utc))
        for planet in Planet:
            long = self.engine.get_sidereal_longitude(jd, planet)
            assert 0.0 <= long < 360.0, f"{planet.name} longitude out of range: {long}"

    def test_moon_speed_positive(self):
        """Moon should generally move forward (positive speed)."""
        jd = datetime_to_jd(datetime(2026, 2, 24, 12, 0, 0, tzinfo=timezone.utc))
        speed = self.engine.get_planet_speed(jd, Planet.MOON)
        # Moon speed is typically 12-15 degrees/day
        assert 10.0 < speed < 16.0
