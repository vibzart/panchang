//! Muhurat time window calculations.
//!
//! Rahu Kalam, Yama Gandam, Gulika Kalam, Abhijit Muhurat, and Choghadiya.
//! All depend on exact sunrise/sunset times.

use crate::constants::*;
use crate::types::TimeWindow;

/// Get JD for a 1/8th slot of the daytime period.
fn day_slot_jd(sunrise_jd: f64, day_duration_hours: f64, slot: u32) -> (f64, f64) {
    let slot_hours = day_duration_hours / 8.0;
    let slot_jd = slot_hours / 24.0;
    let start = sunrise_jd + slot as f64 * slot_jd;
    let end = sunrise_jd + (slot + 1) as f64 * slot_jd;
    (start, end)
}

/// Compute Rahu Kalam time window.
pub fn rahu_kalam(weekday: u32, sunrise_jd: f64, day_duration_hours: f64) -> TimeWindow {
    let slot = RAHU_KALAM_SLOT[weekday as usize % 7];
    let (start, end) = day_slot_jd(sunrise_jd, day_duration_hours, slot);
    TimeWindow {
        name: "Rahu Kalam".to_string(),
        start_jd: start,
        end_jd: end,
        is_auspicious: false,
    }
}

/// Compute Yama Gandam time window.
pub fn yama_gandam(weekday: u32, sunrise_jd: f64, day_duration_hours: f64) -> TimeWindow {
    let slot = YAMA_GANDAM_SLOT[weekday as usize % 7];
    let (start, end) = day_slot_jd(sunrise_jd, day_duration_hours, slot);
    TimeWindow {
        name: "Yama Gandam".to_string(),
        start_jd: start,
        end_jd: end,
        is_auspicious: false,
    }
}

/// Compute Gulika Kalam time window.
pub fn gulika_kalam(weekday: u32, sunrise_jd: f64, day_duration_hours: f64) -> TimeWindow {
    let slot = GULIKA_KALAM_SLOT[weekday as usize % 7];
    let (start, end) = day_slot_jd(sunrise_jd, day_duration_hours, slot);
    TimeWindow {
        name: "Gulika Kalam".to_string(),
        start_jd: start,
        end_jd: end,
        is_auspicious: false,
    }
}

/// Compute Abhijit Muhurat — the auspicious midday window.
///
/// The 8th muhurta (out of 15) between sunrise and sunset.
pub fn abhijit_muhurat(sunrise_jd: f64, day_duration_hours: f64) -> TimeWindow {
    let muhurta_hours = day_duration_hours / 15.0;
    let muhurta_jd = muhurta_hours / 24.0;

    // 8th muhurta (0-indexed: 7th)
    let start = sunrise_jd + 7.0 * muhurta_jd;
    let end = start + muhurta_jd;

    TimeWindow {
        name: "Abhijit Muhurat".to_string(),
        start_jd: start,
        end_jd: end,
        is_auspicious: true,
    }
}

/// Compute all 16 Choghadiya windows (8 day + 8 night).
pub fn choghadiya(
    weekday: u32,
    sunrise_jd: f64,
    sunset_jd: f64,
    day_duration_hours: f64,
) -> Vec<TimeWindow> {
    let mut windows = Vec::with_capacity(16);

    // Day choghadiya: 8 equal divisions of daytime
    let day_slot_jd_len = day_duration_hours / 8.0 / 24.0;
    let start_idx = DAY_CHOGHADIYA_START[weekday as usize % 7];

    for i in 0..8u32 {
        let name_idx = (start_idx + i as usize) % 7;
        let name = CHOGHADIYA_NAMES[name_idx];
        let is_auspicious = matches!(name, "Amrit" | "Shubh" | "Labh" | "Char");

        windows.push(TimeWindow {
            name: name.to_string(),
            start_jd: sunrise_jd + i as f64 * day_slot_jd_len,
            end_jd: sunrise_jd + (i + 1) as f64 * day_slot_jd_len,
            is_auspicious,
        });
    }

    // Night choghadiya: 8 equal divisions of nighttime
    let night_hours = 24.0 - day_duration_hours;
    let night_slot_jd_len = night_hours / 8.0 / 24.0;
    let start_idx = NIGHT_CHOGHADIYA_START[weekday as usize % 7];

    for i in 0..8u32 {
        let name_idx = (start_idx + i as usize) % 7;
        let name = CHOGHADIYA_NAMES[name_idx];
        let is_auspicious = matches!(name, "Amrit" | "Shubh" | "Labh" | "Char");

        windows.push(TimeWindow {
            name: name.to_string(),
            start_jd: sunset_jd + i as f64 * night_slot_jd_len,
            end_jd: sunset_jd + (i + 1) as f64 * night_slot_jd_len,
            is_auspicious,
        });
    }

    windows
}
