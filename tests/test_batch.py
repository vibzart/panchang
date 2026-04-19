"""Tests for batch computation.

Year-long batch fixtures (``batch_2026_delhi``, ``batch_2024_delhi``) are
session-scoped in conftest.py so the ~6s Swiss-Ephemeris pass runs once.
"""

from datetime import date

from panchang import batch, panchang


class TestBatchYear:
    def test_day_count_2026(self, batch_2026_delhi):
        assert len(batch_2026_delhi) == 365

    def test_day_count_leap_year(self, batch_2024_delhi):
        assert len(batch_2024_delhi) == 366

    def test_first_and_last_day(self, batch_2026_delhi):
        assert batch_2026_delhi[0].date == date(2026, 1, 1)
        assert batch_2026_delhi[-1].date == date(2026, 12, 31)

    def test_panchang_valid(self, batch_2026_delhi):
        for d in batch_2026_delhi:
            assert 1 <= d.tithi.number <= 30
            assert 1 <= d.nakshatra.number <= 27
            assert 1 <= d.yoga.number <= 27
            assert 1 <= d.karana.number <= 11


class TestBatchRange:
    def test_range_count(self, delhi):
        days = batch.compute_range(date(2026, 3, 1), date(2026, 3, 31), delhi)
        assert len(days) == 31

    def test_cross_month_range(self, delhi):
        days = batch.compute_range(date(2026, 2, 25), date(2026, 3, 5), delhi)
        assert len(days) == 9

    def test_spot_check_matches_individual(self, delhi):
        """Batch computation should match individual panchang.compute()."""
        test_date = date(2026, 3, 1)
        batch_days = batch.compute_range(test_date, test_date, delhi)
        assert len(batch_days) == 1
        batch_day = batch_days[0]

        individual = panchang.compute(test_date, delhi, include_muhurat=False)

        assert batch_day.tithi.number == individual.tithi.number
        assert batch_day.nakshatra.number == individual.nakshatra.number
        assert batch_day.yoga.number == individual.yoga.number
        assert batch_day.vara.english == individual.vara.english
