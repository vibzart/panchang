"""Tests for festival, Ekadashi, and Vrat date computation.

All tests reuse the session-scoped fixtures ``festivals_2026_delhi`` /
``ekadashis_2026_delhi`` / ``vrats_2026_delhi`` so the expensive year-long
computations run once per test session instead of once per test.
"""

import datetime


class TestFestivals:
    def test_festival_count(self, festivals_2026_delhi):
        assert len(festivals_2026_delhi) >= 14, f"Got {len(festivals_2026_delhi)} festivals"

    def test_diwali_date(self, festivals_2026_delhi):
        diwali = next(f for f in festivals_2026_delhi if f.id == "diwali")
        assert diwali.date.month == 11
        assert 8 <= diwali.date.day <= 10

    def test_holi_date(self, festivals_2026_delhi):
        holi = next(f for f in festivals_2026_delhi if f.id == "holi")
        assert holi.date.month == 3
        assert 2 <= holi.date.day <= 4

    def test_makar_sankranti_date(self, festivals_2026_delhi):
        ms = next(f for f in festivals_2026_delhi if f.id == "makar_sankranti")
        assert ms.date.month == 1
        assert 13 <= ms.date.day <= 15

    def test_festivals_sorted_chronologically(self, festivals_2026_delhi):
        for i in range(1, len(festivals_2026_delhi)):
            assert festivals_2026_delhi[i].date >= festivals_2026_delhi[i - 1].date

    def test_all_have_reasoning(self, festivals_2026_delhi):
        for f in festivals_2026_delhi:
            assert f.reasoning, f"Missing reasoning for {f.name}"

    def test_all_dates_in_year(self, festivals_2026_delhi):
        for f in festivals_2026_delhi:
            assert f.date.year == 2026, f"{f.name} date year: {f.date.year}"

    def test_janmashtami_date(self, festivals_2026_delhi):
        jk = next(f for f in festivals_2026_delhi if f.id == "janmashtami")
        # Janmashtami = Shravana Krishna Ashtami (Amant month 5).
        # Per shloka: मासि तु श्रावणेऽष्टम्यां (Brahma Vaivarta Purana).
        # Web sources confirm Sep 4, 2026.
        assert jk.date.month in (8, 9), f"Janmashtami month: {jk.date.month}"

    def test_akshaya_tritiya_2026_vyapti_aparahna(self, festivals_2026_delhi):
        """AT 2026 with YAML-configured vyapti/aparahna rule → April 19.

        This matches Drik Panchang and the popular dana-pradhan convention.
        The paraviddha alternate (April 20 — strict udayatithi) must be
        surfaced so callers can present both to users.
        """
        at = next(f for f in festivals_2026_delhi if f.id == "akshaya_tritiya")

        assert at.date.isoformat() == "2026-04-19", (
            f"AT 2026 should be April 19 (vyapti/aparahna); got {at.date}. "
            f"Reasoning: {at.reasoning}"
        )
        assert at.priority_applied == "vyapti"
        assert at.kaala_applied == "aparahna"

        # Alternate must be April 20 paraviddha (udayatithi).
        assert at.alternate is not None, "vyapti result must expose paraviddha alternate"
        assert at.alternate.date.isoformat() == "2026-04-20"
        assert at.alternate.priority == "paraviddha"


class TestEkadashis:
    def test_ekadashi_count(self, ekadashis_2026_delhi):
        assert 20 <= len(ekadashis_2026_delhi) <= 26, (
            f"Got {len(ekadashis_2026_delhi)} Ekadashis"
        )

    def test_both_pakshas(self, ekadashis_2026_delhi):
        shukla = [e for e in ekadashis_2026_delhi if e.paksha == "Shukla"]
        krishna = [e for e in ekadashis_2026_delhi if e.paksha == "Krishna"]
        assert len(shukla) >= 10
        assert len(krishna) >= 10

    def test_vaishnava_date_same_or_later(self, ekadashis_2026_delhi):
        for ek in ekadashis_2026_delhi:
            assert ek.vaishnava_date >= ek.smartha_date, (
                f"{ek.name}: vaishnava={ek.vaishnava_date} < smartha={ek.smartha_date}"
            )

    def test_all_have_names(self, ekadashis_2026_delhi):
        for ek in ekadashis_2026_delhi:
            assert ek.name, "Ekadashi missing name"

    def test_unique_names(self, ekadashis_2026_delhi):
        names = [ek.name for ek in ekadashis_2026_delhi]
        assert len(names) == len(set(names)), "Duplicate Ekadashi names"

    def test_prabodhini_2026_present(self, ekadashis_2026_delhi):
        """Regression: Prabodhini Ekadashi 2026 falls on Kartik Shukla 11,
        which is *kshaya* (tithi 11 never spans sunrise in Kartik 2026).
        Smarta observance is the preceding Dashami day (Nov 20, 2026).
        Prior sankranti-anchored search silently dropped this entry."""
        prabodhini = [e for e in ekadashis_2026_delhi if e.name == "Prabodhini"]
        assert prabodhini, "Prabodhini Ekadashi missing from 2026 list"
        assert prabodhini[0].smartha_date == datetime.date(2026, 11, 20)
        assert prabodhini[0].lunar_month == 8  # Kartik
        assert prabodhini[0].paksha == "Shukla"

    def test_lunar_month_labels_are_correct(self, ekadashis_2026_delhi):
        """Regression: the old sankranti-anchored search sometimes labeled
        Ekadashis with the wrong lunar month (e.g., Jan 29 2026 was returned
        as 'Pausha Putrada/Pausha' when it is actually 'Jaya/Magha')."""
        by_date = {e.smartha_date: e for e in ekadashis_2026_delhi}
        # Jan 29 2026 is Magha Shukla Ekadashi (Jaya), not Pausha.
        if datetime.date(2026, 1, 29) in by_date:
            e = by_date[datetime.date(2026, 1, 29)]
            assert e.lunar_month_name == "Magha", f"Expected Magha, got {e.lunar_month_name}"
            assert e.paksha == "Shukla"


class TestVratDates:
    def test_vrat_count(self, vrats_2026_delhi):
        assert 45 <= len(vrats_2026_delhi) <= 65, f"Got {len(vrats_2026_delhi)} Vrat dates"

    def test_all_dates_in_year(self, vrats_2026_delhi):
        for v in vrats_2026_delhi:
            assert v.date.year == 2026

    def test_vrat_types_present(self, vrats_2026_delhi):
        types = {v.vrat_type for v in vrats_2026_delhi}
        assert "Pradosh Vrat" in types
        assert "Amavasya" in types
        assert "Purnima Vrat" in types
        assert "Sankashti Chaturthi" in types
