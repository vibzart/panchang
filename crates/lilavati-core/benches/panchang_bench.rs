//! Criterion benchmarks for Lilavati core computations.
//!
//! Measures performance of all major computation paths:
//! sunrise/sunset, planetary positions, full panchang, and muhurat windows.
//!
//! Run: cargo bench --manifest-path crates/lilavati-core/Cargo.toml

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lilavati_core::ephemeris::{self, Planet};
use lilavati_core::julian;
use lilavati_core::muhurat;
use lilavati_core::panchang;
use lilavati_core::sun;

// --- Fixed parameters: Delhi, 2026-02-24 ---
const DELHI_LAT: f64 = 28.6139;
const DELHI_LNG: f64 = 77.2090;
const DELHI_ALT: f64 = 216.0;
const IST_OFFSET: i32 = 19800; // UTC+5:30 in seconds

/// Get midnight JD for Delhi, 2026-02-24.
fn delhi_midnight_jd() -> f64 {
    julian::midnight_jd(2026, 2, 24, IST_OFFSET)
}

fn bench_sunrise(c: &mut Criterion) {
    ephemeris::init(None);
    let jd_midnight = delhi_midnight_jd();

    c.bench_function("sunrise_jd", |b| {
        b.iter(|| sun::sunrise_jd(black_box(jd_midnight), DELHI_LAT, DELHI_LNG, DELHI_ALT))
    });
}

fn bench_sunset(c: &mut Criterion) {
    ephemeris::init(None);
    let jd_noon = delhi_midnight_jd() + 0.5;

    c.bench_function("sunset_jd", |b| {
        b.iter(|| sun::sunset_jd(black_box(jd_noon), DELHI_LAT, DELHI_LNG, DELHI_ALT))
    });
}

fn bench_compute_sun_data(c: &mut Criterion) {
    ephemeris::init(None);

    c.bench_function("compute_sun_data", |b| {
        b.iter(|| {
            sun::compute_sun_data(
                black_box(2026),
                black_box(2),
                black_box(24),
                DELHI_LAT,
                DELHI_LNG,
                DELHI_ALT,
                IST_OFFSET,
            )
        })
    });
}

fn bench_tropical_longitude(c: &mut Criterion) {
    ephemeris::init(None);
    let jd = delhi_midnight_jd() + 0.25; // ~6 AM

    c.bench_function("tropical_longitude_moon", |b| {
        b.iter(|| ephemeris::tropical_longitude(black_box(jd), Planet::Moon))
    });
}

fn bench_sidereal_longitude(c: &mut Criterion) {
    ephemeris::init(None);
    let jd = delhi_midnight_jd() + 0.25;

    c.bench_function("sidereal_longitude_moon", |b| {
        b.iter(|| ephemeris::sidereal_longitude(black_box(jd), Planet::Moon))
    });
}

fn bench_ayanamsa(c: &mut Criterion) {
    ephemeris::init(None);
    let jd = delhi_midnight_jd() + 0.25;

    c.bench_function("ayanamsa", |b| {
        b.iter(|| ephemeris::ayanamsa(black_box(jd)))
    });
}

fn bench_full_panchang(c: &mut Criterion) {
    ephemeris::init(None);
    let sun_data = sun::compute_sun_data(2026, 2, 24, DELHI_LAT, DELHI_LNG, DELHI_ALT, IST_OFFSET);
    let weekday = 2; // Tuesday

    c.bench_function("full_panchang", |b| {
        b.iter(|| panchang::compute(black_box(sun_data.sunrise_jd), black_box(weekday)))
    });
}

fn bench_muhurat_windows(c: &mut Criterion) {
    ephemeris::init(None);
    let sun_data = sun::compute_sun_data(2026, 2, 24, DELHI_LAT, DELHI_LNG, DELHI_ALT, IST_OFFSET);
    let weekday = 2u32;

    c.bench_function("muhurat_all_windows", |b| {
        b.iter(|| {
            muhurat::rahu_kalam(
                black_box(weekday),
                black_box(sun_data.sunrise_jd),
                black_box(sun_data.day_duration_hours),
            );
            muhurat::yama_gandam(weekday, sun_data.sunrise_jd, sun_data.day_duration_hours);
            muhurat::gulika_kalam(weekday, sun_data.sunrise_jd, sun_data.day_duration_hours);
            muhurat::abhijit_muhurat(sun_data.sunrise_jd, sun_data.day_duration_hours);
        })
    });
}

fn bench_choghadiya(c: &mut Criterion) {
    ephemeris::init(None);
    let sun_data = sun::compute_sun_data(2026, 2, 24, DELHI_LAT, DELHI_LNG, DELHI_ALT, IST_OFFSET);
    let weekday = 2u32;

    c.bench_function("choghadiya_16_windows", |b| {
        b.iter(|| {
            muhurat::choghadiya(
                black_box(weekday),
                black_box(sun_data.sunrise_jd),
                black_box(sun_data.sunset_jd),
                black_box(sun_data.day_duration_hours),
            )
        })
    });
}

fn bench_all_planets(c: &mut Criterion) {
    ephemeris::init(None);
    let jd = delhi_midnight_jd() + 0.25;
    let planets = [
        Planet::Sun,
        Planet::Moon,
        Planet::Mercury,
        Planet::Venus,
        Planet::Mars,
        Planet::Jupiter,
        Planet::Saturn,
        Planet::Rahu,
        Planet::Ketu,
    ];

    c.bench_function("all_9_planets_sidereal", |b| {
        b.iter(|| {
            for &planet in &planets {
                ephemeris::sidereal_longitude(black_box(jd), planet);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_sunrise,
    bench_sunset,
    bench_compute_sun_data,
    bench_tropical_longitude,
    bench_sidereal_longitude,
    bench_ayanamsa,
    bench_full_panchang,
    bench_muhurat_windows,
    bench_choghadiya,
    bench_all_planets,
);
criterion_main!(benches);
