//! Reference-chart integration tests for the Lagna computation.
//!
//! Each test pins the sidereal Lahiri ascendant we compute against a
//! well-known birth chart whose Lagna is documented in standard jyotish
//! literature. Tolerances are intentionally loose (≤ 1°) so the tests
//! survive minor ayanamsa-precision drift between Swiss Ephemeris
//! revisions, while still catching algorithmic regressions.

use panchang_core::julian::datetime_to_jd;
use panchang_core::lagna::{compute, HouseSystem};

/// India's Independence chart — 1947-08-15 00:00 IST, New Delhi.
/// Standard jyotish literature (B.V. Raman, K.N. Rao) cites the Lagna
/// as Vrishabha ≈ 6°-8°. Drik Panchang gives Vrishabha ≈ 7°44'.
#[test]
fn india_independence_lagna_is_vrishabha() {
    // 1947-08-15 00:00 IST = 1947-08-14 18:30 UT
    let jd = datetime_to_jd(1947, 8, 14, 18, 30, 0.0);
    let info = compute(jd, 28.6139, 77.2090, HouseSystem::Placidus);

    assert_eq!(
        info.rashi, 1,
        "expected Vrishabha (1), got {}",
        info.rashi_name
    );
    assert_eq!(info.rashi_name, "Vrishabha");
    // Drik Panchang: 7°44'. Allow ±1° for ayanamsa-precision drift.
    assert!(
        (info.degree_in_rashi - 7.73).abs() < 1.0,
        "expected ~7.73° in Vrishabha, got {:.4}°",
        info.degree_in_rashi
    );
}

/// M.K. Gandhi — 1869-10-02 07:11:42 LMT, Porbandar (21.6417°N, 69.6293°E).
/// Gandhi's birth chart is one of the most-published in jyotish
/// literature (B.V. Raman, "Notable Horoscopes"). Lagna is documented as
/// Tula (Libra), typically 4°-5°.
#[test]
fn gandhi_lagna_is_tula() {
    // LMT offset for 69.6293°E = 4h38m31s. 07:11:42 LMT - 4h38m31s = 02:33:11 UT.
    let jd = datetime_to_jd(1869, 10, 2, 2, 33, 11.0);
    let info = compute(jd, 21.6417, 69.6293, HouseSystem::Placidus);

    assert_eq!(info.rashi, 6, "expected Tula (6), got {}", info.rashi_name);
    assert_eq!(info.rashi_name, "Tula");
    // Published values cluster around Tula 4°-5°. Our compute: 4.55°.
    assert!(
        (info.degree_in_rashi - 4.55).abs() < 1.0,
        "expected ~4.55° in Tula, got {:.4}°",
        info.degree_in_rashi
    );
}

/// J2000.0 reference — 2000-01-01 12:00 UT, Greenwich.
/// Used as a stable astronomical-epoch checkpoint. Not a published
/// natal chart, but the JD value (2451545.0) is fixed by definition,
/// so any drift in this assertion indicates a regression in our
/// `swe_houses_ex` wiring.
#[test]
fn j2000_epoch_lagna_regression_lock() {
    let jd = datetime_to_jd(2000, 1, 1, 12, 0, 0.0);
    assert!((jd - 2451545.0).abs() < 1e-9, "JD drift: {jd}");

    let info = compute(jd, 51.4779, 0.0, HouseSystem::Placidus);
    // Expected from current Swiss Ephemeris build at Lahiri ayanamsa
    // ~ 23.852°. Lock this in as a regression checkpoint.
    assert_eq!(info.rashi, 0, "expected Mesha (0), got {}", info.rashi_name);
    assert!(
        (info.degree_in_rashi - 0.413).abs() < 0.5,
        "expected ~0.413° in Mesha, got {:.4}°",
        info.degree_in_rashi
    );
}

/// Cross-system sanity: the ascendant longitude is independent of the
/// house-system choice — only the cusp definitions differ. So the
/// rashi/degree_in_rashi must agree across all four systems.
#[test]
fn ascendant_invariant_across_house_systems() {
    let jd = datetime_to_jd(1947, 8, 14, 18, 30, 0.0);
    let lat = 28.6139;
    let lng = 77.2090;

    let p = compute(jd, lat, lng, HouseSystem::Placidus);
    let w = compute(jd, lat, lng, HouseSystem::WholeSign);
    let e = compute(jd, lat, lng, HouseSystem::Equal);
    let o = compute(jd, lat, lng, HouseSystem::Porphyry);

    for other in [&w, &e, &o] {
        assert_eq!(p.rashi, other.rashi);
        assert!((p.ascendant_longitude - other.ascendant_longitude).abs() < 1e-9);
    }
}
