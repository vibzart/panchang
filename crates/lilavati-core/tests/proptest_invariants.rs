//! Property-based tests for Lilavati core mathematical invariants.
//!
//! Uses proptest to verify invariants hold across random inputs,
//! catching edge cases that hand-written unit tests miss.
//!
//! Run: cargo test --manifest-path crates/lilavati-core/Cargo.toml -- --test-threads=1
//!
//! IMPORTANT: --test-threads=1 is required because Swiss Ephemeris uses global C state.

use proptest::prelude::*;
use std::sync::Mutex;

use lilavati_core::angles::{forward_distance, normalize};
use lilavati_core::ephemeris::{self, Planet};
use lilavati_core::julian;
use lilavati_core::muhurat;
use lilavati_core::panchang;
use lilavati_core::sun;

/// Global mutex to serialize Swiss Ephemeris access across proptest threads.
static SWE_LOCK: Mutex<()> = Mutex::new(());

/// Ensure Swiss Ephemeris is initialized (thread-safe).
fn ensure_init() {
    let _guard = SWE_LOCK.lock().unwrap();
    ephemeris::init(None);
}

// ============================================================================
// Angle invariants
// ============================================================================

proptest! {
    #[test]
    fn normalize_always_in_range(angle in prop::num::f64::NORMAL) {
        let n = normalize(angle);
        prop_assert!(n >= 0.0, "normalize({}) = {} < 0", angle, n);
        prop_assert!(n < 360.0, "normalize({}) = {} >= 360", angle, n);
    }

    #[test]
    fn normalize_is_periodic(angle in -1e6_f64..1e6_f64) {
        let n1 = normalize(angle);
        let n2 = normalize(angle + 360.0);
        prop_assert!((n1 - n2).abs() < 1e-10,
            "normalize({}) = {}, normalize({} + 360) = {}", angle, n1, angle, n2);
    }

    #[test]
    fn normalize_idempotent(angle in -1e6_f64..1e6_f64) {
        let n1 = normalize(angle);
        let n2 = normalize(n1);
        prop_assert!((n1 - n2).abs() < 1e-10,
            "normalize not idempotent: normalize({}) = {}, normalize({}) = {}", angle, n1, n1, n2);
    }

    #[test]
    fn forward_distance_always_in_range(a in 0.0_f64..360.0, b in 0.0_f64..360.0) {
        let d = forward_distance(a, b);
        prop_assert!(d >= 0.0, "forward_distance({}, {}) = {} < 0", a, b, d);
        prop_assert!(d < 360.0, "forward_distance({}, {}) = {} >= 360", a, b, d);
    }

    #[test]
    fn forward_distance_zero_to_self(a in 0.0_f64..360.0) {
        let d = forward_distance(a, a);
        prop_assert!(d.abs() < 1e-10,
            "forward_distance({}, {}) = {} (expected 0)", a, a, d);
    }
}

// ============================================================================
// Julian Day roundtrip
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn jd_roundtrip(
        year in 1900_i32..2100,
        month in 1_u32..=12,
        day in 1_u32..=28,    // Avoid month-end edge cases
        hour in 0_u32..=23,
        min in 0_u32..=59,
    ) {
        let jd = julian::datetime_to_jd(year, month, day, hour, min, 0.0);
        let dt = julian::jd_to_datetime(jd);

        prop_assert_eq!(dt.year, year, "Year mismatch for JD {}", jd);
        prop_assert_eq!(dt.month, month, "Month mismatch for JD {}", jd);
        prop_assert_eq!(dt.day, day, "Day mismatch for JD {}", jd);

        // Compare total seconds — allows ±1 second tolerance for inherent
        // floating-point precision loss in JD representation (~millisecond
        // precision at JD magnitudes ~2400000).
        let original_total_sec = (hour * 3600 + min * 60) as i64;
        let roundtrip_total_sec = (dt.hour * 3600 + dt.minute * 60 + dt.second) as i64;
        let diff_sec = (original_total_sec - roundtrip_total_sec).abs();
        prop_assert!(diff_sec <= 1,
            "Time drift {} seconds for {}-{:02}-{:02} {:02}:{:02}:00 (got {:02}:{:02}:{:02})",
            diff_sec, year, month, day, hour, min, dt.hour, dt.minute, dt.second);
    }
}

// ============================================================================
// Planetary position invariants
// ============================================================================

