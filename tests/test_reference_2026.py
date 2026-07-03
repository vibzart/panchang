"""2026 reference-data regression tests (Delhi).

Every date here was cross-verified on 2026-07-03 against at least two
independent sources:

- Drik Panchang–derived published calendars (drikpanchang.com and sites
  that republish its data)
- jyotisha v0.1.9 (github.com/jyotisham/jyotisha) computed locally for
  Delhi with the adyatithi rule corpus

All dates are pinned as hard assertions. Tests tagged with a BUG number
are regressions for bugs found (and fixed) during that verification:

BUG-1  Adhik-maas ekadashis were skipped entirely: Padmini (2026-05-27)
       and Parama (2026-06-11) were absent, leaving an impossible 42-day
       ekadashi gap. Fixing Padmini also required the Dharmasindhu
       two-sunrise vedha rule in resolve_ekadashi.
BUG-2  The Diwali-cluster festivals (Karva Chauth, Ahoi Ashtami,
       Dhanteras, Narak Chaturdashi, Diwali) were defined with the
       purnimant month label (Kartik, 8) in the amant-numbered YAML,
       landing ~1 month late. Fixing Diwali also exposed a month-window
       boundary leak in find_tithi_at_sunrise_in_range (the previous
       month's Amavasya matched at the first civil day's sunrise).
BUG-3  Kaala-based observance rules were not configured: Maha Shivaratri
       (nishita), Diwali/Dhanteras (pradosha), Ram Navami and Ganesh
       Chaturthi (madhyahna) each resolved one day late.
BUG-4  Location silently ignored unknown kwargs: Location(lat=..,
       lng=.., timezone="Asia/Kolkata") dropped "timezone" (the field is
       ``tz``) and computed for the default UTC, shifting dates by a
       day with no error. Location now uses extra="forbid".
BUG-5  Dussehra was one day late (2026-10-21 vs 10-20): when Dashami has
       aparahna-vyapti on BOTH days, Vijayadashami takes the EARLIER day
       — now expressed via `vyapti_tie: purva` in festivals.yaml.
"""

from datetime import date, timedelta
from zoneinfo import ZoneInfo

import pytest

from panchang.calendar import compute_lunar_months
from panchang.calendar.lunar_month import CalendarSystem

IST = ZoneInfo("Asia/Kolkata")


def _by_id(festivals):
    return {f.id: f for f in festivals}


def _by_name(ekadashis):
    return {e.name: e for e in ekadashis}


# ─── Adhik Maas 2026 ─────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def amant_months(delhi):
    return compute_lunar_months(2026, delhi, CalendarSystem.AMANT)


class TestAdhikMaas2026:
    """2026 has Adhika Jyeshtha: May 17 – June 15 (Drik Panchang,
    confirmed by jyotisha's Padmini/Parama ekadashi placement)."""

    def test_exactly_one_adhik_month(self, amant_months):
        adhik = [m for m in amant_months if m.is_adhik]
        assert len(adhik) == 1

    def test_adhik_month_is_jyeshtha(self, amant_months):
        adhik = next(m for m in amant_months if m.is_adhik)
        assert adhik.name == "Jyeshtha"

    def test_adhik_month_boundaries(self, amant_months):
        """First civil day May 17 IST, ends on amavasya day June 15 IST."""
        adhik = next(m for m in amant_months if m.is_adhik)
        start_ist = adhik.start.astimezone(IST)
        end_ist = adhik.end.astimezone(IST)
        assert start_ist.date() == date(2026, 5, 17)
        assert end_ist.date() == date(2026, 6, 15)

    def test_nija_jyeshtha_follows_adhika(self, amant_months):
        names = [(m.name, m.is_adhik) for m in amant_months]
        i = names.index(("Jyeshtha", True))
        assert names[i + 1] == ("Jyeshtha", False)

    def test_no_kshaya_month_2026(self, amant_months):
        assert not any(m.is_kshaya for m in amant_months)

    def test_no_adhik_month_2025(self, delhi):
        months = compute_lunar_months(2025, delhi, CalendarSystem.AMANT)
        assert not any(m.is_adhik for m in months)


# ─── Ekadashi 2026 ───────────────────────────────────────────────────────────

