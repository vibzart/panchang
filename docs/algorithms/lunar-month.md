# Lunar Month Determination

Hindu lunar months are named after the Sankranti (solar ingress) that falls within their boundaries. Two calendar systems define the boundaries differently.

## Calendar Systems

### Amant (South Bhārat)
- Month boundaries: consecutive **Amavasyas** (New Moons, tithi angle = 0 degrees)
- Used in: Karnataka, Andhra Pradesh, Tamil Nadu, Maharashtra, Gujarat
- Also called: Amanta, Amavasyant

### Purnimant (North Bhārat)
- Month boundaries: consecutive **Purnimas** (Full Moons, tithi angle = 180 degrees)
- Used in: Uttar Pradesh, Madhya Pradesh, Rajasthan, Bihar, Nepal
- Also called: Purnimanta

Both systems name months after the **same Sankranti**. The difference is only in where the month boundary falls. In the Amant system, the Shukla paksha (waxing fortnight) comes first; in Purnimant, the Krishna paksha (waning fortnight) comes first.

## Month Names

| Number | Name | Approximate Period | Naming Sankranti |
|---|---|---|---|
| 1 | Chaitra | Mar-Apr | Mesha Sankranti (Sun enters Mesha) |
| 2 | Vaishakha | Apr-May | Vrishabha Sankranti |
| 3 | Jyeshtha | May-Jun | Mithuna Sankranti |
| 4 | Ashadha | Jun-Jul | Karka Sankranti |
| 5 | Shravana | Jul-Aug | Simha Sankranti |
| 6 | Bhadrapada | Aug-Sep | Kanya Sankranti |
| 7 | Ashwin | Sep-Oct | Tula Sankranti |
| 8 | Kartik | Oct-Nov | Vrischika Sankranti |
| 9 | Margashirsha | Nov-Dec | Dhanu Sankranti |
| 10 | Pausha | Dec-Jan | Makar Sankranti |
| 11 | Magha | Jan-Feb | Kumbha Sankranti |
| 12 | Phalguna | Feb-Mar | Meena Sankranti |

## Algorithm

### Finding Boundary Points

1. **Search range**: From December 1 of the previous year to January 31 of the next year (to capture months overlapping the calendar year)

2. **For Amant**: Find all Amavasyas by searching for tithi angle crossings at 0 degrees
   - `tithi_angle(jd) = normalize(Moon_tropical - Sun_tropical)`
   - Use `find_crossing_forward` with target = 0 degrees, stepping in 1-hour brackets
   - Jump forward 25 days after each crossing (lunar cycle is ~29.5 days)
   - Result: ~13-15 Amavasyas

3. **For Purnimant**: Same process but searching for crossings at 180 degrees

### Assigning Month Names

For each consecutive pair of boundaries `[start_jd, end_jd]`:

1. **Find Sankrantis within**: Filter the year's 12 Sankrantis (plus adjacent years' Sankrantis for boundary months) to those where `start_jd < sankranti_jd <= end_jd`

2. **Match count determines month type**:
   - **1 Sankranti** (normal): The Sankranti's Rashi maps to the month name via `SANKRANTI_TO_LUNAR_MONTH`. For example, Mesha Sankranti (Rashi index 0) maps to month 1 (Chaitra).
   - **0 Sankrantis** (Adhik Maas): An intercalary month. Named after the **next** month's Sankranti, with the "Adhik" prefix.
   - **2 Sankrantis** (Kshaya Maas): A compressed month (extremely rare). Named after the first Sankranti.

### Filtering

Finally, filter to months that overlap with the requested Gregorian year.

## Adhik Maas (Intercalary Month)

A lunar month (~29.5 days) is shorter than a solar month (~30.4 days). Over time, the lunar calendar drifts ahead. When no Sankranti falls within a lunar month, that month is an **Adhik Maas** (extra month).

This happens roughly every 32-33 months (about 7 times in 19 years, following the Metonic cycle). Adhik months are considered inauspicious for ceremonies, and festivals are **not** observed during them — they are celebrated in the Nija (regular) month instead.

## Kshaya Maas (Compressed Month)

When two Sankrantis fall within a single lunar month, it's a Kshaya Maas. This is extremely rare and is always accompanied by an Adhik Maas nearby to compensate. Kshaya months last occurred in 1983.

## Year Boundary Handling

Since lunar months don't align with January 1:
- Sankrantis are computed for year-1, year, and year+1 (then sorted by JD)
- Boundary search starts from December of the previous year
- Results are filtered to months overlapping with the requested year