// Valid JD range: ~1900 to ~2100
const JD_MIN: f64 = 2415020.0; // 1900-01-01
const JD_MAX: f64 = 2488070.0; // 2100-01-01

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn tropical_longitude_in_range(jd in JD_MIN..JD_MAX) {
        ensure_init();
        let planets = [
            Planet::Sun, Planet::Moon, Planet::Mercury, Planet::Venus,
            Planet::Mars, Planet::Jupiter, Planet::Saturn, Planet::Rahu, Planet::Ketu,
        ];
        for planet in &planets {
            let lon = ephemeris::tropical_longitude(jd, *planet);
            prop_assert!(lon >= 0.0 && lon < 360.0,
                "tropical_longitude({:?}, JD={}) = {} out of range", planet, jd, lon);
        }
    }

    #[test]
    fn sidereal_longitude_in_range(jd in JD_MIN..JD_MAX) {
        ensure_init();
        let planets = [
            Planet::Sun, Planet::Moon, Planet::Mercury, Planet::Venus,
            Planet::Mars, Planet::Jupiter, Planet::Saturn, Planet::Rahu, Planet::Ketu,
        ];
        for planet in &planets {
            let lon = ephemeris::sidereal_longitude(jd, *planet);
            prop_assert!(lon >= 0.0 && lon < 360.0,
                "sidereal_longitude({:?}, JD={}) = {} out of range", planet, jd, lon);
        }
    }

    #[test]
    fn sidereal_equals_tropical_minus_ayanamsa(jd in JD_MIN..JD_MAX) {
        ensure_init();
        let planets = [
            Planet::Sun, Planet::Moon, Planet::Mercury, Planet::Venus,
            Planet::Mars, Planet::Jupiter, Planet::Saturn, Planet::Rahu, Planet::Ketu,
        ];
        let ayan = ephemeris::ayanamsa(jd);
        for planet in &planets {
            let tropical = ephemeris::tropical_longitude(jd, *planet);
            let sidereal = ephemeris::sidereal_longitude(jd, *planet);
            let expected = normalize(tropical - ayan);
            let diff = (sidereal - expected).abs();
            // Allow small floating-point tolerance
            prop_assert!(diff < 0.001 || (360.0 - diff) < 0.001,
                "sidereal({:?}) = {}, normalize(tropical - ayan) = {}, diff = {}",
                planet, sidereal, expected, diff);
        }
    }

    #[test]
    fn ketu_opposite_rahu(jd in JD_MIN..JD_MAX) {
        ensure_init();
        let rahu = ephemeris::tropical_longitude(jd, Planet::Rahu);
        let ketu = ephemeris::tropical_longitude(jd, Planet::Ketu);
        let expected_ketu = normalize(rahu + 180.0);
        let diff = (ketu - expected_ketu).abs();
        prop_assert!(diff < 0.001 || (360.0 - diff) < 0.001,
            "Ketu = {}, expected normalize(Rahu + 180) = {}, Rahu = {}", ketu, expected_ketu, rahu);
    }

    #[test]
    fn ayanamsa_reasonable_range(jd in JD_MIN..JD_MAX) {
        ensure_init();
        let ayan = ephemeris::ayanamsa(jd);
        // Lahiri ayanamsa for 1900-2100 should be roughly 22-26 degrees
        prop_assert!(ayan > 20.0 && ayan < 28.0,
            "Ayanamsa at JD {} = {} (expected 20-28)", jd, ayan);
    }
}

// ============================================================================
// Panchang element bounds
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn panchang_element_bounds(
        // Use JDs around reasonable sunrise times (midnight + 0.25..0.35 days)
        jd_offset in 0.25_f64..0.35,
        day_offset in 0_i32..365,
        weekday in 0_u32..7,
    ) {
        ensure_init();
        // Base JD: 2026-01-01 midnight UTC
        let base_jd = 2460676.5;
        let jd = base_jd + day_offset as f64 + jd_offset;

        let result = panchang::compute(jd, weekday);

        // Tithi: 1-30
        prop_assert!(result.tithi.number >= 1 && result.tithi.number <= 30,
            "Tithi number {} out of [1,30] at JD {}", result.tithi.number, jd);

        // Nakshatra: 1-27
        prop_assert!(result.nakshatra.number >= 1 && result.nakshatra.number <= 27,
            "Nakshatra number {} out of [1,27] at JD {}", result.nakshatra.number, jd);

        // Yoga: 1-27
        prop_assert!(result.yoga.number >= 1 && result.yoga.number <= 27,
            "Yoga number {} out of [1,27] at JD {}", result.yoga.number, jd);

        // Karana: 1-11
        prop_assert!(result.karana.number >= 1 && result.karana.number <= 11,
            "Karana number {} out of [1,11] at JD {}", result.karana.number, jd);

        // Transition times: end > start
        prop_assert!(result.tithi.end_jd > result.tithi.start_jd,
            "Tithi end ({}) <= start ({}) at JD {}", result.tithi.end_jd, result.tithi.start_jd, jd);
        prop_assert!(result.nakshatra.end_jd > result.nakshatra.start_jd,
            "Nakshatra end <= start at JD {}", jd);
        prop_assert!(result.yoga.end_jd > result.yoga.start_jd,
            "Yoga end <= start at JD {}", jd);
        prop_assert!(result.karana.end_jd > result.karana.start_jd,
            "Karana end <= start at JD {}", jd);

        // Vara number matches input weekday
        prop_assert_eq!(result.vara.number, weekday,
            "Vara number {} != weekday {} at JD {}", result.vara.number, weekday, jd);
    }
}

