//! Lunar month computation for Amant and Purnimant systems.
//!
//! A lunar month is named after the Sankranti that falls within its boundaries.
//! - Amant (South Bhārat): boundaries are consecutive Amavasyas (New Moon)
//! - Purnimant (North Bhārat): boundaries are consecutive Purnimas (Full Moon)
//!
//! An Adhik Maas (intercalary month) occurs when NO Sankranti falls within
//! a lunar month. A Kshaya Maas occurs when TWO Sankrantis fall within one
//! lunar month (extremely rare).

use crate::angles::normalize;
use crate::constants::{LUNAR_MONTH_NAMES, SANKRANTI_RASHI_INDEX, SANKRANTI_TO_LUNAR_MONTH};
use crate::ephemeris::{self, Planet};
use crate::julian;
use crate::sankranti;
use crate::search::find_crossing_forward;

/// Calendar system variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalendarSystem {
    /// South Bhāratīya: month boundary at Amavasya (New Moon).
    Amant,
    /// North Bhāratīya: month boundary at Purnima (Full Moon).
    Purnimant,
}

/// Information about a single lunar month.
#[derive(Debug, Clone)]
pub struct LunarMonthInfo {
    /// 1-12 (Chaitra=1, ..., Phalguna=12). 0 if unknown.
    pub number: u32,
    /// Month name (e.g., "Chaitra").
    pub name: &'static str,
    /// Whether this is an Adhik (intercalary) month.
    pub is_adhik: bool,
    /// Whether this is a Kshaya (compressed) month — two Sankrantis within.
    pub is_kshaya: bool,
    /// JD of the month boundary start (Amavasya or Purnima).
    pub start_jd: f64,
    /// JD of the month boundary end.
    pub end_jd: f64,
}

/// Tithi angle: Moon tropical − Sun tropical, normalized to [0, 360).
/// Amavasya (New Moon) = 0°, Purnima (Full Moon) = 180°.
fn tithi_angle(jd: f64) -> f64 {
    let moon = ephemeris::tropical_longitude(jd, Planet::Moon);
    let sun = ephemeris::tropical_longitude(jd, Planet::Sun);
    normalize(moon - sun)
}

/// Find consecutive Amavasyas (tithi angle crossing 0°) spanning a year.
///
/// Searches from ~1 month before Jan 1 to ~1 month after Dec 31.
/// Returns ~13 JDs.
pub fn find_amavasyas(year: i32) -> Vec<f64> {
    ephemeris::init(None);

    // Start searching from ~Dec 1 of previous year
    let start_jd = julian::midnight_jd(year - 1, 12, 1, 0);
    // End ~Jan 31 of next year
    let end_jd = julian::midnight_jd(year + 1, 1, 31, 0);

    let mut results = Vec::with_capacity(14);
    let mut jd = start_jd;

    while jd < end_jd {
        if let Some(crossing_jd) = find_crossing_forward(jd, 0.0, &tithi_angle, 35.0) {
            results.push(crossing_jd);
            // Jump forward ~25 days to find the next one (~29.5 day lunar cycle)
            jd = crossing_jd + 25.0;
        } else {
            // No crossing found within 35 days — jump ahead
            jd += 30.0;
        }
    }

    results
}

/// Find consecutive Purnimas (tithi angle crossing 180°) spanning a year.
///
/// Returns ~13 JDs.
pub fn find_purnimas(year: i32) -> Vec<f64> {
    ephemeris::init(None);

    let start_jd = julian::midnight_jd(year - 1, 12, 1, 0);
    let end_jd = julian::midnight_jd(year + 1, 1, 31, 0);

    let mut results = Vec::with_capacity(14);
    let mut jd = start_jd;

    while jd < end_jd {
        if let Some(crossing_jd) = find_crossing_forward(jd, 180.0, &tithi_angle, 35.0) {
            results.push(crossing_jd);
            jd = crossing_jd + 25.0;
        } else {
            jd += 30.0;
        }
    }

    results
}

