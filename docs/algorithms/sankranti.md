# Sankranti Computation

A Sankranti is the moment when the Sun's sidereal longitude crosses a multiple of 30 degrees, entering a new Rashi (zodiac sign). There are 12 Sankrantis per year.

## The 12 Rashis

| Index | Rashi | Sidereal Range | Sankranti Name | Approx. Date |
|---|---|---|---|---|
| 0 | Mesha (Aries) | 0-30 | Mesha Sankranti | Apr 14 |
| 1 | Vrishabha (Taurus) | 30-60 | Vrishabha Sankranti | May 14 |
| 2 | Mithuna (Gemini) | 60-90 | Mithuna Sankranti | Jun 15 |
| 3 | Karka (Cancer) | 90-120 | Karka Sankranti | Jul 16 |
| 4 | Simha (Leo) | 120-150 | Simha Sankranti | Aug 17 |
| 5 | Kanya (Virgo) | 150-180 | Kanya Sankranti | Sep 17 |
| 6 | Tula (Libra) | 180-210 | Tula Sankranti | Oct 17 |
| 7 | Vrischika (Scorpio) | 210-240 | Vrischika Sankranti | Nov 16 |
| 8 | Dhanu (Sagittarius) | 240-270 | Dhanu Sankranti | Dec 16 |
| 9 | Makara (Capricorn) | 270-300 | Makar Sankranti | Jan 14 |
| 10 | Kumbha (Aquarius) | 300-330 | Kumbha Sankranti | Feb 13 |
| 11 | Meena (Pisces) | 330-360 | Meena Sankranti | Mar 14 |

Note: In constants, Sankrantis are stored in **calendar-year order** (Makar first, Dhanu last), not Rashi order.

## Algorithm

### Input
- Year (Gregorian)

### Output
- 12 `SankrantiInfo` structs, each containing: index, name, Rashi, target longitude, exact JD

### Steps

1. **Initialize approximate search dates**: For each Sankranti, start searching ~2 weeks before the expected date. These approximate dates are stored in `SANKRANTI_APPROX_DATES`.

2. **Convert to Julian Day**: `approx_jd = midnight_jd(year, approx_month, approx_day, utc_offset=0)`

3. **Find exact crossing**: Call `find_crossing_forward(approx_jd, target_longitude, sun_sidereal, max_days=45)` where `sun_sidereal(jd)` returns the Sun's sidereal longitude at the given JD.

4. **Search mechanism**: The `find_crossing_forward` function:
   - Steps forward in 1-hour increments from the approximate JD
   - At each step, checks if `crossed_target(sun_sidereal(jd_a), sun_sidereal(jd_b), target)`
   - When a crossing bracket is found, refines via bisection to 1-second precision
   - 45-day search window (Sun moves ~1 degree/day, so 30 degrees takes ~30 days)

5. **Result**: The exact JD (UT) when the Sun's sidereal longitude equals the target

### Ayanamsa Correction

The Swiss Ephemeris computes tropical longitudes. The sidereal longitude is:

```
sidereal = tropical - ayanamsa
```

We use the **Lahiri/Chitrapaksha** ayanamsa, which is the Government of Bhārat standard. For 2026, the ayanamsa is approximately 24.22 degrees.

### Precision

The bisection search converges to 1/86400 JD (~1 second). At the computed JD, the Sun's sidereal longitude is within 0.02 degrees (72 arcseconds) of the target, verified by unit tests.

### Spacing

Consecutive Sankrantis are typically 28-33 days apart, reflecting the slightly variable speed of the Sun along the ecliptic (faster near perihelion in January, slower near aphelion in July).

## Cultural Significance

- **Makar Sankranti** (Jan 14): Harvest festival, marks the Sun's northward journey (Uttarayana in tropical astronomy, though sidereal Uttarayana is the traditional definition)
- **Mesha Sankranti** (Apr 14): Hindu solar New Year in many traditions; Baisakhi (Punjab), Vishu (Kerala), Puthandu (Tamil Nadu)
