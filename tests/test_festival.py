"""Tests for festival, Ekadashi, and Vrat date computation."""

from panchang import Location, calendar

DELHI = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")


class TestFestivals:
    def test_festival_count(self):
        festivals = calendar.compute_festivals(2026, DELHI)
        assert len(festivals) >= 14, f"Got {len(festivals)} festivals"

    def test_diwali_date(self):
        festivals = calendar.compute_festivals(2026, DELHI)
        diwali = next(f for f in festivals if f.id == "diwali")
        assert diwali.date.month == 11
        assert 8 <= diwali.date.day <= 10

    def test_holi_date(self):
        festivals = calendar.compute_festivals(2026, DELHI)
        holi = next(f for f in festivals if f.id == "holi")
        assert holi.date.month == 3
        assert 2 <= holi.date.day <= 4

    def test_makar_sankranti_date(self):
        festivals = calendar.compute_festivals(2026, DELHI)
        ms = next(f for f in festivals if f.id == "makar_sankranti")
        assert ms.date.month == 1
        assert 13 <= ms.date.day <= 15

    def test_festivals_sorted_chronologically(self):
        festivals = calendar.compute_festivals(2026, DELHI)
        for i in range(1, len(festivals)):
            assert festivals[i].date >= festivals[i - 1].date

    def test_all_have_reasoning(self):
        festivals = calendar.compute_festivals(2026, DELHI)
        for f in festivals:
            assert f.reasoning, f"Missing reasoning for {f.name}"

    def test_all_dates_in_year(self):
        festivals = calendar.compute_festivals(2026, DELHI)
        for f in festivals:
            assert f.date.year == 2026, f"{f.name} date year: {f.date.year}"

    def test_janmashtami_date(self):
        festivals = calendar.compute_festivals(2026, DELHI)
        jk = next(f for f in festivals if f.id == "janmashtami")
        assert jk.date.month in (8, 9), f"Janmashtami month: {jk.date.month}"


class TestEkadashis:
    def test_ekadashi_count(self):
        ekadashis = calendar.compute_ekadashis(2026, DELHI)
        assert 20 <= len(ekadashis) <= 26, f"Got {len(ekadashis)} Ekadashis"

    def test_both_pakshas(self):
        ekadashis = calendar.compute_ekadashis(2026, DELHI)
        shukla = [e for e in ekadashis if e.paksha == "Shukla"]
        krishna = [e for e in ekadashis if e.paksha == "Krishna"]
        assert len(shukla) >= 10
        assert len(krishna) >= 10

    def test_vaishnava_date_same_or_later(self):
        ekadashis = calendar.compute_ekadashis(2026, DELHI)
        for ek in ekadashis:
            assert ek.vaishnava_date >= ek.smartha_date, (
                f"{ek.name}: vaishnava={ek.vaishnava_date} < smartha={ek.smartha_date}"
            )

    def test_all_have_names(self):
        ekadashis = calendar.compute_ekadashis(2026, DELHI)
        for ek in ekadashis:
            assert ek.name, "Ekadashi missing name"

    def test_unique_names(self):
        ekadashis = calendar.compute_ekadashis(2026, DELHI)
        names = [ek.name for ek in ekadashis]
        assert len(names) == len(set(names)), "Duplicate Ekadashi names"


class TestVratDates:
    def test_vrat_count(self):
        vrats = calendar.compute_vrat_dates(2026, DELHI)
        assert 45 <= len(vrats) <= 65, f"Got {len(vrats)} Vrat dates"

    def test_all_dates_in_year(self):
        vrats = calendar.compute_vrat_dates(2026, DELHI)
        for v in vrats:
            assert v.date.year == 2026

    def test_vrat_types_present(self):
        vrats = calendar.compute_vrat_dates(2026, DELHI)
        types = {v.vrat_type for v in vrats}
        assert "Pradosh Vrat" in types
        assert "Amavasya" in types
        assert "Purnima Vrat" in types
        assert "Sankashti Chaturthi" in types
