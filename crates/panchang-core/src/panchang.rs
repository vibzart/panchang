//! Panchang computation engine — the 5 elements of the Hindu calendar.
//!
//! All computations use sunrise JD as the reference point,
//! because the Hindu day starts at sunrise.

use crate::angles::normalize;
use crate::constants::*;
use crate::ephemeris::{self, Planet};
use crate::search::{find_crossing_backward, find_crossing_forward};
use crate::types::*;

/// Span of each element in degrees.
const TITHI_SPAN: f64 = 12.0; // 360 / 30
const NAKSHATRA_SPAN: f64 = 360.0 / 27.0; // 13°20'
const YOGA_SPAN: f64 = 360.0 / 27.0;
const KARANA_SPAN: f64 = 6.0; // 360 / 60

/// Max days to search for transition times.
const MAX_SEARCH_DAYS: f64 = 2.0;

/// Compute complete Panchang at a given sunrise JD.
pub fn compute(sunrise_jd: f64, weekday: u32) -> PanchangResult {
    let vara = compute_vara(weekday);
    let tithi = compute_tithi(sunrise_jd);
    let nakshatra = compute_nakshatra(sunrise_jd);
    let yoga = compute_yoga(sunrise_jd);
    let karana = compute_karana(sunrise_jd);

    PanchangResult {
        sun: SunData {
            sunrise_jd,
            sunset_jd: 0.0,          // filled by caller
            day_duration_hours: 0.0, // filled by caller
        },
        vara,
        tithi,
        nakshatra,
        yoga,
        karana,
    }
}

// --- Vara (weekday) ---

fn compute_vara(weekday: u32) -> VaraInfo {
    let idx = weekday as usize % 7;
    VaraInfo {
        number: weekday,
        name: VARA_NAMES[idx],
        english: VARA_ENGLISH[idx],
    }
}

// --- Tithi ---

/// Get the tithi angle (Moon - Sun tropical longitude) at a given JD.
fn tithi_angle(jd: f64) -> f64 {
    let moon = ephemeris::tropical_longitude(jd, Planet::Moon);
    let sun = ephemeris::tropical_longitude(jd, Planet::Sun);
    normalize(moon - sun)
}

fn compute_tithi(jd: f64) -> TithiInfo {
    let angle = tithi_angle(jd);
    let tithi_idx = (angle / TITHI_SPAN) as u32; // 0-indexed (0-29)
    let tithi_num = tithi_idx + 1; // 1-indexed (1-30)

    let paksha = if tithi_num <= 15 { "Shukla" } else { "Krishna" };
    let tithi_name = TITHI_NAMES[tithi_idx as usize];
    let full_name = format!("{} {}", paksha, tithi_name);

    // Transition times
    let target_start = tithi_idx as f64 * TITHI_SPAN;
    let target_end = (tithi_idx + 1) as f64 * TITHI_SPAN;

    let angle_fn = |jd: f64| -> f64 { tithi_angle(jd) };

    let start_jd =
        find_crossing_backward(jd, target_start, &angle_fn, MAX_SEARCH_DAYS).unwrap_or(jd - 1.0);
    let end_jd =
        find_crossing_forward(jd, target_end, &angle_fn, MAX_SEARCH_DAYS).unwrap_or(jd + 1.0);

    TithiInfo {
        number: tithi_num,
        name: full_name,
        paksha,
        start_jd,
        end_jd,
    }
}

// --- Nakshatra ---

/// Get Moon's sidereal longitude for Nakshatra computation.
fn nakshatra_angle(jd: f64) -> f64 {
    ephemeris::sidereal_longitude(jd, Planet::Moon)
}

