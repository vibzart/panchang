"""Smoke tests for the Lagna (sidereal ascendant) computation.

These tests run from Python through the PyO3 boundary, complementing
the deeper Rust-side reference tests in
`crates/panchang-core/tests/lagna_reference.rs`.
"""

from datetime import datetime, timezone

import pytest

from panchang.core.ephemeris import datetime_to_jd
from panchang.core.lagna import (
    HouseSystem,
    LagnaInfo,
    LagnaWindow,
    compute_lagna,
    compute_lagna_windows,
)


def _india_independence_jd() -> float:
    """1947-08-15 00:00 IST = 1947-08-14 18:30 UT."""
    return datetime_to_jd(datetime(1947, 8, 14, 18, 30, tzinfo=timezone.utc))


class TestComputeLagnaShape:
    """The PyO3 wrapper returns a well-formed dataclass."""

    def test_returns_lagna_info_with_all_fields(self):
        info = compute_lagna(_india_independence_jd(), 28.6139, 77.2090)

        assert isinstance(info, LagnaInfo)
        assert isinstance(info.ascendant_longitude, float)
        assert isinstance(info.rashi, int)
        assert isinstance(info.rashi_name, str)
        assert isinstance(info.degree_in_rashi, float)
        assert isinstance(info.mc_longitude, float)
        assert isinstance(info.bhava_cusps, tuple)
        assert len(info.bhava_cusps) == 12
        assert info.house_system == HouseSystem.PLACIDUS

    def test_ascendant_in_valid_range(self):
        info = compute_lagna(_india_independence_jd(), 28.6139, 77.2090)
        assert 0.0 <= info.ascendant_longitude < 360.0
        assert 0 <= info.rashi <= 11
        assert 0.0 <= info.degree_in_rashi < 30.0

    def test_rashi_and_degree_reconstruct_ascendant(self):
        info = compute_lagna(_india_independence_jd(), 28.6139, 77.2090)
        reconstructed = info.rashi * 30.0 + info.degree_in_rashi
        assert abs(reconstructed - info.ascendant_longitude) < 1e-9


class TestIndiaIndependenceLagna:
    """1947-08-15 00:00 IST, New Delhi — Vrishabha Lagna in jyotish lit."""

    @pytest.fixture
    def info(self):
        return compute_lagna(_india_independence_jd(), 28.6139, 77.2090)

    def test_lagna_rashi_is_vrishabha(self, info):
        assert info.rashi == 1
        assert info.rashi_name == "Vrishabha"

    def test_lagna_degree_matches_drik_panchang(self, info):
        # Drik Panchang: Vrishabha 7°44'. Allow ±1° drift.
        assert abs(info.degree_in_rashi - 7.73) < 1.0


class TestHouseSystems:
    """All four house systems return a valid result with the same Lagna."""

    @pytest.mark.parametrize(
        "system",
        [
            HouseSystem.PLACIDUS,
            HouseSystem.WHOLE_SIGN,
            HouseSystem.EQUAL,
            HouseSystem.PORPHYRY,
        ],
    )
    def test_each_system_returns_valid_cusps(self, system):
        info = compute_lagna(_india_independence_jd(), 28.6139, 77.2090, system)
        assert info.house_system == system
        for cusp in info.bhava_cusps:
            assert 0.0 <= cusp < 360.0

    def test_string_system_argument_accepted(self):
        info = compute_lagna(_india_independence_jd(), 28.6139, 77.2090, "W")
        assert info.house_system == HouseSystem.WHOLE_SIGN

    def test_ascendant_invariant_across_systems(self):
        jd = _india_independence_jd()
        infos = [
            compute_lagna(jd, 28.6139, 77.2090, s)
            for s in [
                HouseSystem.PLACIDUS,
                HouseSystem.WHOLE_SIGN,
                HouseSystem.EQUAL,
                HouseSystem.PORPHYRY,
            ]
        ]
        first = infos[0].ascendant_longitude
        for other in infos[1:]:
            assert abs(other.ascendant_longitude - first) < 1e-9

    def test_invalid_system_raises(self):
        with pytest.raises(ValueError):
            compute_lagna(_india_independence_jd(), 28.6139, 77.2090, "Z")


class TestAyanamsaSurfaced:
    """The ``ayanamsa`` field is populated and matches the Lahiri value."""

    def test_ayanamsa_is_populated(self):
        info = compute_lagna(_india_independence_jd(), 28.6139, 77.2090)
        # Lahiri ayanamsa at Aug 1947 ≈ 23.10°.
        assert 22.5 < info.ayanamsa < 24.5

    def test_ayanamsa_drifts_correctly_across_decades(self):
        # ~50.27"/year ≈ 0.014°/year. Independence (1947) → J2000 = 53y ≈ 0.74°.
        info_1947 = compute_lagna(_india_independence_jd(), 28.6139, 77.2090)
        info_2000 = compute_lagna(
            datetime_to_jd(datetime(2000, 1, 1, 12, 0, tzinfo=timezone.utc)),
            28.6139,
            77.2090,
        )
        delta = info_2000.ayanamsa - info_1947.ayanamsa
        assert 0.6 < delta < 0.95


class TestComputeLagnaWindows:
    """Birth-time uncertainty UX — 12 lagna-rising windows for a day."""

    def test_returns_list_of_windows(self):
        jd_start = datetime_to_jd(datetime(1990, 6, 15, 0, 0, tzinfo=timezone.utc))
        windows = compute_lagna_windows(jd_start, 19.0760, 72.8777)
        assert all(isinstance(w, LagnaWindow) for w in windows)
        assert 12 <= len(windows) <= 13

    def test_windows_are_continuous_and_span_24h(self):
        jd_start = datetime_to_jd(datetime(1990, 6, 15, 0, 0, tzinfo=timezone.utc))
        windows = compute_lagna_windows(jd_start, 19.0760, 72.8777)
        total = sum(w.end_jd - w.start_jd for w in windows)
        assert abs(total - 1.0) < 1e-6
        for prev, curr in zip(windows, windows[1:]):
            assert abs(prev.end_jd - curr.start_jd) < 1e-9

    def test_each_window_has_valid_rashi(self):
        jd_start = datetime_to_jd(datetime(1990, 6, 15, 0, 0, tzinfo=timezone.utc))
        windows = compute_lagna_windows(jd_start, 19.0760, 72.8777)
        for w in windows:
            assert 0 <= w.rashi <= 11
            assert isinstance(w.rashi_name, str) and len(w.rashi_name) > 0


class TestNoStatePollutionAcrossCalls:
    """Regression test: calling `compute_lagna` repeatedly with different
    JDs must NOT cause ayanamsa drift (Swiss Ephemeris caches the sidereal
    epoch on first call; we reset it on every call to defeat that)."""

    def test_back_to_back_calls_are_stable(self):
        jd_independence = _india_independence_jd()
        jd_j2000 = datetime_to_jd(datetime(2000, 1, 1, 12, 0, tzinfo=timezone.utc))

        first = compute_lagna(jd_independence, 28.6139, 77.2090).ascendant_longitude
        _ = compute_lagna(jd_j2000, 51.4779, 0.0)
        again = compute_lagna(jd_independence, 28.6139, 77.2090).ascendant_longitude

        # Without the sid-mode reset in compute(), this drifts by ~0.88°.
        assert abs(first - again) < 1e-6