# name -> smartha date. All agree across panchang, jyotisha and published
# Drik Panchang calendars.
VERIFIED_EKADASHI_SMARTHA = {
    "Shattila": date(2026, 1, 14),
    "Jaya": date(2026, 1, 29),
    "Vijaya": date(2026, 2, 13),
    "Amalaki": date(2026, 2, 27),
    "Papamochani": date(2026, 3, 15),
    "Kamada": date(2026, 3, 29),
    "Varuthini": date(2026, 4, 13),
    "Mohini": date(2026, 4, 27),
    "Apara": date(2026, 5, 13),
    "Nirjala": date(2026, 6, 25),
    "Yogini": date(2026, 7, 10),
    "Devshayani": date(2026, 7, 25),
    "Kamika": date(2026, 8, 9),
    "Shravana Putrada": date(2026, 8, 23),
    "Aja": date(2026, 9, 7),
    "Parsva": date(2026, 9, 22),
    "Indira": date(2026, 10, 6),
    "Papankusha": date(2026, 10, 22),
    "Rama": date(2026, 11, 5),
    "Prabodhini": date(2026, 11, 20),
    "Utpanna": date(2026, 12, 4),
    "Mokshada": date(2026, 12, 20),
}

# Smartha/Vaishnava splits that both panchang and jyotisha agree on.
VERIFIED_EKADASHI_VAISHNAVA = {
    "Yogini": date(2026, 7, 11),
    "Prabodhini": date(2026, 11, 21),
}


class TestEkadashi2026:
    def test_verified_smartha_dates(self, ekadashis_2026_delhi):
        eks = _by_name(ekadashis_2026_delhi)
        for name, expected in VERIFIED_EKADASHI_SMARTHA.items():
            assert name in eks, f"{name} Ekadashi missing from 2026 output"
            got = eks[name].smartha_date
            assert got == expected, f"{name}: expected {expected}, got {got}"

    def test_verified_vaishnava_dates(self, ekadashis_2026_delhi):
        eks = _by_name(ekadashis_2026_delhi)
        for name, expected in VERIFIED_EKADASHI_VAISHNAVA.items():
            got = eks[name].vaishnava_date
            assert got == expected, f"{name}: expected {expected}, got {got}"

    def test_padmini_ekadashi_present(self, ekadashis_2026_delhi):
        """Regression for BUG-1. Adhika Jyeshtha Shukla Ekadashi = Padmini,
        2026-05-27 sarva (jyotisha + Drik Panchang). The tithi spans two
        sunrises with Dashami-vedha on day 1, so both Smartha and Vaishnava
        shift to day 2 — also covers the Dharmasindhu two-sunrise rule."""
        eks = _by_name(ekadashis_2026_delhi)
        assert "Padmini" in eks
        assert eks["Padmini"].is_adhik is True
        assert eks["Padmini"].smartha_date == date(2026, 5, 27)
        assert eks["Padmini"].vaishnava_date == date(2026, 5, 27)

    def test_parama_ekadashi_present(self, ekadashis_2026_delhi):
        """Regression for BUG-1. Adhika Jyeshtha Krishna Ekadashi = Parama,
        2026-06-11 (jyotisha + Drik Panchang)."""
        eks = _by_name(ekadashis_2026_delhi)
        assert "Parama" in eks
        assert eks["Parama"].is_adhik is True
        assert eks["Parama"].smartha_date == date(2026, 6, 11)

    def test_regular_ekadashis_not_adhik(self, ekadashis_2026_delhi):
        for e in ekadashis_2026_delhi:
            if e.name not in ("Padmini", "Parama"):
                assert e.is_adhik is False, e.name

    def test_ekadashi_spacing_invariant(self, ekadashis_2026_delhi):
        """Ekadashis occur every ~14.8 days; consecutive smartha dates can
        never be more than 17 days apart. Catches dropped ekadashis in any
        year, not just 2026 (regression for BUG-1's 42-day gap)."""
        dates = sorted(e.smartha_date for e in ekadashis_2026_delhi)
        gaps = [(b - a).days for a, b in zip(dates, dates[1:])]
        assert max(gaps) <= 17, f"impossible ekadashi gap of {max(gaps)} days"

    def test_ekadashi_count_adhik_year(self, ekadashis_2026_delhi):
        """22 regular + Padmini + Parama fall within calendar year 2026."""
        assert len(ekadashis_2026_delhi) == 24

    def test_no_adhik_ekadashis_in_regular_year(self, delhi):
        """2025 has no adhik maas → no Padmini/Parama, no adhik flags."""
        from panchang.calendar import compute_ekadashis

        eks = compute_ekadashis(2025, delhi)
        names = {e.name for e in eks}
        assert "Padmini" not in names
        assert "Parama" not in names
        assert not any(e.is_adhik for e in eks)
        dates = sorted(e.smartha_date for e in eks)
        gaps = [(b - a).days for a, b in zip(dates, dates[1:])]
        assert max(gaps) <= 17