fn compute_nakshatra(jd: f64) -> NakshatraInfo {
    let moon_sid = nakshatra_angle(jd);
    let nak_idx = (moon_sid / NAKSHATRA_SPAN) as u32; // 0-indexed (0-26)
    let nak_num = nak_idx + 1;

    // Pada (quarter)
    let pada_span = NAKSHATRA_SPAN / 4.0;
    let offset_in_nak = moon_sid - (nak_idx as f64 * NAKSHATRA_SPAN);
    let pada = ((offset_in_nak / pada_span) as u32 + 1).min(4);

    // Transition times
    let target_start = nak_idx as f64 * NAKSHATRA_SPAN;
    let mut target_end = (nak_idx + 1) as f64 * NAKSHATRA_SPAN;
    if target_end >= 360.0 {
        target_end -= 360.0;
    }

    let angle_fn = |jd: f64| -> f64 { nakshatra_angle(jd) };

    let start_jd =
        find_crossing_backward(jd, target_start, &angle_fn, MAX_SEARCH_DAYS).unwrap_or(jd - 1.0);
    let end_jd =
        find_crossing_forward(jd, target_end, &angle_fn, MAX_SEARCH_DAYS).unwrap_or(jd + 1.0);

    NakshatraInfo {
        number: nak_num,
        name: NAKSHATRA_NAMES[nak_idx as usize],
        pada,
        lord: NAKSHATRA_LORDS[nak_idx as usize],
        start_jd,
        end_jd,
    }
}

// --- Yoga ---

/// Yoga angle: (Sun sidereal + Moon sidereal) mod 360.
fn yoga_angle(jd: f64) -> f64 {
    let sun_sid = ephemeris::sidereal_longitude(jd, Planet::Sun);
    let moon_sid = ephemeris::sidereal_longitude(jd, Planet::Moon);
    normalize(sun_sid + moon_sid)
}

fn compute_yoga(jd: f64) -> YogaInfo {
    let angle = yoga_angle(jd);
    let yoga_idx = (angle / YOGA_SPAN) as u32; // 0-indexed (0-26)
    let yoga_num = yoga_idx + 1;

    let target_start = yoga_idx as f64 * YOGA_SPAN;
    let mut target_end = (yoga_idx + 1) as f64 * YOGA_SPAN;
    if target_end >= 360.0 {
        target_end -= 360.0;
    }

    let angle_fn = |jd: f64| -> f64 { yoga_angle(jd) };

    let start_jd =
        find_crossing_backward(jd, target_start, &angle_fn, MAX_SEARCH_DAYS).unwrap_or(jd - 1.0);
    let end_jd =
        find_crossing_forward(jd, target_end, &angle_fn, MAX_SEARCH_DAYS).unwrap_or(jd + 1.0);

    YogaInfo {
        number: yoga_num,
        name: YOGA_NAMES[yoga_idx as usize],
        start_jd,
        end_jd,
    }
}

// --- Karana ---

fn compute_karana(jd: f64) -> KaranaInfo {
    let angle = tithi_angle(jd);
    let karana_60 = (angle / KARANA_SPAN) as u32; // 0-indexed (0-59)

    let karana_name = karana_name_from_number(karana_60);
    let karana_number = KARANA_NAMES
        .iter()
        .position(|&n| n == karana_name)
        .map(|i| i as u32 + 1)
        .unwrap_or(1);

    // Transition times
    let target_start = karana_60 as f64 * KARANA_SPAN;
    let mut target_end = (karana_60 + 1) as f64 * KARANA_SPAN;
    if target_end >= 360.0 {
        target_end -= 360.0;
    }

    let angle_fn = |jd: f64| -> f64 { tithi_angle(jd) };

    let start_jd =
        find_crossing_backward(jd, target_start, &angle_fn, MAX_SEARCH_DAYS).unwrap_or(jd - 1.0);
    let end_jd =
        find_crossing_forward(jd, target_end, &angle_fn, MAX_SEARCH_DAYS).unwrap_or(jd + 1.0);

    KaranaInfo {
        number: karana_number,
        name: karana_name,
        start_jd,
        end_jd,
    }
}

/// Map a karana number (0-59) to its name.
///
/// Karana cycle in a lunar month:
///   0: Kimstughna (fixed — first half of Shukla Pratipada)
///   1-56: 7 rotating karanas repeated 8 times
///   57: Shakuni (fixed)
///   58: Chatushpada (fixed)
///   59: Nagava (fixed)
fn karana_name_from_number(karana_60: u32) -> &'static str {
    match karana_60 {
        0 => "Kimstughna",
        1..=56 => {
            let rotating_idx = ((karana_60 - 1) % 7) as usize;
            KARANA_NAMES[rotating_idx]
        }
        57 => "Shakuni",
        58 => "Chatushpada",
        59 => "Nagava",
        _ => "Bava", // Should never happen
    }
}
