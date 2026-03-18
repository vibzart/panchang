# Festival Date Resolution

Hindu festivals are determined by the luni-solar calendar — most fall on a specific Tithi within a specific lunar month. Resolving the Gregorian date requires mapping from (lunar month, tithi) to a calendar date.

## Resolution Rules

### 1. TithiAtSunrise Rule

Most Hindu festivals follow this rule: the festival is observed on the day when the target Tithi **prevails at sunrise**.

**Why sunrise?** The Hindu day begins at sunrise. If a Tithi spans parts of two calendar days, the day where the Tithi is active at sunrise "owns" that Tithi.

### 2. Sankranti Rule

Some festivals (Makar Sankranti, Baisakhi) are determined by the Sun entering a specific Rashi, not by Tithi. The festival date is simply the date of the Sankranti.

## The Sankranti-Based Search Approach

### The Problem with Month Boundaries

The naive approach — find the lunar month boundaries, then search within them for the target Tithi — has edge cases:

- In the **Purnimant** system, the month boundary IS the Purnima crossing. Festivals on Purnima (like Holi on Phalguna Purnima) may fall exactly at the boundary, causing resolution failures.
- **Adhik Maas** detection depends on whether a Sankranti falls within the month, but the month boundaries themselves depend on Amavasya/Purnima detection.

### The Solution

Both Amant and Purnimant systems name months after the **same Sankranti**. Rather than finding month boundaries and searching within them, we:

1. Find the Sankranti that names the festival's lunar month
2. Search **+/-20 days** around that Sankranti for the target Tithi at sunrise
3. If multiple matches exist (can happen near month boundaries), pick the one **closest** to the Sankranti

This approach is:
- **Calendar-system agnostic**: Works identically for Amant and Purnimant
- **Boundary-safe**: No edge cases at Amavasya/Purnima crossings
- **Simple**: No need to compute full lunar month structures for festival resolution

### Mapping: Lunar Month to Sankranti

Each lunar month is named after the Sankranti where the Sun enters the corresponding Rashi:
- Month 1 (Chaitra) -> Mesha Sankranti (Rashi 0) -> Sankranti index 3
- Month 8 (Kartik) -> Vrischika Sankranti (Rashi 7) -> Sankranti index 10

The mapping uses `SANKRANTI_RASHI_INDEX`: find `i` where `SANKRANTI_RASHI_INDEX[i] == (month - 1)`.

## Algorithm: TithiAtSunrise

### Input
- Festival definition: `(id, name, lunar_month, tithi)`
- Year, location (lat, lng, alt, utc_offset)
- Precomputed Sankrantis for the year

### Steps

1. **Find the naming Sankranti**: Map `lunar_month` to a Rashi index `(lunar_month - 1)`, then find the Sankranti index where `SANKRANTI_RASHI_INDEX[i]` equals that Rashi index.

2. **Compute search window**: Start from 20 days before the Sankranti JD, end 20 days after. Convert to a local midnight JD for day iteration.

3. **Iterate days**: For each of the 40 days in the search window:
   - Compute sunrise JD using `sun::sunrise_jd(midnight, lat, lng, alt)`
   - Compute Tithi number at sunrise: `floor(tithi_angle(sunrise) / 12) + 1`
   - If Tithi matches the target, record (distance_from_sankranti, sunrise_jd, date)

4. **Select best match**: Among all matching days, pick the one closest to the Sankranti JD.

5. **Generate reasoning**: Build a human-readable explanation string, e.g.:
   > "Kartik Shukla Ashtami (Tithi 8) prevails at sunrise (06:23) on 2026-10-29. Lunar month Kartik determined by Vrischika Sankranti (Vrischika) on 2026-11-16."

### Output
- `FestivalOccurrence`: date, sunrise JD, tithi at sunrise, lunar month name, reasoning string

## Algorithm: Sankranti Festival

For Sankranti-based festivals:

1. Look up the Sankranti by index from the precomputed array
2. Convert the Sankranti JD to a local date
3. Compute sunrise for that date and report the Tithi at sunrise
4. Generate reasoning describing the Sun's entry into the Rashi

## Adhik Maas Handling

Festivals are **not** observed in Adhik (intercalary) months. The Sankranti-based search naturally handles this: since the search anchors to the naming Sankranti (which by definition falls in the Nija month), the closest match will always be in the regular month, not the Adhik month.

## Festival Definitions in YAML

Festival specifications are stored in `data/festivals.yaml`, not hardcoded in Rust. This enables:
- Easy addition of new festivals without recompiling
- Community contributions via pull request
- Regional variants as separate YAML files

Each definition specifies:
```yaml
- id: diwali
  name: Diwali
  rule: tithi_at_sunrise
  lunar_month: 8        # Kartik
  tithi: 30              # Amavasya
```

Python loads the YAML at import time and passes structured dicts to Rust via PyO3. The Rust engine receives festival definitions as `Vec<FestivalDef>` — no YAML parsing in Rust.

## Reasoning Field

Every resolved festival includes a `reasoning` string explaining the date determination. This serves as:
- **Transparency**: Users can verify why a date was chosen
- **Debugging**: Developers can trace resolution logic
- **Trust signal**: Shows the computation is grounded in astronomical data, not lookup tables

Example:
> "Phalguna Shukla Purnima (Tithi 15) prevails at sunrise (06:38) on 2026-03-03. Lunar month Phalguna determined by Meena Sankranti (Meena) on 2026-03-14."
