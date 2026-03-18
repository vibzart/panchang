# Batch Computation

The batch API computes Panchang data for every day in a year or date range. This is used for calendar generation, festival computation, and bulk data export.

## API

### `compute_year(year, lat, lng, alt, utc_offset)`

Computes Panchang for all 365 (or 366) days from January 1 to December 31.

### `compute_range(start_date, end_date, lat, lng, alt, utc_offset)`

Computes Panchang for an arbitrary date range (inclusive).

## Algorithm

For each day in the range:

1. **Compute midnight JD**: `midnight_jd(year, month, day, utc_offset)` — the Julian Day at local midnight, used as the search start for sunrise.

2. **Compute sunrise and sunset**: `sun::compute_sun_data(year, month, day, lat, lng, alt, utc_offset)` — returns sunrise JD, sunset JD, and day duration.

3. **Compute weekday**: `weekday_from_date(year, month, day)` using Tomohiko Sakamoto's algorithm (avoids timezone issues with JD-based weekday computation).

4. **Compute Panchang**: `panchang::compute(sunrise_jd, weekday)` — returns all 5 elements (Vara, Tithi, Nakshatra, Yoga, Karana) with transition times.

5. **Assemble result**: Combine sun data and Panchang into a `BatchDayResult`.

### Date Iteration

Days are iterated using a simple counter:
- Increment day; if day exceeds `days_in_month(year, month)`, reset to 1 and increment month
- If month exceeds 12, reset to 1 and increment year
- Loop terminates when the midnight JD exceeds the end date JD + 0.5

Leap years are handled by `days_in_month`, which returns 29 for February when `(year % 4 == 0 && year % 100 != 0) || year % 400 == 0`.

### Capacity Pre-allocation

The result vector is pre-allocated based on the estimated number of days: `(end_jd - start_jd).round() + 1`. This avoids reallocations during iteration.

## Performance

Individual computations per day:
- Sunrise: ~29 microseconds
- Full Panchang (5 elements + transitions): ~3.7 milliseconds

**Estimated total for a full year**: ~29us x 365 + ~3.7ms x 365 = ~1.36 seconds

The batch computation is sequential (single-threaded) because the Swiss Ephemeris is not thread-safe. A global mutex (`SWE_LOCK`) serializes access to the ephemeris engine.

## Invariants

The batch output guarantees:
- **Correct day count**: 365 for non-leap years, 366 for leap years
- **Monotonic sunrise JDs**: Each day's sunrise is strictly after the previous day's
- **Valid Panchang ranges**: Tithi 1-30, Nakshatra 1-27, Yoga 1-27, Karana 1-11, Vara 0-6
- **Reasonable day durations**: Between 8 and 16 hours (valid for latitudes within ~60 degrees)
- **Consistency**: Batch results match individual `panchang::compute()` calls for the same date and location
