//! Batch computation for full-year panchang.
//!
//! Computes sunrise, sunset, and all 5 Panchang elements for every day
//! in a year or date range. Useful for calendar generation, festival
//! computation, and bulk data export.

use crate::ephemeris;
use crate::julian;
use crate::panchang;
use crate::sun;
use crate::types::PanchangResult;

/// Panchang result for a single day in a batch computation.
#[derive(Debug, Clone)]
pub struct BatchDayResult {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub panchang: PanchangResult,
}

/// Weekday from a local date (0=Sunday … 6=Saturday, Hindu convention).
/// Uses Tomohiko Sakamoto's algorithm.
fn weekday_from_date(year: i32, month: u32, day: u32) -> u32 {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    let d = day as i32;
    let w = (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + d) % 7;
    w.unsigned_abs()
}

/// Days in a Gregorian month.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 => 31,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 30,
    }
}

/// Compute panchang for every day in a Gregorian year.
///
/// Returns 365 (or 366) `BatchDayResult`s, one per day from Jan 1 to Dec 31.
pub fn compute_year(
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Vec<BatchDayResult> {
    compute_range(year, 1, 1, year, 12, 31, lat, lng, alt, utc_offset)
}

/// Compute panchang for a date range (inclusive).
///
/// Iterates day-by-day from `(start_year, start_month, start_day)` to
/// `(end_year, end_month, end_day)`, computing sunrise/sunset and all
/// 5 Panchang elements for each day.
#[allow(clippy::too_many_arguments)]
pub fn compute_range(
    start_year: i32,
    start_month: u32,
    start_day: u32,
    end_year: i32,
    end_month: u32,
    end_day: u32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Vec<BatchDayResult> {
    ephemeris::init(None);

    let end_jd = julian::midnight_jd(end_year, end_month, end_day, utc_offset);

    // Estimate capacity
    let start_jd = julian::midnight_jd(start_year, start_month, start_day, utc_offset);
    let est_days = (end_jd - start_jd).round() as usize + 1;
    let mut results = Vec::with_capacity(est_days);

    let mut year = start_year;
    let mut month = start_month;
    let mut day = start_day;

    loop {
        let midnight = julian::midnight_jd(year, month, day, utc_offset);
        if midnight > end_jd + 0.5 {
            break;
        }

        let sun_data = sun::compute_sun_data(year, month, day, lat, lng, alt, utc_offset);
        let weekday = weekday_from_date(year, month, day);
        let mut panchang_result = panchang::compute(sun_data.sunrise_jd, weekday);
        panchang_result.sun = sun_data;

        results.push(BatchDayResult {
            year,
            month,
            day,
            panchang: panchang_result,
        });

        // Advance to next day
        day += 1;
        if day > days_in_month(year, month) {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }

    results
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const LAT: f64 = 28.6139;
    const LNG: f64 = 77.2090;
    const ALT: f64 = 0.0;
    const IST: i32 = 19800;

    #[test]
    fn test_compute_year_day_count() {
        ephemeris::init(None);
        let results = compute_year(2026, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 365, "2026 is not a leap year");
    }

    #[test]
    fn test_compute_year_leap_year() {
        ephemeris::init(None);
        let results = compute_year(2024, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 366, "2024 is a leap year");
    }

    #[test]
    fn test_compute_year_first_and_last_day() {
        ephemeris::init(None);
        let results = compute_year(2026, LAT, LNG, ALT, IST);

        let first = &results[0];
        assert_eq!(first.year, 2026);
        assert_eq!(first.month, 1);
        assert_eq!(first.day, 1);

        let last = results.last().unwrap();
        assert_eq!(last.year, 2026);
        assert_eq!(last.month, 12);
        assert_eq!(last.day, 31);
    }

    #[test]
    fn test_compute_year_dates_monotonic() {
        ephemeris::init(None);
        let results = compute_year(2026, LAT, LNG, ALT, IST);

        for i in 1..results.len() {
            let prev_jd = results[i - 1].panchang.sun.sunrise_jd;
            let curr_jd = results[i].panchang.sun.sunrise_jd;
            assert!(
                curr_jd > prev_jd,
                "Day {} sunrise ({}) should be after day {} sunrise ({})",
                i + 1,
                curr_jd,
                i,
                prev_jd,
            );
        }
    }

    #[test]
    fn test_compute_year_panchang_valid() {
        ephemeris::init(None);
        let results = compute_year(2026, LAT, LNG, ALT, IST);

        for r in &results {
            let p = &r.panchang;
            assert!(p.tithi.number >= 1 && p.tithi.number <= 30);
            assert!(p.nakshatra.number >= 1 && p.nakshatra.number <= 27);
            assert!(p.yoga.number >= 1 && p.yoga.number <= 27);
            assert!(p.karana.number >= 1 && p.karana.number <= 11);
            assert!(p.vara.number <= 6);
            assert!(p.sun.day_duration_hours > 8.0 && p.sun.day_duration_hours < 16.0);
        }
    }

    #[test]
    fn test_compute_range_partial_month() {
        ephemeris::init(None);
        let results = compute_range(2026, 3, 1, 2026, 3, 31, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 31, "March has 31 days");

        assert_eq!(results[0].day, 1);
        assert_eq!(results[30].day, 31);
    }

    #[test]
    fn test_compute_range_cross_month() {
        ephemeris::init(None);
        let results = compute_range(2026, 2, 25, 2026, 3, 5, LAT, LNG, ALT, IST);
        // Feb 25-28 = 4 days + Mar 1-5 = 5 days = 9 days
        assert_eq!(results.len(), 9);
        assert_eq!(results[0].month, 2);
        assert_eq!(results[0].day, 25);
        assert_eq!(results[8].month, 3);
        assert_eq!(results[8].day, 5);
    }

    #[test]
    fn test_spot_check_matches_individual() {
        ephemeris::init(None);

        // Batch compute March 1
        let batch = compute_range(2026, 3, 1, 2026, 3, 1, LAT, LNG, ALT, IST);
        assert_eq!(batch.len(), 1);
        let batch_day = &batch[0].panchang;

        // Individual compute for same day
        let sun_data = sun::compute_sun_data(2026, 3, 1, LAT, LNG, ALT, IST);
        let weekday = weekday_from_date(2026, 3, 1);
        let mut individual = panchang::compute(sun_data.sunrise_jd, weekday);
        individual.sun = sun_data;

        // Should match exactly
        assert_eq!(batch_day.tithi.number, individual.tithi.number);
        assert_eq!(batch_day.nakshatra.number, individual.nakshatra.number);
        assert_eq!(batch_day.yoga.number, individual.yoga.number);
        assert_eq!(batch_day.karana.number, individual.karana.number);
        assert_eq!(batch_day.vara.number, individual.vara.number);
        assert!(
            (batch_day.sun.sunrise_jd - individual.sun.sunrise_jd).abs() < 0.0001,
            "Sunrise JDs should match"
        );
    }

    #[test]
    fn test_weekday_from_date() {
        // January 1, 2026 = Thursday (4)
        assert_eq!(weekday_from_date(2026, 1, 1), 4);
        // March 1, 2026 = Sunday (0)
        assert_eq!(weekday_from_date(2026, 3, 1), 0);
        // February 24, 2026 = Tuesday (2)
        assert_eq!(weekday_from_date(2026, 2, 24), 2);
    }
}
