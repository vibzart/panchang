//! Lagna (sidereal ascendant) and house-cusp computation.
//!
//! Wraps Swiss Ephemeris `swe_houses_ex` to produce the Bhāratīya
//! ascendant in the Lahiri sidereal frame, plus the 12 bhāva cusps in
//! a chosen house system (Placidus by default).
//!
//! In Vedic Jyotish the ascendant *is* the Lagna — the rashi rising on
//! the eastern horizon at the moment of birth — and is the primary
//! reference point for the entire D-1 chart.

use crate::angles::normalize;
use crate::constants::RASHI_NAMES;
use crate::ephemeris;
use crate::ffi;

/// House systems supported by the lagna module.
///
/// The variants map to the ASCII byte that Swiss Ephemeris' `hsys`
/// argument expects; this enum exists so callers don't have to
/// remember the magic letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HouseSystem {
    /// Placidus — modern Bhāva Chalit default.
    #[default]
    Placidus,
    /// Whole-sign — each rashi is a full bhāva.
    WholeSign,
    /// Equal house from the ascendant (every 30°).
    Equal,
    /// Porphyry / Sripati — equal-arc trisection between angles.
    Porphyry,
}

impl HouseSystem {
    fn as_int(self) -> i32 {
        match self {
            HouseSystem::Placidus => ffi::SE_HSYS_PLACIDUS,
            HouseSystem::WholeSign => ffi::SE_HSYS_WHOLE_SIGN,
            HouseSystem::Equal => ffi::SE_HSYS_EQUAL,
            HouseSystem::Porphyry => ffi::SE_HSYS_PORPHYRY,
        }
    }
}

/// Result of a Lagna computation at a given (jd, lat, lng).
///
/// All angular fields are sidereal (Lahiri ayanamsa) and given in degrees.
#[derive(Debug, Clone, PartialEq)]
pub struct LagnaInfo {
    /// Full sidereal longitude of the ascendant in [0, 360).
    pub ascendant_longitude: f64,

    /// Rashi index 0..=11 — 0 = Mesha, 11 = Meena.
    pub rashi: u8,

    /// Sanskrit rashi name (e.g. `"Vrishabha"`).
    pub rashi_name: &'static str,

    /// Degree within the rashi, in [0, 30).
    pub degree_in_rashi: f64,

    /// Midheaven (MC) sidereal longitude in [0, 360).
    pub mc_longitude: f64,

    /// 12 bhāva (house) cusps in sidereal degrees, indexed 0..=11
    /// where index 0 is the 1st bhāva (Lagna bhava).
    pub bhava_cusps: [f64; 12],

    /// House system used to compute `bhava_cusps`.
    pub house_system: HouseSystem,

    /// Lahiri ayanamsa value (degrees) at this jd. Surfaced so the
    /// `/transparency` page can show readers exactly which ayanamsa
    /// epoch their chart was computed against.
    pub ayanamsa: f64,
}

/// One rashi-rising window during a single day.
#[derive(Debug, Clone, PartialEq)]
pub struct LagnaWindow {
    /// Rashi index 0..=11 (0 = Mesha).
    pub rashi: u8,
    /// Sanskrit rashi name.
    pub rashi_name: &'static str,
    /// JD (UT) when this rashi began rising on the eastern horizon.
    pub start_jd: f64,
    /// JD (UT) when this rashi finished rising (next rashi began).
    pub end_jd: f64,
}

