# Panchang Elements

The Panchang ("five limbs") consists of five elements computed from the positions of the Sun and Moon at sunrise on a given day.

## Why Sunrise?

The Hindu day (Ahoratra) begins at sunrise, not midnight. All Panchang elements are evaluated at the moment of sunrise to determine which element "prevails" for that day. This is the TithiAtSunrise convention used across Bhārat.

## 1. Vara (Weekday)

The simplest element. Vara is the weekday, numbered 0-6 (Sunday through Saturday).

**Algorithm**: Tomohiko Sakamoto's method computes the weekday directly from the Gregorian date (year, month, day) without converting to Julian Day. This avoids timezone-related off-by-one errors that arise when computing weekday from a JD at local midnight (which maps to a different UT date for positive timezone offsets).

**Sanskrit names**: Ravivara (Sun-day), Somavara (Moon-day), Mangalavara (Mars-day), Budhavara (Mercury-day), Guruvara (Jupiter-day), Shukravara (Venus-day), Shanivara (Saturn-day).

## 2. Tithi (Lunar Day)

A Tithi is 1/30th of the synodic month — the time it takes for the Moon to gain 12 degrees of longitude over the Sun.

**Angular function**: `tithi_angle(jd) = normalize(Moon_tropical - Sun_tropical)`

**Why tropical?** Although Bhāratīya astronomy uses the sidereal zodiac, the Tithi is defined by the angular separation between Moon and Sun. The ayanamsa correction is the same for both bodies, so it cancels out in the difference. Using tropical longitudes gives the same result with one fewer computation.

**Tithi number**: `floor(tithi_angle / 12) + 1`, yielding 1-30.

**Pakshas**:
- **Shukla (waxing)**: Tithis 1-15 (Pratipada to Purnima)
- **Krishna (waning)**: Tithis 16-30 (Pratipada to Amavasya)

**Transition times**: Found via bisection search. The start time is when the angle crosses `(tithi_idx * 12)` degrees; the end time is when it crosses `((tithi_idx + 1) * 12)` degrees. Search uses 1-hour bracket steps and converges to 1-second precision.

## 3. Nakshatra (Lunar Mansion)

The 27 Nakshatras divide the sidereal ecliptic into equal 13°20' segments. The Nakshatra at sunrise is determined by the Moon's sidereal longitude.

**Angular function**: `nakshatra_angle(jd) = Moon_sidereal(jd)`

**Why sidereal?** Unlike Tithi (a Sun-Moon difference), Nakshatra depends on the Moon's absolute position against the fixed star backdrop. The Lahiri ayanamsa correction is essential here.

**Nakshatra number**: `floor(Moon_sidereal / 13.333) + 1`, yielding 1-27.

**Pada (quarter)**: Each Nakshatra has 4 padas of 3°20' each. `pada = floor(offset_within_nakshatra / 3.333) + 1`.

**Lords**: The 9 Vimshottari Dasha lords cycle through the 27 Nakshatras in groups of 3: Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury.

## 4. Yoga (Sun-Moon Combination)

The 27 Yogas are computed from the sum of the Sun's and Moon's sidereal longitudes, divided into 13°20' segments.

**Angular function**: `yoga_angle(jd) = normalize(Sun_sidereal + Moon_sidereal)`

**Yoga number**: `floor(yoga_angle / 13.333) + 1`, yielding 1-27.

The Yoga indicates the combined "energy" of the Sun and Moon at a given time. Certain Yogas are considered auspicious (e.g., Siddhi, Amrit) and others inauspicious (e.g., Vyatipata, Vaidhriti).

## 5. Karana (Half-Tithi)

A Karana is half a Tithi (6 degrees of Sun-Moon separation). There are 60 Karanas per lunar month, but only 11 unique names.

**Angular function**: Same as Tithi (`tithi_angle`), but divided into 6-degree segments.

**Karana number (0-59)**: `floor(tithi_angle / 6)`

**Naming cycle**:
- Karana 0: **Kimstughna** (fixed — first half of Shukla Pratipada)
- Karanas 1-56: Seven rotating names repeated 8 times: Bava, Balava, Kaulava, Taitila, Garaja, Vanija, **Vishti** (Bhadra)
- Karana 57: **Shakuni** (fixed)
- Karana 58: **Chatushpada** (fixed)
- Karana 59: **Nagava** (fixed)

**Vishti (Bhadra)** is considered inauspicious and is marked in traditional calendars.

## Transition Time Search

All four time-varying elements (Tithi, Nakshatra, Yoga, Karana) have precise start and end times computed via the same bisection algorithm:

1. **Bracket search**: From the sunrise JD, step forward/backward in 1-hour increments
2. **Crossing detection**: `crossed_target(angle_a, angle_b, target)` checks if the target angle falls within the forward sweep from `angle_a` to `angle_b`, correctly handling 360-degree wraparound
3. **Bisection refinement**: Once a bracket `[jd_a, jd_b]` containing the crossing is found, repeatedly halve the interval until precision reaches 1/86400 JD (~1 second) or 50 iterations

The `crossed_target` function handles wraparound by computing forward angular distances:
```
forward_sweep = (b - a + 360) mod 360
forward_to_target = (target - a + 360) mod 360
crossed = sweep < 180 AND forward_to_target <= sweep
```

The 180-degree sanity check prevents false positives from retrograde motion or large steps.
