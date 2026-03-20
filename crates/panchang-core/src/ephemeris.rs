//! Safe Rust wrapper around Swiss Ephemeris FFI.
//!
//! Handles initialization, ayanamsa configuration, and planetary position queries.

use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::angles::normalize;
use crate::ffi;

/// Tracks whether Swiss Ephemeris has been initialized.
/// Reset to `false` on `close()` so re-initialization works correctly.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Planet identifiers matching Swiss Ephemeris constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Planet {
    Sun = ffi::SE_SUN,
    Moon = ffi::SE_MOON,
    Mercury = ffi::SE_MERCURY,
    Venus = ffi::SE_VENUS,
    Mars = ffi::SE_MARS,
    Jupiter = ffi::SE_JUPITER,
    Saturn = ffi::SE_SATURN,
    Rahu = ffi::SE_MEAN_NODE,
    /// Ketu is computed as Rahu + 180°, not from ephemeris.
    Ketu = -1,
}

impl Planet {
    /// Convert from integer (used in PyO3 bridge).
    pub fn from_i32(v: i32) -> Option<Planet> {
        match v {
            0 => Some(Planet::Sun),
            1 => Some(Planet::Moon),
            2 => Some(Planet::Mercury),
            3 => Some(Planet::Venus),
            4 => Some(Planet::Mars),
            5 => Some(Planet::Jupiter),
            6 => Some(Planet::Saturn),
            10 => Some(Planet::Rahu),
            -1 => Some(Planet::Ketu),
            _ => None,
        }
    }
}

/// Initialize Swiss Ephemeris with Lahiri ayanamsa.
/// Safe to call multiple times — re-initializes after `close()`.
pub fn init(ephe_path: Option<&str>) {
    if !INITIALIZED.swap(true, Ordering::SeqCst) {
        if let Some(path) = ephe_path {
            if let Ok(c_path) = CString::new(path) {
                unsafe { ffi::swe_set_ephe_path(c_path.as_ptr()) };
            }
        }
        unsafe { ffi::swe_set_sid_mode(ffi::SE_SIDM_LAHIRI, 0.0, 0.0) };
    }
}

/// Release Swiss Ephemeris resources.
/// Resets the initialization flag so the next `init()` call will re-configure.
pub fn close() {
    unsafe { ffi::swe_close() };
    INITIALIZED.store(false, Ordering::SeqCst);
}

/// Get the Lahiri ayanamsa value (degrees) at a given Julian Day.
pub fn ayanamsa(jd: f64) -> f64 {
    init(None);
    unsafe { ffi::swe_get_ayanamsa_ut(jd) }
}

/// Get tropical (Western) longitude of a planet in degrees [0, 360).
pub fn tropical_longitude(jd: f64, planet: Planet) -> f64 {
    init(None);
    if planet == Planet::Ketu {
        let rahu_long = calc_longitude(jd, Planet::Rahu);
        return normalize(rahu_long + 180.0);
    }
    calc_longitude(jd, planet)
}

/// Get sidereal (Vedic/Nirayana) longitude of a planet in degrees [0, 360).
pub fn sidereal_longitude(jd: f64, planet: Planet) -> f64 {
    let tropical = tropical_longitude(jd, planet);
    let ayan = ayanamsa(jd);
    normalize(tropical - ayan)
}

/// Get the speed of a planet in degrees per day.
pub fn planet_speed(jd: f64, planet: Planet) -> f64 {
    init(None);
    if planet == Planet::Ketu {
        return -calc_speed(jd, Planet::Rahu);
    }
    calc_speed(jd, planet)
}

// --- Internal helpers ---

fn calc_longitude(jd: f64, planet: Planet) -> f64 {
    let mut xx = [0.0f64; 6];
    let mut serr = [0i8; ffi::SE_ERR_LEN];
    let flags = ffi::SEFLG_MOSEPH | ffi::SEFLG_SPEED;

    unsafe {
        ffi::swe_calc_ut(jd, planet as i32, flags, xx.as_mut_ptr(), serr.as_mut_ptr());
    }
    normalize(xx[0])
}

fn calc_speed(jd: f64, planet: Planet) -> f64 {
    let mut xx = [0.0f64; 6];
    let mut serr = [0i8; ffi::SE_ERR_LEN];
    let flags = ffi::SEFLG_MOSEPH | ffi::SEFLG_SPEED;

    unsafe {
        ffi::swe_calc_ut(jd, planet as i32, flags, xx.as_mut_ptr(), serr.as_mut_ptr());
    }
    xx[3]
}