// ============================================================================
// Sunrise/sunset and muhurat invariants
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn sun_data_reasonable(day_offset in 0_i32..365) {
        ensure_init();
        let year = 2026;
        let base_month = 1_u32;
        // Convert day_offset to date
        let day = 1 + (day_offset as u32 % 28);
        let month = base_month + ((day_offset as u32 / 28) % 12);
        let month = if month > 12 { month - 12 } else { month };

        let sun_data = sun::compute_sun_data(
            year, month, day,
            28.6139, 77.2090, 216.0, // Delhi
            19800, // IST
        );

        // Day duration should be between 0 and 24 hours
        prop_assert!(sun_data.day_duration_hours > 0.0 && sun_data.day_duration_hours < 24.0,
            "Day duration {} out of (0,24) for {}-{:02}-{:02}", sun_data.day_duration_hours, year, month, day);

        // Sunset must come after sunrise
        prop_assert!(sun_data.sunset_jd > sun_data.sunrise_jd,
            "Sunset ({}) <= sunrise ({}) for {}-{:02}-{:02}",
            sun_data.sunset_jd, sun_data.sunrise_jd, year, month, day);
    }

    #[test]
    fn muhurat_windows_valid(weekday in 0_u32..7) {
        ensure_init();
        let sun_data = sun::compute_sun_data(2026, 2, 24, 28.6139, 77.2090, 216.0, 19800);

        let rk = muhurat::rahu_kalam(weekday, sun_data.sunrise_jd, sun_data.day_duration_hours);
        let yg = muhurat::yama_gandam(weekday, sun_data.sunrise_jd, sun_data.day_duration_hours);
        let gk = muhurat::gulika_kalam(weekday, sun_data.sunrise_jd, sun_data.day_duration_hours);
        let am = muhurat::abhijit_muhurat(sun_data.sunrise_jd, sun_data.day_duration_hours);

        // All windows: end > start
        for (name, w) in [("Rahu Kalam", &rk), ("Yama Gandam", &yg), ("Gulika", &gk), ("Abhijit", &am)] {
            prop_assert!(w.end_jd > w.start_jd,
                "{} end ({}) <= start ({}) for weekday {}", name, w.end_jd, w.start_jd, weekday);
        }

        // Rahu Kalam, Yama Gandam, Gulika each = 1/8 of day duration
        let expected_duration_jd = sun_data.day_duration_hours / 8.0 / 24.0;
        for (name, w) in [("Rahu Kalam", &rk), ("Yama Gandam", &yg), ("Gulika", &gk)] {
            let actual_duration = w.end_jd - w.start_jd;
            let diff_minutes = (actual_duration - expected_duration_jd).abs() * 24.0 * 60.0;
            prop_assert!(diff_minutes < 1.0,
                "{} duration off by {:.1} minutes for weekday {}", name, diff_minutes, weekday);
        }

        // Rahu Kalam, Yama Gandam, Gulika are inauspicious
        prop_assert!(!rk.is_auspicious, "Rahu Kalam should be inauspicious");
        prop_assert!(!yg.is_auspicious, "Yama Gandam should be inauspicious");
        prop_assert!(!gk.is_auspicious, "Gulika should be inauspicious");

        // Abhijit is auspicious
        prop_assert!(am.is_auspicious, "Abhijit should be auspicious");
    }

    #[test]
    fn choghadiya_returns_16_windows(weekday in 0_u32..7) {
        ensure_init();
        let sun_data = sun::compute_sun_data(2026, 2, 24, 28.6139, 77.2090, 216.0, 19800);

        let windows = muhurat::choghadiya(
            weekday,
            sun_data.sunrise_jd,
            sun_data.sunset_jd,
            sun_data.day_duration_hours,
        );

        prop_assert_eq!(windows.len(), 16,
            "Choghadiya returned {} windows (expected 16) for weekday {}", windows.len(), weekday);

        // All windows: end > start
        for (i, w) in windows.iter().enumerate() {
            prop_assert!(w.end_jd > w.start_jd,
                "Choghadiya window {} ({}) end <= start for weekday {}", i, w.name, weekday);
        }

        // First 8 are day windows (start >= sunrise)
        for w in &windows[..8] {
            prop_assert!(w.start_jd >= sun_data.sunrise_jd - 1e-10,
                "Day choghadiya {} starts before sunrise", w.name);
        }

        // Last 8 are night windows (start >= sunset)
        for w in &windows[8..] {
            prop_assert!(w.start_jd >= sun_data.sunset_jd - 1e-10,
                "Night choghadiya {} starts before sunset", w.name);
        }
    }
}
