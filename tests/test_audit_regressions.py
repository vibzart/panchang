"""Regression tests for the pre-0.2.14 correctness audit (2026-07-03).

Each test pins a bug found by auditing the core after the 2026 reference
verification (see test_reference_2026.py). Reference dates verified
against Drik Panchang–derived calendars and a locally-run jyotisha +
adyatithi oracle.

AUDIT-1  Duplicate month instances: a lunar month overlapping the year
         boundary appears twice in the year's window list; selection
         took the chronologically first, so Margashirsha festivals
         (Gita Jayanti, Dattatreya Jayanti) vanished in ~half of years.
AUDIT-2  compute_vrat_dates skipped adhika months entirely (30-day vrat
         gap in every adhika year) and had no kshaya-tithi fallback.
AUDIT-3  Shraddha lunar-month classification used "last sankranti
         before date", misplacing ~half of all dates a month early.
AUDIT-4  Vikram Samvat was one year ahead between Jan 1 and Chaitra
         Shukla Pratipada.
AUDIT-5  Ekadashi tithi-vriddhi: when Ekadashi prevails at two
         consecutive sunrises, everyone observes the later day
         (vriddhau uttara) — Vijaya 2027 = Mar 4, not Mar 3.
AUDIT-6  Janmashtami lacked the nishita rule (Krishna born at
         midnight) — 2027 = Aug 24, not Aug 25.
AUDIT-7  compute_sankrantis reported UTC civil dates: Mesha Sankranti
         2026 (Apr 14 03:20 IST) reported as Apr 13 while Baisakhi
         correctly reported Apr 14.
"""

from datetime import date, timedelta

import pytest

from panchang import panchang as pk
from panchang.calendar import (
    compute_ekadashis,
    compute_festivals,
    compute_sankrantis,
    compute_shraddha,
    compute_vrat_dates,
)


@pytest.fixture(scope="module")
def festivals_2027_delhi(delhi):
    return compute_festivals(2027, delhi)


@pytest.fixture(scope="module")
def ekadashis_2027_delhi(delhi):
    return compute_ekadashis(2027, delhi)


@pytest.fixture(scope="module")
def vrats_2026_audit(delhi):
    return compute_vrat_dates(2026, delhi)


class TestYearBoundaryMonths:
    """AUDIT-1: Margashirsha 2026 spills into Jan 2027, duplicating the
    month number in 2027's window list."""

    def test_margashirsha_festivals_present_2027(self, festivals_2027_delhi):
        fests = {f.id: f.date for f in festivals_2027_delhi}
        assert fests.get("gita_jayanti") == date(2027, 12, 9)
        assert fests.get("dattatreya_jayanti") == date(2027, 12, 13)

    def test_gita_jayanti_equals_mokshada(self, festivals_2027_delhi, ekadashis_2027_delhi):
        """Gita Jayanti IS Mokshada Ekadashi day — a year-agnostic
        cross-check between the festival and ekadashi engines."""
        gita = next(f for f in festivals_2027_delhi if f.id == "gita_jayanti")
        mokshada = next(e for e in ekadashis_2027_delhi if e.name == "Mokshada")
        assert gita.date == mokshada.smartha_date


class TestAdhikaVrats:
    """AUDIT-2: month-recurring vrats are observed in adhika months."""

    def test_adhika_jyeshtha_vrats_2026(self, vrats_2026_audit):
        window = [v for v in vrats_2026_audit if date(2026, 5, 17) <= v.date <= date(2026, 6, 15)]
        kinds = {v.vrat_type for v in window}
        assert {"Pradosh Vrat", "Purnima Vrat", "Sankashti Chaturthi", "Amavasya"} <= kinds
        assert all("Adhika Jyeshtha" in v.name for v in window)
        # Drik Panchang: Adhika Jyeshtha Purnima = 2026-05-31,
        # Adhika Amavasya = 2026-06-15.
        by_type = {v.vrat_type: v.date for v in window}
        assert by_type["Purnima Vrat"] == date(2026, 5, 31)
        assert by_type["Amavasya"] == date(2026, 6, 15)

    def test_amavasya_vrat_spacing_invariant(self, vrats_2026_audit):
        """Amavasya occurs every ~29.5 days; a gap > 32 days means a
        month's vrat was silently dropped (adhika skip / kshaya)."""
        amavasyas = sorted(v.date for v in vrats_2026_audit if v.vrat_type == "Amavasya")
        gaps = [(b - a).days for a, b in zip(amavasyas, amavasyas[1:])]
        assert max(gaps) <= 32, f"amavasya vrat gap of {max(gaps)} days"


class TestShraddhaMonth:
    """AUDIT-3: dates between month-start amavasya and the naming
    sankranti were classified into the previous month."""

    def test_death_before_naming_sankranti(self, delhi):
        # 2026-03-25: Chaitra began Mar 19, Mesha Sankranti only Apr 14.
        # The old heuristic returned Phalguna.
        r = compute_shraddha(date(2026, 3, 25), 2027, delhi)
        assert r.lunar_month == 1
        assert r.lunar_month_name == "Chaitra"
        # Chaitra Shukla Saptami 2027
        assert r.shraddha_date == date(2027, 4, 13)


