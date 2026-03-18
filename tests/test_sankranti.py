"""Tests for Sankranti computation."""

from panchang import Location, calendar

DELHI = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")


class TestSankrantiComputation:
    def test_twelve_sankrantis(self):
        results = calendar.compute_sankrantis(2026, DELHI)
        assert len(results) == 12

    def test_makar_sankranti_date(self):
        results = calendar.compute_sankrantis(2026, DELHI)
        makar = results[0]
        assert makar.name == "Makar Sankranti"
        assert makar.rashi == "Makara"
        assert makar.date.month == 1
        assert 13 <= makar.date.day <= 15

    def test_sankrantis_chronological(self):
        results = calendar.compute_sankrantis(2026, DELHI)
        for i in range(1, len(results)):
            assert results[i].date >= results[i - 1].date

    def test_all_rashi_names_valid(self):
        valid = {
            "Mesha",
            "Vrishabha",
            "Mithuna",
            "Karka",
            "Simha",
            "Kanya",
            "Tula",
            "Vrischika",
            "Dhanu",
            "Makara",
            "Kumbha",
            "Meena",
        }
        results = calendar.compute_sankrantis(2026, DELHI)
        for s in results:
            assert s.rashi in valid, f"Invalid rashi: {s.rashi}"

    def test_different_year(self):
        results = calendar.compute_sankrantis(2024, DELHI)
        assert len(results) == 12