# ─── Festivals 2026 ──────────────────────────────────────────────────────────

# id -> date. All agree across panchang, jyotisha and published calendars.
VERIFIED_FESTIVALS = {
    "makar_sankranti": date(2026, 1, 14),
    # BUG-3 regression: nishita-vyapti (the night holding Chaturdashi).
    "maha_shivaratri": date(2026, 2, 15),
    "holika_dahan": date(2026, 3, 2),
    "holi": date(2026, 3, 3),
    # BUG-3 regression: madhyahna-vyapti (Smarta date; ISKCON differs).
    "ram_navami": date(2026, 3, 26),
    "akshaya_tritiya": date(2026, 4, 19),
    "buddha_purnima": date(2026, 5, 1),
    "raksha_bandhan": date(2026, 8, 28),
    "janmashtami": date(2026, 9, 4),
    # BUG-3 regression: madhyahna-vyapti.
    "ganesh_chaturthi": date(2026, 9, 14),
    # BUG-5 regression: aparahna-vyapti with purva tie-break (Dashami
    # covers aparahna on both Oct 20 and 21 → earlier day wins).
    "dussehra": date(2026, 10, 20),
    # BUG-2 regressions: purnimant "Kartik Krishna" = amant Ashwin Krishna.
    "karva_chauth": date(2026, 10, 29),
    "ahoi_ashtami": date(2026, 11, 1),
    "dhanteras": date(2026, 11, 6),
    "narak_chaturdashi": date(2026, 11, 8),
    # BUG-3 regression: pradosha-vyapti (Amavasya at pradosh = Lakshmi Puja).
    "diwali": date(2026, 11, 8),
    "govardhan_puja": date(2026, 11, 10),
    "bhai_dooj": date(2026, 11, 11),
}


class TestFestivals2026:
    def test_verified_dates(self, festivals_2026_delhi):
        fests = _by_id(festivals_2026_delhi)
        for fid, expected in VERIFIED_FESTIVALS.items():
            assert fid in fests, f"festival {fid} missing from 2026 output"
            got = fests[fid].date
            assert got == expected, (
                f"{fid}: expected {expected}, got {got}. Reasoning: {fests[fid].reasoning}"
            )

    def test_kaala_rules_applied(self, festivals_2026_delhi):
        """The kaala-based observance rules must actually be engaged,
        not silently fall back to sunrise/paraviddha (BUG-3 regression)."""
        fests = _by_id(festivals_2026_delhi)
        expected_kaala = {
            "maha_shivaratri": "nishita",
            "ram_navami": "madhyahna",
            "ganesh_chaturthi": "madhyahna",
            "dussehra": "aparahna",
            "diwali": "pradosha",
            "dhanteras": "pradosha",
        }
        for fid, kaala in expected_kaala.items():
            assert fests[fid].priority_applied == "vyapti", fid
            assert fests[fid].kaala_applied == kaala, fid

    def test_diwali_cluster_ordering(self, festivals_2026_delhi):
        """Structural invariant, valid every year: Karva Chauth < Ahoi
        Ashtami < Dhanteras < Narak Chaturdashi < Diwali < Govardhan Puja
        < Bhai Dooj."""
        fests = _by_id(festivals_2026_delhi)
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
        assert dates[-1] - dates[0] < timedelta(days=15), "Diwali cluster must span < 15 days"


# ─── API footguns ────────────────────────────────────────────────────────────


class TestLocationFootgun:
    def test_unknown_kwargs_rejected(self):
        """Regression for BUG-4: Location(timezone=...) used to be silently
        dropped (field is ``tz``), computing every date for UTC."""
        from pydantic import ValidationError

        from panchang.types import Location

        with pytest.raises(ValidationError):
            Location(lat=28.6139, lng=77.2090, timezone="Asia/Kolkata")
