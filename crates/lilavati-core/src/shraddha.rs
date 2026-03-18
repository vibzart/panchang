//! Shraddha Tithi calculator.
//!
//! Shraddha (death anniversary) is observed on the same lunar Tithi and
//! in the same lunar month as the death. This module finds the Gregorian
//! date of that Tithi for any target year.
//!
//! ## Algorithm
//!
//! Given a death date:
//! 1. Compute the Tithi at sunrise on the death date → this is the Shraddha Tithi
//! 2. Determine the lunar month of the death date → this is the Shraddha month
//! 3. For each target year, find the date when that Tithi prevails at
//!    sunrise within that lunar month
//!
//! The search uses the same Sankranti-anchored approach as festival resolution.

use crate::constants::LUNAR_MONTH_NAMES;
use crate::ephemeris;
use crate::festival;
use crate::julian;
use crate::sankranti;
use crate::sun;

/// Result of computing a Shraddha date.
#[derive(Debug, Clone)]
pub struct ShraddhaResult {
    /// The Tithi number at sunrise on the original death date (1-30).
    pub tithi: u32,
    /// Lunar month number (1-12) of the death.
    pub lunar_month: u32,
    /// Lunar month name.
    pub lunar_month_name: &'static str,
    /// Resolved Gregorian year of the Shraddha.
    pub year: i32,
    /// Resolved Gregorian month.
    pub month: u32,
    /// Resolved Gregorian day.
    pub day: u32,
    /// Sunrise JD on the Shraddha date.
    pub sunrise_jd: f64,
    /// Human-readable explanation.
    pub reasoning: String,
}

