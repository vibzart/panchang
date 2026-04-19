"""Shared test fixtures for Panchang tests.

Expensive, read-only computations (a full year of festivals, a full year of
batch panchang) are cached as session-scoped fixtures so they run once per
test session instead of once per test. This cuts the suite from ~5 minutes
to ~1 minute without reducing coverage.
"""

from datetime import date

import pytest

from panchang import calendar
from panchang.batch import compute_range
from panchang.types import Location

# ─── Location fixtures (session-scoped for consistency) ────────────────────


@pytest.fixture(scope="session")
def delhi():
    """New Delhi, Bhārat."""
    return Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")


@pytest.fixture(scope="session")
def mumbai():
    """Mumbai, Bhārat."""
    return Location(lat=19.0760, lng=72.8777, tz="Asia/Kolkata")


@pytest.fixture(scope="session")
def chennai():
    """Chennai, Bhārat."""
    return Location(lat=13.0827, lng=80.2707, tz="Asia/Kolkata")


@pytest.fixture(scope="session")
def new_york():
    """New York, USA — for testing timezone handling."""
    return Location(lat=40.7128, lng=-74.0060, tz="America/New_York")


@pytest.fixture(scope="session")
def london():
    """London, UK — for testing GMT timezone."""
    return Location(lat=51.5074, lng=-0.1278, tz="Europe/London")


# ─── Expensive-to-compute fixtures (cached across all tests) ────────────────


@pytest.fixture(scope="session")
def festivals_2026_delhi(delhi):
    """All 2026 festivals for Delhi — ~23s to compute, reused across tests."""
    return calendar.compute_festivals(2026, delhi)


@pytest.fixture(scope="session")
def ekadashis_2026_delhi(delhi):
    """All 2026 Ekadashis for Delhi."""
    return calendar.compute_ekadashis(2026, delhi)


@pytest.fixture(scope="session")
def vrats_2026_delhi(delhi):
    """All 2026 Vrat dates for Delhi."""
    return calendar.compute_vrat_dates(2026, delhi)


@pytest.fixture(scope="session")
def batch_2026_delhi(delhi):
    """Full-year 2026 batch panchang for Delhi — ~6s to compute, reused."""
    return compute_range(date(2026, 1, 1), date(2026, 12, 31), delhi)


@pytest.fixture(scope="session")
def batch_2024_delhi(delhi):
    """Full-year 2024 (leap year) batch panchang for Delhi."""
    return compute_range(date(2024, 1, 1), date(2024, 12, 31), delhi)
