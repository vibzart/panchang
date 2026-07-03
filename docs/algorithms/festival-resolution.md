# Festival Date Resolution

Hindu festivals are determined by the luni-solar calendar — most fall on a specific Tithi within a specific lunar month. Resolving the Gregorian date requires mapping from (lunar month, tithi) to a calendar date, then applying the festival's **observance rule** (which kaala of the day must hold the tithi, and which day wins when the tithi spans two).

## Resolution Rules

### 1. TithiAtSunrise Rule

Most Hindu festivals follow this rule: the festival's *natural day* is the day when the target Tithi **prevails at sunrise** (udayatithi), optionally shifted by a priority rule (below).

**Why sunrise?** The Hindu day begins at sunrise. If a Tithi spans parts of two calendar days, the day where the Tithi is active at sunrise "owns" that Tithi.

### 2. Sankranti Rule

Some festivals (Makar Sankranti, Baisakhi) are determined by the Sun entering a specific Rashi, not by Tithi. The festival date is the local civil date of the Sankranti.

### 3. NakshatraAtSunrise Rule

A few festivals (Onam) are determined by a nakshatra prevailing at sunrise near an anchor Sankranti.

## Month Windows (Amant) + Sankranti Anchor

Month numbers in `festivals.yaml` are **AMANT** (month = Amavasya→Amavasya). Popular North-Indian names for Krishna-paksha festivals use the PURNIMANT label, which is one month ahead for that paksha: "Kartik Krishna Trayodashi" (Dhanteras) is amant **Ashwin** (7), not Kartik (8).

Resolution steps:

1. **Compute the year's Amant month windows once** (shared across all festival definitions).
2. **Find the naming Sankranti** for the definition's month (a Sankranti always falls inside the month it names). The anchor disambiguates **duplicate month instances**: a month spilling past Dec 31 appears in two consecutive years' window lists, and the instance containing the anchor is the right one (getting this wrong silently dropped Margashirsha festivals in ~half of years).
3. **Select the target window per the `adhika_maasa` policy** (see below), then search it for the first sunrise holding the target tithi. A sunrise on the window's first civil day that precedes the month-start moment belongs to the previous month and is never an exact match (it would wrongly match the previous Amavasya for tithi-30 festivals), but it is kept as the kshaya fallback day for tithi 1.
4. **Kshaya fallback**: if the tithi never spans a sunrise in the window, the preceding tithi's first day is used (Dharmasindhu).
5. **Sankranti-anchored ±20-day search** remains only as a last-resort fallback when the window search fails entirely.

For **display**, the Purnimant system (default) names the Krishna paksha of amant month N as month N+1 — so Dhanteras still reads "Kartik Krishna Trayodashi" while being resolved in the amant Ashwin window.

## Observance Rules (priority / kaala)

After the natural (udayatithi) day is found, the definition's `priority` decides the final day:

| priority | Behavior |
|---|---|
| `paraviddha` (default) | Keep the natural day. |
| `puurvaviddha` | Shift one day earlier (where the tithi begins). |
| `vyapti` | Check which of (earlier, natural) day has the tithi **present during the configured `kaala` window**. Earlier day wins if it alone qualifies. If BOTH qualify, the tie-break is `vyapti_tie`: `para` (default — natural day) or `purva` (earlier day). |

`kaala` values: `sunrise`, `praatah`, `sangava`, `madhyahna`, `aparahna`, `saayaahna`, `poorvahna` (panchama-vibhaga day divisions computed from actual sunrise/sunset), `pradosha` (sunset → sunset + 144 min), `nishita` (centered on true midnight of the night), `full_day`.

Examples in the shipped YAML (each verified against Drik Panchang and jyotisha for 2026 + 2027):

| Festival | Rule | Why |
|---|---|---|
| Maha Shivaratri | `vyapti` / `nishita` | The day whose *night* holds Chaturdashi |
| Ram Navami, Ganesh Chaturthi | `vyapti` / `madhyahna` | Midday birth/puja |
| Diwali, Dhanteras, Karva Chauth, Ahoi Ashtami | `vyapti` / `pradosha` | Evening observance |
| Akshaya Tritiya | `vyapti` / `aparahna` | Dana-pradhan afternoon rule |
| Dussehra | `vyapti` / `aparahna` / `vyapti_tie: purva` | "dinadvaye aparahna-vyaptau purva" (Nirnaya Sindhu) |

When a vyapti rule shifts the day, the paraviddha (udayatithi) date is surfaced as `alternate` so callers can present both, and `priority_applied` / `kaala_applied` report which rule actually decided the date.

## Adhik Maas Handling

Whether a festival is observed in an Adhika (intercalary) month is policy-driven via the `adhika_maasa` field:

| Value | Behavior |
|---|---|
| `nija` (default) | Observe only in the regular month. |
| `adhika` | Observe only in the adhika month (skip years without one). |
| `adhika_if_exists` | Prefer the adhika month; fall back to nija. |
| `adhika_and_nija` | Observe in both (currently resolves the nija instance; emitting both is a planned enhancement). |

Month-recurring **vrats** (Pradosh, Sankashti Chaturthi, Amavasya, Purnima) ARE observed in adhika months and are emitted with an "Adhika" month label. Adhika-month **Ekadashis** (Padmini/Parama) are handled by the ekadashi engine — see [ekadashi.md](ekadashi.md).

## Festival Definitions in YAML

Festival specifications are stored in `data/festivals.yaml`, not hardcoded in Rust. Each definition:

```yaml
- id: diwali
  name: Diwali
  rule: tithi_at_sunrise
  lunar_month: 7        # amant Ashwin ("Kartik Amavasya" is the purnimant label)
  tithi: 30             # Amavasya
  priority: vyapti
  kaala: pradosha
```

Optional fields: `priority`, `kaala`, `vyapti_tie`, `adhika_maasa` (all default to paraviddha / sunrise / para / nija). Python loads the YAML at import time and passes structured dicts to Rust via PyO3 — no YAML parsing in Rust.

## Reasoning Field

Every resolved festival includes a `reasoning` string explaining the date determination — which tithi prevailed when, which kaala/priority rule was applied, and (for vyapti shifts) the paraviddha alternate. This serves as:
- **Transparency**: Users can verify why a date was chosen
- **Debugging**: Developers can trace resolution logic
- **Trust signal**: Shows the computation is grounded in astronomical data, not lookup tables

Example:
> "Kartik Krishna Amavasya (Tithi 30) is present during the pradosha kaala on 2026-11-08 but NOT during that kaala on 2026-11-09. Vyapti rule: observe on the earlier day. Paraviddha alternate (udayatithi rule): 2026-11-09."
