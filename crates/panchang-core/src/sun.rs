//! Sunrise, sunset, and solar calculations.
//!
//! Uses Swiss Ephemeris `swe_rise_trans()` with Hindu rising method:
//! disc center at horizon with Bhāratīya atmospheric model.

use crate::ffi;
use crate::types::SunData;

/// Rise/set flags: Hindu rising method.
const RISE_FLAGS: i32 = ffi::SE_BIT_HINDU_RISING;

/// Compute sunrise Julian Day for a given date and location.
///
/// `jd_start` should be the JD of local midnight (start of day).
/// `swe_rise_trans` searches FORWARD from the given JD.
pub fn sunrise_jd(jd_start: f64, lat: f64, lng: f64, alt: f64) -> f64 {
    let mut geopos = [lng, lat, alt];
    let mut tret = [0.0f64; 10];
    let mut serr = [0i8; ffi::SE_ERR_LEN];

    unsafe {
        ffi::swe_rise_trans(
            jd_start,
            ffi::SE_SUN,
            std::ptr::null(),
            ffi::SEFLG_MOSEPH,
            ffi::SE_CALC_RISE | RISE_FLAGS,
            geopos.as_mut_ptr(),
            0.0, // atpress (0 = default)
            0.0, // attemp (0 = default)
            tret.as_mut_ptr(),
            serr.as_mut_ptr(),
        );
    }
    tret[0]
}

/// Compute sunset Julian Day for a given date and location.
///
/// `jd_noon` should be the JD of local noon. Sunset always comes after noon.
pub fn sunset_jd(jd_noon: f64, lat: f64, lng: f64, alt: f64) -> f64 {
    let mut geopos = [lng, lat, alt];
    let mut tret = [0.0f64; 10];
    let mut serr = [0i8; ffi::SE_ERR_LEN];

    unsafe {
        ffi::swe_rise_trans(
            jd_noon,
            ffi::SE_SUN,
            std::ptr::null(),
            ffi::SEFLG_MOSEPH,
            ffi::SE_CALC_SET | RISE_FLAGS,
            geopos.as_mut_ptr(),
            0.0,
            0.0,
            tret.as_mut_ptr(),
            serr.as_mut_ptr(),
        );
    }
    tret[0]
}

/// Compute sunrise, sunset, and day duration for a date and location.
///
/// `utc_offset_seconds`: timezone offset from UTC in seconds (e.g. IST = 19800).
pub fn compute_sun_data(
    year: i32,
    month: u32,
    day: u32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset_seconds: i32,
) -> SunData {
    crate::ephemeris::init(None);

    let jd_midnight = crate::julian::midnight_jd(year, month, day, utc_offset_seconds);
    let jd_noon = jd_midnight + 0.5;

    let rise = sunrise_jd(jd_midnight, lat, lng, alt);
    let set = sunset_jd(jd_noon, lat, lng, alt);
    let duration = (set - rise) * 24.0;

    SunData {
        sunrise_jd: rise,
        sunset_jd: set,
        day_duration_hours: (duration * 10000.0).round() / 10000.0,
    }
}
