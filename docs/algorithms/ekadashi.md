# Ekadashi Rules

Ekadashi ("eleventh") is the 11th Tithi in each Paksha. With two Pakshas per lunar month and 12 months per year, there are 24 named Ekadashis annually — **26 in an Adhika-maas year**, when the intercalary month contributes Padmini (Shukla) and Parama (Krishna). Ekadashi is considered the most important fasting day in the Hindu calendar.

## The Two Ekadashi Traditions

### Smartha (Traditional)

The Smartha Ekadashi is observed on the day when Ekadashi Tithi prevails at sunrise (udayatithi), refined by the vedha and vriddhi rules below.

- **Shukla Ekadashi**: Tithi 11 (Shukla Paksha)
- **Krishna Ekadashi**: Tithi 26 (Krishna Paksha, which is the 11th Tithi of the waning fortnight)

### Vaishnava (ISKCON/Gaudiya)

The Vaishnava tradition adds a stricter requirement: **Dashami (the 10th Tithi) must have ended before Arunodaya**.

**Arunodaya** = 96 minutes before sunrise (the pre-dawn twilight when the first light appears).

If Dashami still prevails at Arunodaya, the Vaishnava Ekadashi shifts to the **next day**. The reasoning is that any "contamination" by the 10th Tithi at Arunodaya makes the fast impure.

## Algorithm

### Input
- Ekadashi definition: `(month, shukla_name, krishna_name)`
- Year, location (lat, lng, alt, utc_offset)

### Steps

1. **Compute Amant month windows** for `year − 1`, `year`, `year + 1` (Amavasya→Amavasya boundaries; the ±1 years catch Ekadashis whose lunar month crosses Dec 31 / Jan 1).

2. **For every window matching the definition's month number** — including **adhika instances** — search the window for the first sunrise where the target Tithi (11 or 26) prevails. In an adhika month the Ekadashis take the universal names **Padmini** (Shukla) and **Parama** (Krishna) regardless of which month is doubled, and the occurrence is flagged `is_adhik = true`.

   The search includes a **kshaya fallback**: when the Ekadashi Tithi never spans a sunrise inside the window (tithi-kshaya), the preceding Dashami day is used per Smarta convention (Dharmasindhu).

3. **Apply the Dharmasindhu nirnaya** (three cases, checked in order):

   | Case | Smartha | Vaishnava |
   |---|---|---|
   | **Tithi-vriddhi** — Ekadashi prevails at BOTH the found sunrise and the next day's sunrise | day 2 | day 2 |
   | **Dashami-vedha** — Dashami still runs at Arunodaya on day 1 (and no vriddhi) | day 1 | day 2 |
   | Neither | day 1 | day 1 |

   The vriddhi rule ("vriddhau uttara") applies regardless of vedha — e.g. Vijaya 2027 spans the sunrises of Mar 3 and Mar 4 with no vedha, and everyone observes Mar 4.

4. **Filter to the requested Gregorian year and dedupe** (the ±1-year window slices can resolve the same Ekadashi twice).

### Output
- `EkadashiOccurrence`: name, lunar month, `is_adhik`, paksha, Smartha date, Vaishnava date, reasoning

## The 24 Named Ekadashis

| Lunar Month | Shukla Ekadashi | Krishna Ekadashi |
|---|---|---|
| 1 Chaitra | Kamada | Varuthini |
| 2 Vaishakha | Mohini | Apara |
| 3 Jyeshtha | Nirjala | Yogini |
| 4 Ashadha | Devshayani | Kamika |
| 5 Shravana | Shravana Putrada | Aja |
| 6 Bhadrapada | Parsva | Indira |
| 7 Ashwin | Papankusha | Rama |
| 8 Kartik | Prabodhini | Utpanna |
| 9 Margashirsha | Mokshada | Saphala |
| 10 Pausha | Pausha Putrada | Shattila |
| 11 Magha | Jaya | Vijaya |
| 12 Phalguna | Amalaki | Papamochani |
| any Adhika month | **Padmini** | **Parama** |

Ekadashi names are stored in `data/festivals.yaml` under the `ekadashis` key and loaded by Python at import time (the adhika names are engine constants).

## When Smartha and Vaishnava Differ

The dates differ when Dashami persists into the pre-dawn hours **and** the Ekadashi does not reach the next day's sunrise:
- The Dashami-to-Ekadashi transition occurs **after** Arunodaya but **before** sunrise
- In such cases, Smartha followers observe on that day (Ekadashi at sunrise), but Vaishnava followers wait until the next day (a Dvadashi fast)

Typically, Smartha and Vaishnava dates differ for 2-4 Ekadashis per year (2026: Yogini Jul 10/11, Prabodhini Nov 20/21 — verified against Drik Panchang and jyotisha).

## Nirjala Ekadashi

Nirjala (Jyeshtha Shukla Ekadashi) is the strictest Ekadashi — even water is not consumed. Tradition holds that observing Nirjala Ekadashi alone is equivalent to observing all 24 Ekadashis. In an Adhika-Jyeshtha year, Nirjala falls in the **nija** (regular) Jyeshtha — e.g. 2026-06-25, after Adhika Jyeshtha's Padmini (May 27) and Parama (Jun 11).
