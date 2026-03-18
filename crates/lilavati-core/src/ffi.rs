//! Raw FFI bindings to the Swiss Ephemeris C library.
//!
//! Hand-written declarations for only the functions we use.
//! This avoids the `bindgen` + `libclang` build-time dependency.

use std::os::raw::{c_char, c_double, c_int};

// --- Planet identifiers ---
pub const SE_SUN: c_int = 0;
pub const SE_MOON: c_int = 1;
pub const SE_MERCURY: c_int = 2;
pub const SE_VENUS: c_int = 3;
pub const SE_MARS: c_int = 4;
pub const SE_JUPITER: c_int = 5;
pub const SE_SATURN: c_int = 6;
pub const SE_MEAN_NODE: c_int = 10; // Mean Rahu (north lunar node)

// --- Calculation flags ---
/// Use Moshier analytical ephemeris (built into C library, no external data files needed).
/// Accuracy: ~0.1 arcsec for Moon, ~1 arcsec for outer planets — more than sufficient
/// for Panchang (sub-second timing error in Tithi transitions).
pub const SEFLG_MOSEPH: c_int = 4;
pub const SEFLG_SPEED: c_int = 256;

// --- Sidereal modes ---
pub const SE_SIDM_LAHIRI: c_int = 1;

// --- Rise/set flags ---
pub const SE_CALC_RISE: c_int = 1;
pub const SE_CALC_SET: c_int = 2;
pub const SE_BIT_DISC_CENTER: c_int = 256;
pub const SE_BIT_NO_REFRACTION: c_int = 512;
pub const SE_BIT_GEOCTR_NO_ECL_LAT: c_int = 128;
/// Hindu rising method: disc center at horizon, Bhāratīya atmospheric model.
pub const SE_BIT_HINDU_RISING: c_int =
    SE_BIT_DISC_CENTER | SE_BIT_NO_REFRACTION | SE_BIT_GEOCTR_NO_ECL_LAT;

// --- Calendar ---
pub const SE_GREG_CAL: c_int = 1;

// --- Error buffer size ---
pub const SE_ERR_LEN: usize = 256;

extern "C" {
    /// Compute planetary position at a given Julian Day (UT).
    ///
    /// `xx` must point to a 6-element array:
    ///   [0] longitude, [1] latitude, [2] distance,
    ///   [3] longitude speed, [4] latitude speed, [5] distance speed
    ///
    /// Returns flags on success, negative on error.
    pub fn swe_calc_ut(
        tjd_ut: c_double,
        ipl: c_int,
        iflag: c_int,
        xx: *mut c_double,
        serr: *mut c_char,
    ) -> c_int;

    /// Set the path for ephemeris data files (.se1).
    pub fn swe_set_ephe_path(path: *const c_char);

    /// Set the sidereal mode (ayanamsa).
    /// `t0` and `ayan_t0` are 0.0 for standard modes.
    pub fn swe_set_sid_mode(sid_mode: c_int, t0: c_double, ayan_t0: c_double);

    /// Get the ayanamsa value for a Julian Day (UT).
    pub fn swe_get_ayanamsa_ut(tjd_ut: c_double) -> c_double;

    /// Compute rise/set/transit times.
    ///
    /// `geopos` must point to a 3-element array: [longitude, latitude, altitude].
    /// `tret` must point to a 1-element array (or at least 10 for some modes).
    ///
    /// Returns OK (0) on success, ERR (-1) on error.
    pub fn swe_rise_trans(
        tjd_ut: c_double,
        ipl: c_int,
        starname: *const c_char,
        epheflag: c_int,
        rsmi: c_int,
        geopos: *const c_double,
        atpress: c_double,
        attemp: c_double,
        tret: *mut c_double,
        serr: *mut c_char,
    ) -> c_int;

    /// Release Swiss Ephemeris resources and free memory.
    pub fn swe_close();

    /// Convert calendar date to Julian Day number.
    pub fn swe_julday(
        year: c_int,
        month: c_int,
        day: c_int,
        hour: c_double,
        gregflag: c_int,
    ) -> c_double;

    /// Convert Julian Day number to calendar date.
    pub fn swe_revjul(
        jd: c_double,
        gregflag: c_int,
        jyear: *mut c_int,
        jmon: *mut c_int,
        jday: *mut c_int,
        jut: *mut c_double,
    );
}
