//! Sankranti (solar ingress) computation.
//!
//! A Sankranti occurs when the Sun's sidereal longitude crosses a
//! multiple of 30 degrees, marking the transition between zodiac signs.
//! There are 12 Sankrantis per year.

use crate::constants::{
    RASHI_NAMES, SANKRANTI_APPROX_DATES, SANKRANTI_NAMES, SANKRANTI_RASHI_INDEX,
    SANKRANTI_TARGET_LONGITUDES,
};
use crate::ephemeris::{self, Planet};
use crate::julian;
use crate::search::find_crossing_forward;

/// Result of a Sankranti computation.
#[derive(Debug, Clone)]
pub struct SankrantiInfo {
    /// 0-11 index in calendar-year order (0 = Makar).
    pub index: u32,
    /// Display name (e.g., "Makar Sankranti").
    pub name: &'static str,
    /// Rashi (zodiac sign) being entered (e.g., "Makara").
    pub rashi: &'static str,
    /// Target sidereal longitude in degrees.
    pub target_longitude: f64,
    /// Exact Julian Day (UT) of the transit.
    pub jd: f64,
}

/// Sun's sidereal longitude at a given Julian Day.
fn sun_sidereal(jd: f64) -> f64 {
    ephemeris::sidereal_longitude(jd, Planet::Sun)
}

/// Find a single Sankranti: the JD when the Sun's sidereal longitude
/// crosses `target_longitude`, searching forward from `approx_start_jd`.
///
/// Uses a 45-day search window (Sun moves ~1°/day, so 30° = ~30 days).
pub fn find_sankranti(target_longitude: f64, approx_start_jd: f64) -> Option<f64> {
    find_crossing_forward(approx_start_jd, target_longitude, &sun_sidereal, 45.0)
}

/// Compute all 12 Sankrantis for a given Gregorian year.
///
/// Returns results in calendar-year order (Makar ~Jan first, Dhanu ~Dec last).
pub fn compute_sankrantis(year: i32) -> Vec<SankrantiInfo> {
    ephemeris::init(None);
    let mut results = Vec::with_capacity(12);

    for i in 0..12 {
        let target = SANKRANTI_TARGET_LONGITUDES[i];
        let (approx_month, approx_day) = SANKRANTI_APPROX_DATES[i];
        let approx_jd = julian::midnight_jd(year, approx_month, approx_day, 0);

        if let Some(jd) = find_sankranti(target, approx_jd) {
            results.push(SankrantiInfo {
                index: i as u32,
                name: SANKRANTI_NAMES[i],
                rashi: RASHI_NAMES[SANKRANTI_RASHI_INDEX[i]],
                target_longitude: target,
                jd,
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_makar_sankranti_2026() {
        ephemeris::init(None);
        let sankrantis = compute_sankrantis(2026);
        assert_eq!(sankrantis.len(), 12);

        let makar = &sankrantis[0];
        assert_eq!(makar.name, "Makar Sankranti");
        assert_eq!(makar.rashi, "Makara");

        // Makar Sankranti 2026 should be around Jan 14
        let dt = julian::jd_to_datetime(makar.jd);
        assert_eq!(dt.year, 2026);
        assert_eq!(dt.month, 1);
        assert!(
            dt.day >= 13 && dt.day <= 15,
            "Makar Sankranti day: {}",
            dt.day
        );
    }

    #[test]
    fn test_all_12_sankrantis_chronological() {
        ephemeris::init(None);
        let sankrantis = compute_sankrantis(2026);
        assert_eq!(sankrantis.len(), 12);

        // JDs should be monotonically increasing
        for i in 1..12 {
            assert!(
                sankrantis[i].jd > sankrantis[i - 1].jd,
                "Sankranti {} (JD {}) should be after {} (JD {})",
                sankrantis[i].name,
                sankrantis[i].jd,
                sankrantis[i - 1].name,
                sankrantis[i - 1].jd
            );
        }
    }

    #[test]
    fn test_sankranti_longitude_at_transit() {
        ephemeris::init(None);
        let sankrantis = compute_sankrantis(2026);

        for s in &sankrantis {
            let actual_long = sun_sidereal(s.jd);
            let target = s.target_longitude;

            // At the transit JD, the Sun should be very close to the target longitude.
            // Allow 0.02 degrees (~72 arcsec) tolerance due to bisection precision.
            let diff = (actual_long - target + 360.0) % 360.0;
            let diff = if diff > 180.0 { 360.0 - diff } else { diff };
            assert!(
                diff < 0.02,
                "{}: Sun at {:.4}° vs target {:.1}° (diff {:.4}°)",
                s.name,
                actual_long,
                target,
                diff
            );
        }
    }

    #[test]
    fn test_sankranti_spacing() {
        ephemeris::init(None);
        let sankrantis = compute_sankrantis(2026);

        for i in 1..12 {
            let gap_days = sankrantis[i].jd - sankrantis[i - 1].jd;
            assert!(
                gap_days > 27.0 && gap_days < 34.0,
                "Gap between {} and {}: {:.1} days (expected 28-33)",
                sankrantis[i - 1].name,
                sankrantis[i].name,
                gap_days
            );
        }
    }
}