/// Compute the Lagna and house cusps for a birth chart.
///
/// * `jd` — Julian Day in UT (must already account for timezone).
/// * `lat` — geographic latitude in degrees, north positive.
/// * `lng` — geographic longitude in degrees, east positive.
/// * `system` — which house-cusp definition to use.
///
/// All output longitudes are sidereal (Lahiri), so `ascendant_longitude`
/// is directly comparable to `ephemeris::sidereal_longitude(jd, planet)`.
pub fn compute(jd: f64, lat: f64, lng: f64, system: HouseSystem) -> LagnaInfo {
    // Initialize Swiss Ephemeris with Lahiri ayanamsa. Idempotent.
    ephemeris::init(None);

    // Re-assert Lahiri sidereal mode on every call. Swiss Ephemeris caches
    // the ayanamsa epoch on the first sidereal computation, and the cache
    // is keyed on the call rather than the JD — so without this reset, a
    // second `compute()` call would silently apply the *first* call's
    // ayanamsa value (drifting subsequent kundalis by 50"/yr × age-gap).
    unsafe {
        ffi::swe_set_sid_mode(ffi::SE_SIDM_LAHIRI, 0.0, 0.0);
    }

    // Swiss Ephemeris writes 13 cusp slots (index 0 unused) and at least
    // 10 ascmc slots. We allocate the documented sizes.
    let mut cusps = [0.0f64; 13];
    let mut ascmc = [0.0f64; 10];

    unsafe {
        ffi::swe_houses_ex(
            jd,
            ffi::SEFLG_SIDEREAL,
            lat,
            lng,
            system.as_int(),
            cusps.as_mut_ptr(),
            ascmc.as_mut_ptr(),
        );
    }

    let asc = normalize(ascmc[0]);
    let mc = normalize(ascmc[1]);

    let rashi = (asc / 30.0).floor() as u8 % 12;
    let degree_in_rashi = asc - (rashi as f64) * 30.0;

    let mut bhava_cusps = [0.0f64; 12];
    for (i, slot) in bhava_cusps.iter_mut().enumerate() {
        *slot = normalize(cusps[i + 1]);
    }

    LagnaInfo {
        ascendant_longitude: asc,
        rashi,
        rashi_name: RASHI_NAMES[rashi as usize],
        degree_in_rashi,
        mc_longitude: mc,
        bhava_cusps,
        house_system: system,
        ayanamsa: ephemeris::ayanamsa(jd),
    }
}

