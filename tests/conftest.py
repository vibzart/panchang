"""Shared test fixtures for Panchang tests."""

import pytest

from panchang.types import Location


@pytest.fixture
def delhi():
    """New Delhi, Bhārat."""
    return Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")


@pytest.fixture
def mumbai():
    """Mumbai, Bhārat."""
    return Location(lat=19.0760, lng=72.8777, tz="Asia/Kolkata")


@pytest.fixture
def chennai():
    """Chennai, Bhārat."""
    return Location(lat=13.0827, lng=80.2707, tz="Asia/Kolkata")


@pytest.fixture
def new_york():
    """New York, USA — for testing timezone handling."""
    return Location(lat=40.7128, lng=-74.0060, tz="America/New_York")


@pytest.fixture
def london():
    """London, UK — for testing GMT timezone."""
    return Location(lat=51.5074, lng=-0.1278, tz="Europe/London")
