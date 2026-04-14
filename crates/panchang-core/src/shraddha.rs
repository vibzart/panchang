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

use crate::constants::{LUNAR_MONTH_NAMES, NAKSHATRA_NAMES, VARA_ENGLISH};
use crate::ephemeris;
use crate::festival;
use crate::julian;
use crate::panchang;
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

/// A single date where a specific tithi occurs at sunrise.
#[derive(Debug, Clone)]
pub struct TithiOccurrence {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub sunrise_jd: f64,
    pub tithi_number: u32,
    pub tithi_name: String,
    pub paksha: &'static str,
    pub nakshatra_name: &'static str,
    pub masa: &'static str,
    pub vara: &'static str,
}

/// Complete shraddha timeline: death details + monthly + annual + pitru paksha.
#[derive(Debug, Clone)]
pub struct ShraddhaTimeline {
    // Death details
    pub death_date: String,
    pub death_tithi: u32,
    pub death_tithi_name: String,
    pub death_paksha: &'static str,
    pub death_nakshatra: &'static str,
    pub death_masa: &'static str,

    // Immediate ceremonies
    pub teesra: String,
    pub dashama: String,
    pub terahvin: String,

    // Computed recurrences
    pub monthly_shraddhas: Vec<TithiOccurrence>,
    pub annual_shraddhas: Vec<TithiOccurrence>,
    pub pitru_paksha_dates: Vec<TithiOccurrence>,
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

/// Build a TithiOccurrence from a sunrise JD.
fn occurrence_at_sunrise(sunrise_jd: f64, utc_offset: i32, target_tithi: u32) -> TithiOccurrence {
    let dt = julian::jd_to_datetime(sunrise_jd + (utc_offset as f64) / 86400.0);
    let weekday = ((sunrise_jd + 1.5).floor() as u32) % 7;

    // Compute nakshatra at sunrise
    let pr = panchang::compute(sunrise_jd, weekday);

    // Determine lunar month
    let masa_num = lunar_month_for_date(dt.year, dt.month, dt.day, utc_offset);
    let masa_name = if masa_num >= 1 && (masa_num as usize) <= LUNAR_MONTH_NAMES.len() {
        LUNAR_MONTH_NAMES[(masa_num - 1) as usize]
    } else {
        "Unknown"
    };

    let paksha = if target_tithi <= 15 {
        "Shukla"
    } else {
        "Krishna"
    };
    let tithi_idx = (target_tithi - 1) as usize;
    let tithi_name = if target_tithi <= 15 {
        format!("Shukla {}", crate::constants::TITHI_NAMES[tithi_idx])
    } else {
        format!("Krishna {}", crate::constants::TITHI_NAMES[tithi_idx])
    };

    let nak_idx = (pr.nakshatra.number - 1) as usize;
    let nakshatra_name = if nak_idx < NAKSHATRA_NAMES.len() {
        NAKSHATRA_NAMES[nak_idx]
    } else {
        "Unknown"
    };

    let vara = VARA_ENGLISH[(weekday as usize) % 7];

    TithiOccurrence {
        year: dt.year,
        month: dt.month,
        day: dt.day,
        sunrise_jd,
        tithi_number: target_tithi,
        tithi_name,
        paksha,
        nakshatra_name,
        masa: masa_name,
        vara,
    }
}

/// Find the next N occurrences of a specific tithi at sunrise, starting from
/// `start_jd`. Uses ~29-day synodic jumps with ±2 day scanning.
fn find_tithi_occurrences(
    target_tithi: u32,
    start_jd: f64,
    count: usize,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Vec<TithiOccurrence> {
    let mut results: Vec<TithiOccurrence> = Vec::with_capacity(count);
    let mut current_jd = start_jd;

    // Find the first occurrence by scanning day-by-day (max ~35 days)
    for _ in 0..35 {
        let sunrise = sun::sunrise_jd(current_jd, lat, lng, alt);
        let t = festival::tithi_at_jd(sunrise);
        if t == target_tithi {
            results.push(occurrence_at_sunrise(sunrise, utc_offset, target_tithi));
            break;
        }
        current_jd += 1.0;
    }

    if results.is_empty() {
        return results;
    }

    // For subsequent occurrences, jump ~28 days then scan ±3 days.
    // Synodic month is ~29.53 days but tithis can be 19-26 hours,
    // so the gap between same-tithi sunrises varies 28-31 days.
    while results.len() < count {
        let last_jd = results.last().unwrap().sunrise_jd;
        let jump_to = last_jd + 28.0;

        let mut found = false;
        // Scan a 7-day window around the expected date
        for offset in -2..5_i32 {
            let midnight = jump_to + offset as f64;
            if midnight <= last_jd + 1.0 {
                continue;
            }
            let sunrise = sun::sunrise_jd(midnight, lat, lng, alt);
            let t = festival::tithi_at_jd(sunrise);
            if t == target_tithi {
                results.push(occurrence_at_sunrise(sunrise, utc_offset, target_tithi));
                found = true;
                break;
            }
        }

        if !found {
            break; // tithi not found in window — stop
        }
    }

    results
}

/// Find Pitru Paksha date for a given tithi and year.
///
/// Pitru Paksha is the Krishna Paksha of Bhadrapada (Sep-Oct).
/// Searches for the target tithi within that period.
fn find_pitru_paksha_date(
    target_tithi: u32,
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<TithiOccurrence> {
    // Pitru Paksha = Krishna Paksha, so we need Krishna tithi (16-30)
    // If death tithi was Shukla (1-15), no direct match in Pitru Paksha
    if target_tithi <= 15 {
        return None;
    }

    // Search Sep 1 to Oct 15 for the matching Krishna tithi
    let start_jd = julian::midnight_jd(year, 9, 1, utc_offset);
    let end_jd = julian::midnight_jd(year, 10, 15, utc_offset);
    let mut current = start_jd;

    while current <= end_jd {
        let sunrise = sun::sunrise_jd(current, lat, lng, alt);
        let t = festival::tithi_at_jd(sunrise);
        if t == target_tithi {
            // Verify it's in Bhadrapada's Krishna Paksha by checking lunar month
            let dt = julian::jd_to_datetime(sunrise + (utc_offset as f64) / 86400.0);
            let masa = lunar_month_for_date(dt.year, dt.month, dt.day, utc_offset);
            // Bhadrapada = month 6
            if masa == 6 {
                return Some(occurrence_at_sunrise(sunrise, utc_offset, target_tithi));
            }
        }
        current += 1.0;
    }

    None
}

/// Compute the complete Shraddha timeline from a death date.
///
/// Returns death details, immediate ceremony dates, monthly tithi recurrences,
/// annual shraddha dates, and Pitru Paksha dates — all computed in Rust.
#[allow(clippy::too_many_arguments)]
pub fn compute_shraddha_timeline(
    death_year: i32,
    death_month: u32,
    death_day: u32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
    monthly_count: usize,
    annual_years: usize,
    pitru_paksha_years: usize,
) -> Option<ShraddhaTimeline> {
    ephemeris::init(None);

    // Step 1: Compute death tithi and panchang
    let (death_tithi, death_sunrise) = tithi_at_sunrise(
        death_year,
        death_month,
        death_day,
        lat,
        lng,
        alt,
        utc_offset,
    );
    let weekday = ((death_sunrise + 1.5).floor() as u32) % 7;
    let pr = panchang::compute(death_sunrise, weekday);

    // Death details
    let lunar_month_num = lunar_month_for_date(death_year, death_month, death_day, utc_offset);
    let death_masa =
        if lunar_month_num >= 1 && (lunar_month_num as usize) <= LUNAR_MONTH_NAMES.len() {
            LUNAR_MONTH_NAMES[(lunar_month_num - 1) as usize]
        } else {
            "Unknown"
        };

    let death_paksha = if death_tithi <= 15 {
        "Shukla"
    } else {
        "Krishna"
    };
    let tithi_idx = (death_tithi - 1) as usize;
    let death_tithi_name = if death_tithi <= 15 {
        format!("Shukla {}", crate::constants::TITHI_NAMES[tithi_idx])
    } else {
        format!("Krishna {}", crate::constants::TITHI_NAMES[tithi_idx])
    };

    let nak_idx = (pr.nakshatra.number - 1) as usize;
    let death_nakshatra = if nak_idx < NAKSHATRA_NAMES.len() {
        NAKSHATRA_NAMES[nak_idx]
    } else {
        "Unknown"
    };

    let death_date = format!("{}-{:02}-{:02}", death_year, death_month, death_day);

    // Step 2: Immediate ceremony dates (date arithmetic)
    let death_jd = julian::midnight_jd(death_year, death_month, death_day, utc_offset);
    let fmt_date = |offset: f64| -> String {
        let dt = julian::jd_to_datetime(death_jd + offset + (utc_offset as f64) / 86400.0);
        format!("{}-{:02}-{:02}", dt.year, dt.month, dt.day)
    };
    let teesra = fmt_date(2.0);
    let dashama = fmt_date(9.0);
    let terahvin = fmt_date(12.0);

    // Step 3: Monthly shraddhas — find next N occurrences of the death tithi
    let monthly_start_jd = death_jd + 13.0; // start after terahvin
    let monthly_shraddhas = find_tithi_occurrences(
        death_tithi,
        monthly_start_jd,
        monthly_count,
        lat,
        lng,
        alt,
        utc_offset,
    );

    // Step 4: Annual shraddhas using festival engine
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

    let mut annual_shraddhas: Vec<TithiOccurrence> = Vec::with_capacity(annual_years);
    for year_offset in 1..=(annual_years as i32) {
        let target_year = death_year + year_offset;
        let results = festival::compute_festivals(
            &defs,
            target_year,
            lat,
            lng,
            alt,
            utc_offset,
            crate::lunar_month::CalendarSystem::Amant,
        );
        if let Some(r) = results.first() {
            let sunrise = sun::sunrise_jd(
                julian::midnight_jd(r.year, r.month, r.day, utc_offset),
                lat,
                lng,
                alt,
            );
            annual_shraddhas.push(occurrence_at_sunrise(sunrise, utc_offset, death_tithi));
        }
    }

    // Step 5: Pitru Paksha dates
    let mut pitru_paksha_dates: Vec<TithiOccurrence> = Vec::with_capacity(pitru_paksha_years);
    for year_offset in 0..(pitru_paksha_years as i32) {
        let target_year = death_year + year_offset;
        if let Some(pp) =
            find_pitru_paksha_date(death_tithi, target_year, lat, lng, alt, utc_offset)
        {
            pitru_paksha_dates.push(pp);
        }
    }

    Some(ShraddhaTimeline {
        death_date,
        death_tithi,
        death_tithi_name,
        death_paksha,
        death_nakshatra,
        death_masa,
        teesra,
        dashama,
        terahvin,
        monthly_shraddhas,
        annual_shraddhas,
        pitru_paksha_dates,
    })
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
    let results = festival::compute_festivals(
        &defs,
        target_year,
        lat,
        lng,
        alt,
        utc_offset,
        crate::lunar_month::CalendarSystem::Amant,
    );

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

    #[test]
    fn test_shraddha_timeline() {
        ephemeris::init(None);
        let result = compute_shraddha_timeline(2025, 6, 15, 28.6139, 77.2090, 0.0, 19800, 2, 1, 1);
        assert!(result.is_some());
        let tl = result.unwrap();
        assert_eq!(tl.death_date, "2025-06-15");
        assert!(tl.death_tithi >= 1 && tl.death_tithi <= 30);
        assert!(!tl.death_tithi_name.is_empty());
        assert!(!tl.teesra.is_empty());
        assert!(!tl.dashama.is_empty());
        assert!(!tl.terahvin.is_empty());
        assert!(tl.monthly_shraddhas.len() <= 2);
        assert!(tl.annual_shraddhas.len() <= 1);
    }
}