class TestSamvatRollover:
    """AUDIT-4: era year must roll over at Chaitra S1, not Jan 1."""

    @pytest.mark.parametrize(
        "d,vikram,shaka",
        [
            (date(2026, 1, 10), 2082, 1947),  # before Chaitra S1 (Mar 19)
            (date(2026, 3, 25), 2083, 1948),  # after Chaitra S1, before April
            (date(2026, 7, 3), 2083, 1948),
            (date(2026, 12, 25), 2083, 1948),  # Pausha in Dec = after rollover
        ],
    )
    def test_era_years(self, delhi, d, vikram, shaka):
        r = pk.compute(d, delhi)
        assert r.samvat.vikram == vikram, f"{d}: vikram {r.samvat.vikram}"
        assert r.samvat.shaka == shaka, f"{d}: shaka {r.samvat.shaka}"


class TestEkadashi2027:
    """AUDIT-5 + regular-year coverage (jyotisha-verified)."""

    def test_vijaya_tithi_vriddhi(self, ekadashis_2027_delhi):
        """Ekadashi tithi spans two sunrises (Mar 3 04:44 → Mar 4 07:24)
        → vriddhau uttara: observed Mar 4 (Drik Panchang + jyotisha)."""
        eks = {e.name: e for e in ekadashis_2027_delhi}
        assert eks["Vijaya"].smartha_date == date(2027, 3, 4)
        assert eks["Vijaya"].vaishnava_date == date(2027, 3, 4)

    def test_2027_count_and_spacing(self, ekadashis_2027_delhi):
        assert len(ekadashis_2027_delhi) == 25  # Saphala falls twice (Jan+Dec)
        dates = sorted(e.smartha_date for e in ekadashis_2027_delhi)
        gaps = [(b - a).days for a, b in zip(dates, dates[1:])]
        assert max(gaps) <= 17

    def test_2027_splits(self, ekadashis_2027_delhi):
        """Smarta/Vaishnava splits verified against jyotisha."""
        eks = {e.name: e for e in ekadashis_2027_delhi}
        assert eks["Kamika"].smartha_date == date(2027, 7, 29)
        assert eks["Kamika"].vaishnava_date == date(2027, 7, 30)
        assert eks["Rama"].smartha_date == date(2027, 10, 25)
        assert eks["Rama"].vaishnava_date == date(2027, 10, 26)


# id -> date, verified against jyotisha (Delhi) and Drik Panchang–derived
# published calendars for 2027.
VERIFIED_FESTIVALS_2027 = {
    "maha_shivaratri": date(2027, 3, 6),
    "holika_dahan": date(2027, 3, 21),
    "ram_navami": date(2027, 4, 15),
    "raksha_bandhan": date(2027, 8, 17),
    "janmashtami": date(2027, 8, 24),  # AUDIT-6: nishita rule
    "ganesh_chaturthi": date(2027, 9, 4),
    "dussehra": date(2027, 10, 9),
    "karva_chauth": date(2027, 10, 18),
    "dhanteras": date(2027, 10, 27),
    "narak_chaturdashi": date(2027, 10, 28),
    "diwali": date(2027, 10, 29),
    "govardhan_puja": date(2027, 10, 30),
    "bhai_dooj": date(2027, 10, 31),
}


class TestFestivals2027:
    def test_verified_dates(self, festivals_2027_delhi):
        fests = {f.id: f for f in festivals_2027_delhi}
        for fid, expected in VERIFIED_FESTIVALS_2027.items():
            assert fid in fests, f"{fid} missing from 2027 output"
            got = fests[fid].date
            assert got == expected, (
                f"{fid}: expected {expected}, got {got}. Reasoning: {fests[fid].reasoning}"
            )

    def test_diwali_cluster_ordering_2027(self, festivals_2027_delhi):
        fests = {f.id: f for f in festivals_2027_delhi}
        order = [
            "karva_chauth",
            "ahoi_ashtami",
            "dhanteras",
            "narak_chaturdashi",
            "diwali",
            "govardhan_puja",
            "bhai_dooj",
        ]
        dates = [fests[fid].date for fid in order]
        assert dates == sorted(dates), dict(zip(order, dates))
        assert dates[-1] - dates[0] < timedelta(days=15)


class TestSankrantiLocalDates:
    """AUDIT-7: sankranti civil dates must be local, not UTC."""

    def test_mesha_sankranti_2026_ist(self, delhi):
        sank = compute_sankrantis(2026, delhi)
        mesha = next(s for s in sank if s.name == "Mesha Sankranti")
        # Apr 13 21:50 UTC = Apr 14 03:20 IST — must agree with Baisakhi.
        assert mesha.date == date(2026, 4, 14)

    def test_sankranti_agrees_with_baisakhi(self, delhi, festivals_2026_delhi):
        sank = {s.name: s.date for s in compute_sankrantis(2026, delhi)}
        fests = {f.id: f.date for f in festivals_2026_delhi}
        assert sank["Mesha Sankranti"] == fests["baisakhi"]
        assert sank["Makar Sankranti"] == fests["makar_sankranti"]
