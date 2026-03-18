# Ekadashi Rules

Ekadashi ("eleventh") is the 11th Tithi in each Paksha. With two Pakshas per lunar month and 12 months per year, there are 24 named Ekadashis annually. Ekadashi is considered the most important fasting day in the Hindu calendar.

## The Two Ekadashi Traditions

### Smartha (Traditional)

The Smartha Ekadashi is observed on the day when Ekadashi Tithi prevails at sunrise. This is the standard TithiAtSunrise rule.

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
- Precomputed Sankrantis for the year

### Steps

1. **Find the naming Sankranti** for the lunar month (same as festival resolution — see [festival-resolution.md](festival-resolution.md))

2. **Find Smartha date**: Search +/-20 days around the Sankranti JD for the target Tithi (11 for Shukla, 26 for Krishna) at sunrise. Pick the closest match.

3. **Check Vaishnava rule**:
   - Compute Arunodaya: `arunodaya_jd = smartha_sunrise_jd - 96/1440` (96 minutes in days)
   - Compute Tithi at Arunodaya
   - If Tithi at Arunodaya equals **Dashami** (10 for Shukla, 25 for Krishna):
     - Vaishnava date shifts to the next day
     - Compute next day's sunrise for the Vaishnava date

4. **Generate reasoning**:
   - If dates are the same: "Dashami ended before Arunodaya — Smartha and Vaishnava dates are the same."
   - If dates differ: "Dashami (Tithi 10) persists at Arunodaya (96 min before sunrise), so Vaishnava Ekadashi shifts to [next date]."

### Output
- `EkadashiOccurrence`: name, lunar month, paksha, Smartha date, Vaishnava date, reasoning

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

Ekadashi names are stored in `data/festivals.yaml` under the `ekadashis` key and loaded by Python at import time.

## When Smartha and Vaishnava Differ

The dates differ when Dashami persists into the pre-dawn hours. This happens when:
- The Dashami-to-Ekadashi transition occurs **after** Arunodaya but **before** sunrise
- In such cases, Smartha followers observe on that day (Ekadashi at sunrise), but Vaishnava followers wait until the next day

Typically, Smartha and Vaishnava dates differ for 3-6 of the 24 Ekadashis per year.

## Nirjala Ekadashi

Nirjala (Jyeshtha Shukla Ekadashi) is the strictest Ekadashi — even water is not consumed. Tradition holds that observing Nirjala Ekadashi alone is equivalent to observing all 24 Ekadashis.
