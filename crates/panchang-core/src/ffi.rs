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
/// Sidereal-mode flag for house calculations (`swe_houses_ex`). When set, the
/// house cusps and ascendant are returned in the configured ayanamsa frame
/// (Lahiri, in our case). Value matches `SEFLG_SIDEREAL` in `swephexp.h`
/// (bit 16 = `64 * 1024`).
pub const SEFLG_SIDEREAL: c_int = 64 * 1024;

// --- House system letters (passed to swe_houses / swe_houses_ex as `int hsys`,
// which the C code interprets as an ASCII byte). ---
//
/// Placidus — the most common modern Vedic Bhāva Chalit system.
pub const SE_HSYS_PLACIDUS: c_int = b'P' as c_int;
/// Whole-sign houses — each sign is a full bhāva, traditional Vedic D-1 layout.
pub const SE_HSYS_WHOLE_SIGN: c_int = b'W' as c_int;
/// Equal house from ascendant (12 × 30°).
pub const SE_HSYS_EQUAL: c_int = b'E' as c_int;
/// Porphyry / Sripati — equal-arc trisection between angles. The pure
/// "Sripati Bhāva" system used by classical Jyotish.
pub const SE_HSYS_PORPHYRY: c_int = b'O' as c_int;

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

    /// Compute house cusps + ascendant / MC at a given Julian Day (UT).
    ///
    /// `iflag` accepts `SEFLG_SIDEREAL` to receive sidereal cusps (the active
    /// ayanamsa configured via `swe_set_sid_mode` is applied).
    ///
    /// `hsys` is an ASCII letter selecting the house system (e.g. 'P' for
    /// Placidus, 'W' for Whole-Sign, 'O' for Porphyry/Sripati).
    ///
    /// On return:
    ///   - `cusps[1..=12]` = house cusps in degrees (index 0 unused)
    ///   - `ascmc[0]`      = ascendant
    ///   - `ascmc[1]`      = midheaven (MC)
    ///   - `ascmc[2]`      = ARMC
    ///   - `ascmc[3]`      = vertex
    ///   - `ascmc[4..=7]`  = additional auxiliary points
    ///
    /// Returns OK (0) on success, ERR (-1) on error.
    pub fn swe_houses_ex(
        tjd_ut: c_double,
        iflag: c_int,
        geolat: c_double,
        geolon: c_double,
        hsys: c_int,
        cusps: *mut c_double,
        ascmc: *mut c_double,
    ) -> c_int;
}
