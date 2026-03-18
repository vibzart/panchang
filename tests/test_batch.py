"""Tests for batch computation."""

from datetime import date

from panchang import Location, batch, panchang

DELHI = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")


class TestBatchYear:
    def test_day_count_2026(self):
        days = batch.compute_year(2026, DELHI)
        assert len(days) == 365

    def test_day_count_leap_year(self):
        days = batch.compute_year(2024, DELHI)
        assert len(days) == 366

    def test_first_and_last_day(self):
        days = batch.compute_year(2026, DELHI)
        assert days[0].date == date(2026, 1, 1)
        assert days[-1].date == date(2026, 12, 31)

    def test_panchang_valid(self):
        days = batch.compute_year(2026, DELHI)
        for d in days:
            assert 1 <= d.tithi.number <= 30
            assert 1 <= d.nakshatra.number <= 27
            assert 1 <= d.yoga.number <= 27
            assert 1 <= d.karana.number <= 11


class TestBatchRange:
    def test_range_count(self):
        days = batch.compute_range(date(2026, 3, 1), date(2026, 3, 31), DELHI)
        assert len(days) == 31

    def test_cross_month_range(self):
        days = batch.compute_range(date(2026, 2, 25), date(2026, 3, 5), DELHI)
        assert len(days) == 9

    def test_spot_check_matches_individual(self):
        """Batch computation should match individual panchang.compute()."""
        test_date = date(2026, 3, 1)
        batch_days = batch.compute_range(test_date, test_date, DELHI)
        assert len(batch_days) == 1
        batch_day = batch_days[0]

        individual = panchang.compute(test_date, DELHI, include_muhurat=False)

        assert batch_day.tithi.number == individual.tithi.number
        assert batch_day.nakshatra.number == individual.nakshatra.number
        assert batch_day.yoga.number == individual.yoga.number
        assert batch_day.vara.english == individual.vara.english