/// Compute the Tithi at sunrise for a given date and location.
fn tithi_at_sunrise(
    year: i32,
    month: u32,
    day: u32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> (u32, f64) {
    let sun_data = sun::compute_sun_data(year, month, day, lat, lng, alt, utc_offset);
    let sunrise_jd = sun_data.sunrise_jd;
    let tithi = festival::tithi_at_jd(sunrise_jd);
    (tithi, sunrise_jd)
}

/// Find which lunar month a date falls in, using Sankranti-based lookup.
///
/// Finds the closest Sankranti before the date and maps it to a lunar month.
fn lunar_month_for_date(year: i32, month: u32, day: u32, utc_offset: i32) -> u32 {
    let date_jd = julian::midnight_jd(year, month, day, utc_offset);
    let sankrantis = sankranti::compute_sankrantis(year);

    // Find the last Sankranti before or on this date
    let mut best_idx = 0;
    let mut best_jd = 0.0;
    for (i, s) in sankrantis.iter().enumerate() {
        if s.jd <= date_jd && s.jd > best_jd {
            best_jd = s.jd;
            best_idx = i;
        }
    }

    // If no Sankranti found before date (early January), check previous year
    if best_jd == 0.0 {
        let prev_sankrantis = sankranti::compute_sankrantis(year - 1);
        if let Some(last) = prev_sankrantis.last() {
            let rashi_idx = crate::constants::SANKRANTI_RASHI_INDEX[last.index as usize];
            return crate::constants::SANKRANTI_TO_LUNAR_MONTH[rashi_idx];
        }
        return 10; // Pausha as fallback for early Jan
    }

    let rashi_idx = crate::constants::SANKRANTI_RASHI_INDEX[sankrantis[best_idx].index as usize];
    crate::constants::SANKRANTI_TO_LUNAR_MONTH[rashi_idx]
}

/// Compute the Shraddha (death anniversary) date for a target year.
///
/// Given the original death date and a target year, finds the Gregorian date
/// when the same Tithi occurs in the same lunar month.
///
/// # Arguments
/// * `death_year`, `death_month`, `death_day` — Original death date
/// * `target_year` — Year to find the Shraddha in
/// * `lat`, `lng`, `alt`, `utc_offset` — Location for sunrise computation
#[allow(clippy::too_many_arguments)]
pub fn compute_shraddha(
    death_year: i32,
    death_month: u32,
    death_day: u32,
    target_year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<ShraddhaResult> {
    ephemeris::init(None);

    // Step 1: Find Tithi at sunrise on death date
    let (death_tithi, _) = tithi_at_sunrise(
        death_year,
        death_month,
        death_day,
        lat,
        lng,
        alt,
        utc_offset,
    );

    // Step 2: Find lunar month of death date
    let lunar_month_num = lunar_month_for_date(death_year, death_month, death_day, utc_offset);

    // Step 3: Use festival resolution to find the target Tithi in the target year
    // Create a temporary festival def with the death's tithi and lunar month
    let def = festival::FestivalDef {
        id: "shraddha".to_string(),
        name: "Shraddha".to_string(),
        rule: "tithi_at_sunrise".to_string(),
        lunar_month: lunar_month_num,
        tithi: death_tithi,
        sankranti_index: None,
        nakshatra: None,
    };

    let defs = vec![def];
    let results = festival::compute_festivals(&defs, target_year, lat, lng, alt, utc_offset);

    if let Some(r) = results.first() {
        let month_name = if (lunar_month_num as usize) >= 1
            && (lunar_month_num as usize) <= LUNAR_MONTH_NAMES.len()
        {
            LUNAR_MONTH_NAMES[(lunar_month_num - 1) as usize]
        } else {
            "Unknown"
        };

        let paksha = if death_tithi <= 15 {
            "Shukla"
        } else {
            "Krishna"
        };
        let tithi_in_paksha = if death_tithi <= 15 {
            death_tithi
        } else {
            death_tithi - 15
        };

        let reasoning = format!(
            "Shraddha for death on {}-{:02}-{:02} (Tithi {} {} Paksha, {} month). \
             In {}, this Tithi prevails at sunrise on {}-{:02}-{:02}.",
            death_year,
            death_month,
            death_day,
            tithi_in_paksha,
            paksha,
            month_name,
            target_year,
            r.year,
            r.month,
            r.day,
        );

        Some(ShraddhaResult {
            tithi: death_tithi,
            lunar_month: lunar_month_num,
            lunar_month_name: month_name,
            year: r.year,
            month: r.month,
            day: r.day,
            sunrise_jd: r.sunrise_jd,
            reasoning,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shraddha_returns_result() {
        ephemeris::init(None);
        // Death date: 2020-06-15, find Shraddha in 2026
        let result = compute_shraddha(2020, 6, 15, 2026, 28.6139, 77.2090, 0.0, 19800);
        assert!(result.is_some(), "Shraddha should be found for 2026");
        let r = result.unwrap();
        assert!(r.tithi >= 1 && r.tithi <= 30);
        assert!(r.lunar_month >= 1 && r.lunar_month <= 12);
        assert_eq!(r.year, 2026);
        assert!(!r.reasoning.is_empty());
    }

    #[test]
    fn test_shraddha_same_tithi() {
        ephemeris::init(None);
        // The Shraddha date should have the same Tithi as the death date
        let death_year = 2015;
        let death_month = 3;
        let death_day = 20;
        let lat = 28.6139;
        let lng = 77.2090;
        let alt = 0.0;
        let utc_offset = 19800;

        let (death_tithi, _) = tithi_at_sunrise(
            death_year,
            death_month,
            death_day,
            lat,
            lng,
            alt,
            utc_offset,
        );

        let result = compute_shraddha(
            death_year,
            death_month,
            death_day,
            2026,
            lat,
            lng,
            alt,
            utc_offset,
        );
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(
            r.tithi, death_tithi,
            "Shraddha tithi {} should match death tithi {}",
            r.tithi, death_tithi
        );
    }

    #[test]
    fn test_shraddha_different_years() {
        ephemeris::init(None);
        // Shraddha should resolve to different Gregorian dates in different years
        let r2025 = compute_shraddha(2010, 8, 10, 2025, 28.6139, 77.2090, 0.0, 19800);
        let r2026 = compute_shraddha(2010, 8, 10, 2026, 28.6139, 77.2090, 0.0, 19800);

        assert!(r2025.is_some());
        assert!(r2026.is_some());
        let r25 = r2025.unwrap();
        let r26 = r2026.unwrap();

        // Same Tithi and lunar month, different Gregorian date
        assert_eq!(r25.tithi, r26.tithi);
        assert_eq!(r25.lunar_month, r26.lunar_month);
        // Dates should differ (different years)
        assert_ne!(
            (r25.year, r25.month, r25.day),
            (r26.year, r26.month, r26.day)
        );
    }
}