/// Compute the Lagna-rising windows that span a single day.
///
/// Given any `jd_start` (typically local sunrise), returns up to 12
/// `LagnaWindow`s — one per rashi — for the next 24-hour period.
///
/// This is the primitive the Khona intake flow uses for birth-time
/// uncertainty: when a user knows their birth only "to the half-day"
/// or "in the morning", the UI shows them which lagnas were rising
/// during that window so the chart can be gated accordingly.
///
/// ### Algorithm
/// The Earth rotates 360° in ~24h, so on average each lagna rashi
/// rises for ~2 hours. We probe at fine intervals, detect each
/// boundary crossing into a new rashi, and use binary search to
/// pin the crossing time to ~1-second precision.
///
/// `lat`/`lng` follow the same convention as [`compute`].
pub fn compute_windows(jd_start: f64, lat: f64, lng: f64) -> Vec<LagnaWindow> {
    // Coarse step: ~5 min. Fine enough to never skip a rashi (each rashi
    // rises for ≥1.5h at non-polar latitudes), small enough to keep the
    // binary search bounded.
    const COARSE_STEP_DAYS: f64 = 5.0 / (24.0 * 60.0);
    const REFINE_PRECISION_DAYS: f64 = 1.0 / (24.0 * 3600.0); // 1 second
    const TOTAL_DAYS: f64 = 1.0;

    let initial = compute(jd_start, lat, lng, HouseSystem::WholeSign);
    let mut windows: Vec<LagnaWindow> = Vec::with_capacity(13);
    let mut current_rashi = initial.rashi;
    let mut window_start_jd = jd_start;

    let mut probe_prev = jd_start;
    let mut rashi_prev = current_rashi;
    let mut jd = jd_start + COARSE_STEP_DAYS;

    while jd <= jd_start + TOTAL_DAYS {
        let info = compute(jd, lat, lng, HouseSystem::WholeSign);
        if info.rashi != rashi_prev {
            // The crossing happened somewhere in [probe_prev, jd]. Refine.
            let cross_jd = bisect_rashi_change(probe_prev, jd, rashi_prev, lat, lng);

            windows.push(LagnaWindow {
                rashi: current_rashi,
                rashi_name: RASHI_NAMES[current_rashi as usize],
                start_jd: window_start_jd,
                end_jd: cross_jd,
            });

            current_rashi = info.rashi;
            window_start_jd = cross_jd;
            rashi_prev = info.rashi;
        }
        probe_prev = jd;
        jd += COARSE_STEP_DAYS;
    }

    // Close the final partial window at jd_start + 24h.
    windows.push(LagnaWindow {
        rashi: current_rashi,
        rashi_name: RASHI_NAMES[current_rashi as usize],
        start_jd: window_start_jd,
        end_jd: jd_start + TOTAL_DAYS,
    });

    // Bisection to ~1-second precision: find the JD between `lo` and
    // `hi` where the lagna rashi flips from `prev_rashi` to anything else.
    fn bisect_rashi_change(mut lo: f64, mut hi: f64, prev_rashi: u8, lat: f64, lng: f64) -> f64 {
        while hi - lo > REFINE_PRECISION_DAYS {
            let mid = (lo + hi) * 0.5;
            let info = compute(mid, lat, lng, HouseSystem::WholeSign);
            if info.rashi == prev_rashi {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        hi
    }

    windows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::julian::datetime_to_jd;

    /// Self-consistency: the computed rashi/degree must reconstruct
    /// the original ascendant longitude.
    #[test]
    fn rashi_and_degree_reconstruct_ascendant() {
        let jd = datetime_to_jd(2000, 1, 1, 12, 0, 0.0);
        let info = compute(jd, 28.6139, 77.2090, HouseSystem::Placidus);

        let reconstructed = (info.rashi as f64) * 30.0 + info.degree_in_rashi;
        assert!(
            (reconstructed - info.ascendant_longitude).abs() < 1e-9,
            "rashi/degree should reconstruct ascendant exactly: {reconstructed} vs {}",
            info.ascendant_longitude
        );
    }

    #[test]
    fn rashi_in_valid_range() {
        let jd = datetime_to_jd(1990, 6, 15, 6, 30, 0.0);
        for hour_offset in 0..24 {
            let jd = jd + (hour_offset as f64) / 24.0;
            let info = compute(jd, 19.0760, 72.8777, HouseSystem::Placidus);
            assert!(info.rashi < 12, "rashi out of range: {}", info.rashi);
            assert!(
                info.degree_in_rashi >= 0.0 && info.degree_in_rashi < 30.0,
                "degree_in_rashi out of [0, 30): {}",
                info.degree_in_rashi
            );
            assert!(
                (0.0..360.0).contains(&info.ascendant_longitude),
                "ascendant out of [0, 360): {}",
                info.ascendant_longitude
            );
        }
    }

    /// House 7 cusp must be exactly opposite house 1 (the ascendant)
    /// in every house system that respects the angles.
    #[test]
    fn seventh_house_opposite_ascendant_placidus() {
        let jd = datetime_to_jd(1990, 6, 15, 6, 30, 0.0);
        let info = compute(jd, 19.0760, 72.8777, HouseSystem::Placidus);

        let diff = normalize(info.bhava_cusps[6] - info.bhava_cusps[0]);
        assert!(
            (diff - 180.0).abs() < 1e-6,
            "7th cusp should be 180° from 1st: diff = {diff}"
        );
    }

    /// In whole-sign houses every cusp falls exactly on a sign boundary
    /// (multiple of 30° from the start of the lagna rashi).
    #[test]
    fn whole_sign_cusps_align_with_signs() {
        let jd = datetime_to_jd(1990, 6, 15, 6, 30, 0.0);
        let info = compute(jd, 19.0760, 72.8777, HouseSystem::WholeSign);

        let lagna_sign_start = (info.rashi as f64) * 30.0;
        for (i, cusp) in info.bhava_cusps.iter().enumerate() {
            let expected = normalize(lagna_sign_start + (i as f64) * 30.0);
            let diff = normalize(cusp - expected);
            let signed = if diff > 180.0 { diff - 360.0 } else { diff };
            assert!(
                signed.abs() < 1e-6,
                "whole-sign cusp {} should be {expected}, got {cusp}",
                i + 1
            );
        }
    }

    /// In equal-house mode every cusp is exactly 30° from the ascendant.
    #[test]
    fn equal_house_cusps_are_30_degrees_apart() {
        let jd = datetime_to_jd(1990, 6, 15, 6, 30, 0.0);
        let info = compute(jd, 19.0760, 72.8777, HouseSystem::Equal);

        for i in 0..12 {
            let expected = normalize(info.ascendant_longitude + (i as f64) * 30.0);
            let diff = normalize(info.bhava_cusps[i] - expected);
            let signed = if diff > 180.0 { diff - 360.0 } else { diff };
            assert!(
                signed.abs() < 1e-6,
                "equal-house cusp {} mis-aligned: expected {expected}, got {}",
                i + 1,
                info.bhava_cusps[i]
            );
        }
    }

    #[test]
    fn ayanamsa_value_is_populated_and_lahiri_range() {
        let jd = datetime_to_jd(2000, 1, 1, 12, 0, 0.0);
        let info = compute(jd, 28.6139, 77.2090, HouseSystem::Placidus);
        // Lahiri ayanamsa at J2000 ≈ 23.85°. Allow a generous window.
        assert!(
            (info.ayanamsa - 23.85).abs() < 0.1,
            "expected Lahiri ayanamsa ~23.85° at J2000, got {}",
            info.ayanamsa
        );
    }

    #[test]
    fn windows_cover_24_hours_and_advance_through_zodiac() {
        let jd_start = datetime_to_jd(1990, 6, 15, 0, 0, 0.0);
        let lat = 19.0760;
        let lng = 72.8777;
        let windows = compute_windows(jd_start, lat, lng);

        // At equatorial-to-mid latitudes, all 12 rashis rise in one day,
        // so we get 12 transitions + closing window = up to 13 entries.
        assert!(
            windows.len() >= 12 && windows.len() <= 13,
            "expected 12-13 lagna windows, got {}",
            windows.len()
        );

        // Coverage: total span equals 24h ± epsilon.
        let total: f64 = windows.iter().map(|w| w.end_jd - w.start_jd).sum();
        assert!(
            (total - 1.0).abs() < 1e-6,
            "windows must span 24h, got {total}d"
        );

        // Continuity: each window starts where the previous ended.
        for pair in windows.windows(2) {
            assert!(
                (pair[0].end_jd - pair[1].start_jd).abs() < 1e-9,
                "discontinuity at {} → {}",
                pair[0].end_jd,
                pair[1].start_jd
            );
        }

        // Monotonic rashi progression (mod 12). Because rashis rise in
        // numerical order on the eastern horizon, w[i+1].rashi == (w[i].rashi + 1) % 12.
        for pair in windows.windows(2) {
            if pair[0].rashi == pair[1].rashi {
                continue; // first/last edge — incomplete window
            }
            let expected = (pair[0].rashi + 1) % 12;
            assert_eq!(
                pair[1].rashi, expected,
                "rashi sequence broke: {} → {}",
                pair[0].rashi_name, pair[1].rashi_name
            );
        }
    }

    /// At sunrise the Sun sits on the eastern horizon, so its sidereal
    /// longitude must (by definition) match the ascendant. We allow a
    /// small tolerance because Swiss Ephemeris' "sunrise" uses disc
    /// center + Bhāratīya atmospheric model, not pure horizon crossing.
    #[test]
    fn sunrise_sun_matches_ascendant() {
        use crate::ephemeris::{sidereal_longitude, Planet};
        use crate::sun::sunrise_jd;

        // Mumbai, 1990-06-15. Compute local-noon JD then back into sunrise.
        let jd_midnight = datetime_to_jd(1990, 6, 15, 0, 0, 0.0);
        let lat = 19.0760;
        let lng = 72.8777;
        let alt = 0.0;
        let sunrise = sunrise_jd(jd_midnight, lat, lng, alt);

        let info = compute(sunrise, lat, lng, HouseSystem::Placidus);
        let sun_lon = sidereal_longitude(sunrise, Planet::Sun);

        let diff = normalize(info.ascendant_longitude - sun_lon);
        let signed = if diff > 180.0 { diff - 360.0 } else { diff };
        // Disc-center + atmospheric model can shift the apparent rising
        // by up to ~1° of longitude near the equinoxes/solstices.
        assert!(
            signed.abs() < 1.5,
            "ascendant at sunrise should ≈ sun longitude: asc={}, sun={}, diff={}",
            info.ascendant_longitude,
            sun_lon,
            signed
        );
    }
}