/// Compute all lunar months for a year in the given calendar system.
///
/// Algorithm:
/// 1. Compute all 12 Sankrantis for the year (and adjacent years for boundary months).
/// 2. Compute all boundary points (Amavasyas or Purnimas).
/// 3. For each consecutive pair of boundaries, find which Sankranti falls within.
/// 4. That Sankranti's Rashi names the month.
/// 5. If no Sankranti falls within: Adhik Maas.
/// 6. If two Sankrantis fall within: Kshaya Maas.
pub fn compute_lunar_months(year: i32, system: CalendarSystem) -> Vec<LunarMonthInfo> {
    ephemeris::init(None);

    // Get boundaries based on calendar system
    let boundaries = match system {
        CalendarSystem::Amant => find_amavasyas(year),
        CalendarSystem::Purnimant => find_purnimas(year),
    };

    if boundaries.len() < 2 {
        return Vec::new();
    }

    // Get Sankrantis for this year and adjacent years (for boundary months)
    let mut all_sankrantis = sankranti::compute_sankrantis(year - 1);
    all_sankrantis.extend(sankranti::compute_sankrantis(year));
    all_sankrantis.extend(sankranti::compute_sankrantis(year + 1));
    // Sort by JD
    all_sankrantis.sort_by(|a, b| a.jd.partial_cmp(&b.jd).unwrap());

    let mut months = Vec::with_capacity(14);

    // For each consecutive pair of boundaries, determine the month
    for i in 0..boundaries.len() - 1 {
        let start_jd = boundaries[i];
        let end_jd = boundaries[i + 1];

        // Find which Sankrantis fall within this boundary pair
        let sankrantis_within: Vec<_> = all_sankrantis
            .iter()
            .filter(|s| s.jd > start_jd && s.jd <= end_jd)
            .collect();

        match sankrantis_within.len() {
            0 => {
                // Adhik Maas — no Sankranti within this month.
                // Name it after the NEXT month's Sankranti.
                let next_sankranti = all_sankrantis.iter().find(|s| s.jd > end_jd);
                let (number, name) = if let Some(ns) = next_sankranti {
                    let rashi_idx = SANKRANTI_RASHI_INDEX[ns.index as usize];
                    let month_num = SANKRANTI_TO_LUNAR_MONTH[rashi_idx];
                    (month_num, LUNAR_MONTH_NAMES[(month_num - 1) as usize])
                } else {
                    (0, "Unknown")
                };
                months.push(LunarMonthInfo {
                    number,
                    name,
                    is_adhik: true,
                    is_kshaya: false,
                    start_jd,
                    end_jd,
                });
            }
            1 => {
                // Normal month — one Sankranti determines the name.
                let s = sankrantis_within[0];
                let rashi_idx = SANKRANTI_RASHI_INDEX[s.index as usize];
                let month_num = SANKRANTI_TO_LUNAR_MONTH[rashi_idx];
                let name = LUNAR_MONTH_NAMES[(month_num - 1) as usize];
                months.push(LunarMonthInfo {
                    number: month_num,
                    name,
                    is_adhik: false,
                    is_kshaya: false,
                    start_jd,
                    end_jd,
                });
            }
            _ => {
                // Kshaya Maas — two Sankrantis within one month (very rare).
                // Name after the first Sankranti.
                let s = sankrantis_within[0];
                let rashi_idx = SANKRANTI_RASHI_INDEX[s.index as usize];
                let month_num = SANKRANTI_TO_LUNAR_MONTH[rashi_idx];
                let name = LUNAR_MONTH_NAMES[(month_num - 1) as usize];
                months.push(LunarMonthInfo {
                    number: month_num,
                    name,
                    is_adhik: false,
                    is_kshaya: true,
                    start_jd,
                    end_jd,
                });
            }
        }
    }

    // Filter to months that overlap with the requested year
    let year_start_jd = julian::midnight_jd(year, 1, 1, 0);
    let year_end_jd = julian::midnight_jd(year, 12, 31, 0) + 1.0;
    months.retain(|m| m.end_jd > year_start_jd && m.start_jd < year_end_jd);

    months
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_amavasyas_count() {
        ephemeris::init(None);
        let amavasyas = find_amavasyas(2026);
        // There should be 13-15 Amavasyas spanning the year
        // (we search from Dec prev year to Jan next year)
        assert!(
            amavasyas.len() >= 13 && amavasyas.len() <= 16,
            "Got {} Amavasyas, expected 13-16",
            amavasyas.len()
        );
    }

    #[test]
    fn test_find_purnimas_count() {
        ephemeris::init(None);
        let purnimas = find_purnimas(2026);
        assert!(
            purnimas.len() >= 13 && purnimas.len() <= 16,
            "Got {} Purnimas, expected 13-16",
            purnimas.len()
        );
    }

    #[test]
    fn test_amavasya_is_new_moon() {
        ephemeris::init(None);
        let amavasyas = find_amavasyas(2026);
        for &jd in &amavasyas {
            let angle = tithi_angle(jd);
            // At Amavasya, tithi angle should be very close to 0° (or 360°)
            let dist = if angle > 180.0 { 360.0 - angle } else { angle };
            assert!(
                dist < 1.0,
                "Amavasya at JD {}: tithi angle = {:.2}° (expected ~0°)",
                jd,
                angle
            );
        }
    }

    #[test]
    fn test_purnima_is_full_moon() {
        ephemeris::init(None);
        let purnimas = find_purnimas(2026);
        for &jd in &purnimas {
            let angle = tithi_angle(jd);
            // At Purnima, tithi angle should be very close to 180°
            let dist = (angle - 180.0).abs();
            assert!(
                dist < 1.0,
                "Purnima at JD {}: tithi angle = {:.2}° (expected ~180°)",
                jd,
                angle
            );
        }
    }

    #[test]
    fn test_amant_lunar_months_cover_year() {
        ephemeris::init(None);
        let months = compute_lunar_months(2026, CalendarSystem::Amant);
        // Should have 12-13 months overlapping with 2026
        assert!(
            months.len() >= 12 && months.len() <= 14,
            "Got {} months, expected 12-14",
            months.len()
        );

        // Check that consecutive months have no gaps
        for i in 1..months.len() {
            let gap = (months[i].start_jd - months[i - 1].end_jd).abs();
            assert!(
                gap < 0.01,
                "Gap between month {} and {}: {:.4} JD",
                months[i - 1].name,
                months[i].name,
                gap
            );
        }
    }

    #[test]
    fn test_purnimant_lunar_months() {
        ephemeris::init(None);
        let months = compute_lunar_months(2026, CalendarSystem::Purnimant);
        assert!(
            months.len() >= 12 && months.len() <= 14,
            "Got {} Purnimant months, expected 12-14",
            months.len()
        );
    }

    #[test]
    fn test_all_month_names_valid() {
        ephemeris::init(None);
        let months = compute_lunar_months(2026, CalendarSystem::Amant);
        for m in &months {
            assert!(
                LUNAR_MONTH_NAMES.contains(&m.name),
                "Invalid month name: {}",
                m.name
            );
        }
    }
}
