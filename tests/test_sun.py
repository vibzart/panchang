"""Tests for sunrise/sunset calculations."""

from datetime import datetime, timezone

from panchang.core.sun import compute_sun_data, compute_sunrise, compute_sunset


class TestSunrise:
    """Sunrise calculation tests."""

    def test_delhi_sunrise_february(self, delhi):
        """Delhi sunrise in February should be around 06:45-07:00 IST."""
        dt = datetime(2026, 2, 24)
        sunrise = compute_sunrise(dt, delhi)

        assert sunrise.tzinfo is not None
        assert sunrise.hour == 6
        assert 30 <= sunrise.minute <= 59

    def test_mumbai_sunrise_differs_from_delhi(self, mumbai, delhi):
        """Mumbai and Delhi should have different sunrise times."""
        dt = datetime(2026, 2, 24)
        mumbai_rise = compute_sunrise(dt, mumbai).astimezone(timezone.utc)
        delhi_rise = compute_sunrise(dt, delhi).astimezone(timezone.utc)
        # Mumbai is ~4.4° west but ~9.5° south of Delhi.
        # In February, latitude effect can offset longitude effect.
        diff_minutes = abs((mumbai_rise - delhi_rise).total_seconds()) / 60
        assert 0 < diff_minutes < 30

    def test_summer_sunrise_earlier(self, delhi):
        """Summer sunrise should be earlier than winter sunrise."""
        winter_rise = compute_sunrise(datetime(2026, 1, 15), delhi)
        summer_rise = compute_sunrise(datetime(2026, 6, 15), delhi)
        assert summer_rise.hour < winter_rise.hour or (
            summer_rise.hour == winter_rise.hour and summer_rise.minute < winter_rise.minute
        )


class TestSunset:
    """Sunset calculation tests."""

    def test_delhi_sunset_february(self, delhi):
        """Delhi sunset in February should be around 18:00-18:20 IST."""
        dt = datetime(2026, 2, 24)
        sunset = compute_sunset(dt, delhi)

        assert sunset.tzinfo is not None
        assert sunset.hour == 18
        assert 0 <= sunset.minute <= 30


class TestSunData:
    """Complete sun data tests."""

    def test_day_duration_reasonable(self, delhi):
        """Day duration should be between 10 and 14 hours for Bhāratīya locations."""
        dt = datetime(2026, 2, 24)
        data = compute_sun_data(dt, delhi)
        assert 10.0 < data.day_duration_hours < 14.0

    def test_sunrise_before_sunset(self, delhi):
        """Sunrise must always be before sunset."""
        dt = datetime(2026, 2, 24)
        data = compute_sun_data(dt, delhi)
        assert data.sunrise < data.sunset

    def test_equinox_day_near_12_hours(self, delhi):
        """Near equinox (March 20), day duration should be close to 12 hours."""
        dt = datetime(2026, 3, 20)
        data = compute_sun_data(dt, delhi)
        assert 11.5 < data.day_duration_hours < 12.5

    def test_new_york_timezone(self, new_york):
        """Sunrise in New York should be in EST/EDT timezone."""
        dt = datetime(2026, 2, 24)
        data = compute_sun_data(dt, new_york)
        assert data.sunrise.tzinfo is not None
        # February = EST (UTC-5). Sunrise around 06:30-07:00 EST
        assert 6 <= data.sunrise.hour <= 7
