//! Julian Day <-> calendar date conversions.
//!
//! Uses Swiss Ephemeris `swe_julday` / `swe_revjul` for accuracy.

#![allow(dead_code)]

use crate::ffi;

/// Calendar date/time components (UTC).
#[derive(Debug, Clone, Copy)]
pub struct DateTimeComponents {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub microsecond: u32,
}

/// Convert calendar date/time (UTC) to Julian Day number.
pub fn datetime_to_jd(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: f64) -> f64 {
    let hour_frac = hour as f64 + min as f64 / 60.0 + sec / 3600.0;
    unsafe { ffi::swe_julday(year, month as i32, day as i32, hour_frac, ffi::SE_GREG_CAL) }
}

/// Convert Julian Day number to calendar date/time components (UTC).
pub fn jd_to_datetime(jd: f64) -> DateTimeComponents {
    let mut year: i32 = 0;
    let mut month: i32 = 0;
    let mut day: i32 = 0;
    let mut hour_frac: f64 = 0.0;

    unsafe {
        ffi::swe_revjul(
            jd,
            ffi::SE_GREG_CAL,
            &mut year,
            &mut month,
            &mut day,
            &mut hour_frac,
        );
    }

    // Convert to total microseconds first, then decompose.
    // This avoids cascading truncation errors (e.g. 6.9999999 → 6 hours).
    let total_us = (hour_frac * 3_600_000_000.0).round() as u64;
    let hours = (total_us / 3_600_000_000) as u32;
    let remainder_us = total_us % 3_600_000_000;
    let minutes = (remainder_us / 60_000_000) as u32;
    let remainder_us = remainder_us % 60_000_000;
    let seconds = (remainder_us / 1_000_000) as u32;
    let microseconds = (remainder_us % 1_000_000) as u32;

    DateTimeComponents {
        year,
        month: month as u32,
        day: day as u32,
        hour: hours,
        minute: minutes,
        second: seconds,
        microsecond: microseconds,
    }
}

/// Convenience: compute JD for local midnight given a date and UTC offset in seconds.
pub fn midnight_jd(year: i32, month: u32, day: u32, utc_offset_seconds: i32) -> f64 {
    // Local midnight = 00:00 local = -offset in UTC
    let utc_hour_frac = -(utc_offset_seconds as f64) / 3600.0;
    // If negative, roll back to previous day
    if utc_hour_frac < 0.0 {
        // Midnight local is on the previous UTC day
        let jd_noon = datetime_to_jd(year, month, day, 12, 0, 0.0);
        jd_noon - 0.5 + utc_hour_frac / 24.0
    } else {
        datetime_to_jd(year, month, day, 0, 0, 0.0) + utc_hour_frac / 24.0
    }
}

/// Convenience: compute JD for local noon given a date and UTC offset in seconds.
pub fn noon_jd(year: i32, month: u32, day: u32, utc_offset_seconds: i32) -> f64 {
    midnight_jd(year, month, day, utc_offset_seconds) + 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_j2000_epoch() {
        // J2000.0: 2000-01-01 12:00 UT = JD 2451545.0
        let jd = datetime_to_jd(2000, 1, 1, 12, 0, 0.0);
        assert!((jd - 2451545.0).abs() < 0.0001);
    }

    #[test]
    fn test_roundtrip() {
        let jd = datetime_to_jd(2026, 2, 24, 6, 30, 0.0);
        let dt = jd_to_datetime(jd);
        assert_eq!(dt.year, 2026);
        assert_eq!(dt.month, 2);
        assert_eq!(dt.day, 24);
        assert_eq!(dt.hour, 6);
        assert_eq!(dt.minute, 30);
    }
}
