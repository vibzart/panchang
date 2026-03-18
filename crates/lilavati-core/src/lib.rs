//! Lilavati core computation engine.
//!
//! Rust implementation of Bhāratīya calendar (Panchang) computations,
//! exposed to Python via PyO3.

pub mod angles;
pub mod batch;
pub mod constants;
pub mod ephemeris;
pub mod festival;
pub mod ffi;
pub mod julian;
pub mod lunar_month;
pub mod muhurat;
pub mod panchang;
pub mod samvat;
pub mod sankranti;
pub mod search;
pub mod shraddha;
pub mod sun;
pub mod types;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// ============================================================================
// PyO3 module definition
// ============================================================================

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialization
    m.add_function(wrap_pyfunction!(py_init, m)?)?;
    m.add_function(wrap_pyfunction!(py_close, m)?)?;

    // Julian Day conversions
    m.add_function(wrap_pyfunction!(py_datetime_to_jd, m)?)?;
    m.add_function(wrap_pyfunction!(py_jd_to_datetime, m)?)?;

    // Planetary positions (granular — for debugging)
    m.add_function(wrap_pyfunction!(py_tropical_longitude, m)?)?;
    m.add_function(wrap_pyfunction!(py_sidereal_longitude, m)?)?;
    m.add_function(wrap_pyfunction!(py_ayanamsa, m)?)?;
    m.add_function(wrap_pyfunction!(py_planet_speed, m)?)?;

    // Sunrise/sunset (granular)
    m.add_function(wrap_pyfunction!(py_sunrise_jd, m)?)?;
    m.add_function(wrap_pyfunction!(py_sunset_jd, m)?)?;
    m.add_function(wrap_pyfunction!(py_compute_sun_data, m)?)?;

    // High-level: complete Panchang
    m.add_function(wrap_pyfunction!(py_compute_panchang, m)?)?;

    // Muhurat windows
    m.add_function(wrap_pyfunction!(py_compute_muhurat, m)?)?;
    m.add_function(wrap_pyfunction!(py_compute_choghadiya, m)?)?;

    // Calendar: Sankranti + Lunar months
    m.add_function(wrap_pyfunction!(py_compute_sankrantis, m)?)?;
    m.add_function(wrap_pyfunction!(py_compute_lunar_months, m)?)?;

    // Festivals, Ekadashis, Vrat dates
    m.add_function(wrap_pyfunction!(py_compute_festivals, m)?)?;
    m.add_function(wrap_pyfunction!(py_compute_ekadashis, m)?)?;
    m.add_function(wrap_pyfunction!(py_compute_vrat_dates, m)?)?;

    // Batch computation
    m.add_function(wrap_pyfunction!(py_compute_batch_year, m)?)?;
    m.add_function(wrap_pyfunction!(py_compute_batch_range, m)?)?;

    // Samvat (era year) + Shraddha
    m.add_function(wrap_pyfunction!(py_era_year, m)?)?;
    m.add_function(wrap_pyfunction!(py_jovian_cycle_index, m)?)?;
    m.add_function(wrap_pyfunction!(py_compute_shraddha, m)?)?;

    // Planet constants for Python
    m.add("PLANET_SUN", 0)?;
    m.add("PLANET_MOON", 1)?;
    m.add("PLANET_MERCURY", 2)?;
    m.add("PLANET_VENUS", 3)?;
    m.add("PLANET_MARS", 4)?;
    m.add("PLANET_JUPITER", 5)?;
    m.add("PLANET_SATURN", 6)?;
    m.add("PLANET_RAHU", 10)?;
    m.add("PLANET_KETU", -1)?;

    Ok(())
}

// ============================================================================
// Initialization
// ============================================================================

#[pyfunction]
#[pyo3(signature = (ephe_path=None))]
fn py_init(ephe_path: Option<&str>) {
    ephemeris::init(ephe_path);
}

#[pyfunction]
fn py_close() {
    ephemeris::close();
}

// ============================================================================
// Julian Day conversions
// ============================================================================

/// Convert (year, month, day, hour, min, sec) to Julian Day.
#[pyfunction]
fn py_datetime_to_jd(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: f64) -> f64 {
    julian::datetime_to_jd(year, month, day, hour, min, sec)
}

/// Convert Julian Day to (year, month, day, hour, min, sec, microsec).
#[pyfunction]
fn py_jd_to_datetime(jd: f64) -> (i32, u32, u32, u32, u32, u32, u32) {
    let dt = julian::jd_to_datetime(jd);
    (
        dt.year,
        dt.month,
        dt.day,
        dt.hour,
        dt.minute,
        dt.second,
        dt.microsecond,
    )
}

// ============================================================================
// Planetary positions (granular — for debugging)
// ============================================================================

#[pyfunction]
fn py_tropical_longitude(jd: f64, planet_id: i32) -> PyResult<f64> {
    let planet = ephemeris::Planet::from_i32(planet_id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid planet ID"))?;
    Ok(ephemeris::tropical_longitude(jd, planet))
}

#[pyfunction]
fn py_sidereal_longitude(jd: f64, planet_id: i32) -> PyResult<f64> {
    let planet = ephemeris::Planet::from_i32(planet_id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid planet ID"))?;
    Ok(ephemeris::sidereal_longitude(jd, planet))
}

#[pyfunction]
fn py_ayanamsa(jd: f64) -> f64 {
    ephemeris::ayanamsa(jd)
}

#[pyfunction]
fn py_planet_speed(jd: f64, planet_id: i32) -> PyResult<f64> {
    let planet = ephemeris::Planet::from_i32(planet_id)
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid planet ID"))?;
    Ok(ephemeris::planet_speed(jd, planet))
}

// ============================================================================
// Sunrise / Sunset
// ============================================================================

/// Compute sunrise JD. `jd_start` = local midnight JD.
#[pyfunction]
fn py_sunrise_jd(jd_start: f64, lat: f64, lng: f64, alt: f64) -> f64 {
    ephemeris::init(None);
    sun::sunrise_jd(jd_start, lat, lng, alt)
}

/// Compute sunset JD. `jd_noon` = local noon JD.
#[pyfunction]
fn py_sunset_jd(jd_noon: f64, lat: f64, lng: f64, alt: f64) -> f64 {
    ephemeris::init(None);
    sun::sunset_jd(jd_noon, lat, lng, alt)
}

/// Compute sun data: {sunrise_jd, sunset_jd, day_duration_hours}.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_compute_sun_data<'py>(
    py: Python<'py>,
    year: i32,
    month: u32,
    day: u32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset_seconds: i32,
) -> PyResult<Bound<'py, PyDict>> {
    let data = sun::compute_sun_data(year, month, day, lat, lng, alt, utc_offset_seconds);
    let dict = PyDict::new(py);
    dict.set_item("sunrise_jd", data.sunrise_jd)?;
    dict.set_item("sunset_jd", data.sunset_jd)?;
    dict.set_item("day_duration_hours", data.day_duration_hours)?;
    Ok(dict)
}

// ============================================================================
// Complete Panchang
// ============================================================================

/// Compute complete Panchang. Returns a dict with all 5 elements.
///
/// `weekday`: 0=Sunday, 6=Saturday (Hindu convention).
/// `utc_offset_seconds`: timezone offset (e.g. 19800 for IST).
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_compute_panchang<'py>(
    py: Python<'py>,
    year: i32,
    month: u32,
    day: u32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset_seconds: i32,
    weekday: u32,
) -> PyResult<Bound<'py, PyDict>> {
    let sun_data = sun::compute_sun_data(year, month, day, lat, lng, alt, utc_offset_seconds);
    let mut result = panchang::compute(sun_data.sunrise_jd, weekday);
    result.sun = sun_data;

    let dict = PyDict::new(py);

    // Sun data
    let sun_dict = PyDict::new(py);
    sun_dict.set_item("sunrise_jd", result.sun.sunrise_jd)?;
    sun_dict.set_item("sunset_jd", result.sun.sunset_jd)?;
    sun_dict.set_item("day_duration_hours", result.sun.day_duration_hours)?;
    dict.set_item("sun", sun_dict)?;

    // Vara
    let vara_dict = PyDict::new(py);
    vara_dict.set_item("number", result.vara.number)?;
    vara_dict.set_item("name", result.vara.name)?;
    vara_dict.set_item("english", result.vara.english)?;
    dict.set_item("vara", vara_dict)?;

    // Tithi
    let tithi_dict = PyDict::new(py);
    tithi_dict.set_item("number", result.tithi.number)?;
    tithi_dict.set_item("name", &result.tithi.name)?;
    tithi_dict.set_item("paksha", result.tithi.paksha)?;
    tithi_dict.set_item("start_jd", result.tithi.start_jd)?;
    tithi_dict.set_item("end_jd", result.tithi.end_jd)?;
    dict.set_item("tithi", tithi_dict)?;

    // Nakshatra
    let nak_dict = PyDict::new(py);
    nak_dict.set_item("number", result.nakshatra.number)?;
    nak_dict.set_item("name", result.nakshatra.name)?;
    nak_dict.set_item("pada", result.nakshatra.pada)?;
    nak_dict.set_item("lord", result.nakshatra.lord)?;
    nak_dict.set_item("start_jd", result.nakshatra.start_jd)?;
    nak_dict.set_item("end_jd", result.nakshatra.end_jd)?;
    dict.set_item("nakshatra", nak_dict)?;

    // Yoga
    let yoga_dict = PyDict::new(py);
    yoga_dict.set_item("number", result.yoga.number)?;
    yoga_dict.set_item("name", result.yoga.name)?;
    yoga_dict.set_item("start_jd", result.yoga.start_jd)?;
    yoga_dict.set_item("end_jd", result.yoga.end_jd)?;
    dict.set_item("yoga", yoga_dict)?;

    // Karana
    let karana_dict = PyDict::new(py);
    karana_dict.set_item("number", result.karana.number)?;
    karana_dict.set_item("name", result.karana.name)?;
    karana_dict.set_item("start_jd", result.karana.start_jd)?;
    karana_dict.set_item("end_jd", result.karana.end_jd)?;
    dict.set_item("karana", karana_dict)?;

    Ok(dict)
}

// ============================================================================
// Muhurat windows
// ============================================================================

/// Compute muhurat windows (Rahu Kalam, Yama Gandam, Gulika, Abhijit).
/// Returns a dict with all 4 windows.
#[pyfunction]
fn py_compute_muhurat<'py>(
    py: Python<'py>,
    weekday: u32,
    sunrise_jd: f64,
    day_duration_hours: f64,
) -> PyResult<Bound<'py, PyDict>> {
    let rk = muhurat::rahu_kalam(weekday, sunrise_jd, day_duration_hours);
    let yg = muhurat::yama_gandam(weekday, sunrise_jd, day_duration_hours);
    let gk = muhurat::gulika_kalam(weekday, sunrise_jd, day_duration_hours);
    let am = muhurat::abhijit_muhurat(sunrise_jd, day_duration_hours);

    let dict = PyDict::new(py);
    dict.set_item("rahu_kalam", window_to_dict(py, &rk)?)?;
    dict.set_item("yama_gandam", window_to_dict(py, &yg)?)?;
    dict.set_item("gulika_kalam", window_to_dict(py, &gk)?)?;
    dict.set_item("abhijit_muhurat", window_to_dict(py, &am)?)?;
    Ok(dict)
}

/// Compute Choghadiya windows (8 day + 8 night = 16 total).
/// Returns a list of dicts.
#[pyfunction]
fn py_compute_choghadiya<'py>(
    py: Python<'py>,
    weekday: u32,
    sunrise_jd: f64,
    sunset_jd: f64,
    day_duration_hours: f64,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    let windows = muhurat::choghadiya(weekday, sunrise_jd, sunset_jd, day_duration_hours);
    windows
        .iter()
        .map(|w| window_to_dict(py, w))
        .collect::<PyResult<Vec<_>>>()
}

// ============================================================================
// Calendar: Sankranti + Lunar months
// ============================================================================

/// Compute all 12 Sankrantis for a year. Returns list of dicts.
#[pyfunction]
fn py_compute_sankrantis<'py>(py: Python<'py>, year: i32) -> PyResult<Vec<Bound<'py, PyDict>>> {
    ephemeris::init(None);
    let results = sankranti::compute_sankrantis(year);

    results
        .iter()
        .map(|s| {
            let dt = julian::jd_to_datetime(s.jd);
            let dict = PyDict::new(py);
            dict.set_item("index", s.index)?;
            dict.set_item("name", s.name)?;
            dict.set_item("rashi", s.rashi)?;
            dict.set_item("target_longitude", s.target_longitude)?;
            dict.set_item("jd", s.jd)?;
            dict.set_item("year", dt.year)?;
            dict.set_item("month", dt.month)?;
            dict.set_item("day", dt.day)?;
            Ok(dict)
        })
        .collect()
}

/// Compute lunar months for a year. `system`: "amant" or "purnimant".
#[pyfunction]
#[pyo3(signature = (year, system="amant"))]
fn py_compute_lunar_months<'py>(
    py: Python<'py>,
    year: i32,
    system: &str,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    ephemeris::init(None);

    let cal_system = match system.to_lowercase().as_str() {
        "purnimant" => lunar_month::CalendarSystem::Purnimant,
        _ => lunar_month::CalendarSystem::Amant,
    };

    let months = lunar_month::compute_lunar_months(year, cal_system);

    months
        .iter()
        .map(|m| {
            let dict = PyDict::new(py);
            dict.set_item("number", m.number)?;
            dict.set_item("name", m.name)?;
            dict.set_item("is_adhik", m.is_adhik)?;
            dict.set_item("is_kshaya", m.is_kshaya)?;
            dict.set_item("start_jd", m.start_jd)?;
            dict.set_item("end_jd", m.end_jd)?;
            Ok(dict)
        })
        .collect()
}

// ============================================================================
// Festivals, Ekadashis, Vrat dates
// ============================================================================

/// Compute festival dates from definitions (passed as list of dicts from YAML).
/// Each dict: {id, name, rule, lunar_month, tithi, sankranti_index}.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_compute_festivals<'py>(
    py: Python<'py>,
    festival_defs: &Bound<'py, PyList>,
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    ephemeris::init(None);

    let mut defs = Vec::new();
    for item in festival_defs.iter() {
        let d = item.downcast::<PyDict>()?;
        defs.push(festival::FestivalDef {
            id: extract_str(d, "id")?,
            name: extract_str(d, "name")?,
            rule: extract_str(d, "rule")?,
            lunar_month: extract_u32_or(d, "lunar_month", 0)?,
            tithi: extract_u32_or(d, "tithi", 0)?,
            sankranti_index: extract_opt_u32(d, "sankranti_index")?,
            nakshatra: extract_opt_u32(d, "nakshatra")?,
        });
    }

    let results = festival::compute_festivals(&defs, year, lat, lng, alt, utc_offset);

    results
        .iter()
        .map(|r| {
            let dict = PyDict::new(py);
            dict.set_item("festival_id", &r.festival_id)?;
            dict.set_item("festival_name", &r.festival_name)?;
            dict.set_item("year", r.year)?;
            dict.set_item("month", r.month)?;
            dict.set_item("day", r.day)?;
            dict.set_item("sunrise_jd", r.sunrise_jd)?;
            dict.set_item("tithi_at_sunrise", r.tithi_at_sunrise)?;
            dict.set_item("lunar_month_name", r.lunar_month_name)?;
            dict.set_item("is_adhik_month", r.is_adhik_month)?;
            dict.set_item("reasoning", &r.reasoning)?;
            Ok(dict)
        })
        .collect()
}

/// Compute Ekadashis from definitions (passed as list of dicts from YAML).
/// Each dict: {month, shukla_name, krishna_name}.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_compute_ekadashis<'py>(
    py: Python<'py>,
    ekadashi_defs: &Bound<'py, PyList>,
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    ephemeris::init(None);

    let mut defs = Vec::new();
    for item in ekadashi_defs.iter() {
        let d = item.downcast::<PyDict>()?;
        defs.push(festival::EkadashiDef {
            month: extract_u32(d, "month")?,
            shukla_name: extract_str(d, "shukla_name")?,
            krishna_name: extract_str(d, "krishna_name")?,
        });
    }

    let results = festival::compute_ekadashis(&defs, year, lat, lng, alt, utc_offset);

    results
        .iter()
        .map(|ek| {
            let dict = PyDict::new(py);
            dict.set_item("name", &ek.name)?;
            dict.set_item("lunar_month", ek.lunar_month)?;
            dict.set_item("lunar_month_name", ek.lunar_month_name)?;
            dict.set_item("paksha", ek.paksha)?;
            dict.set_item("smartha_year", ek.smartha_year)?;
            dict.set_item("smartha_month", ek.smartha_month)?;
            dict.set_item("smartha_day", ek.smartha_day)?;
            dict.set_item("smartha_sunrise_jd", ek.smartha_sunrise_jd)?;
            dict.set_item("vaishnava_year", ek.vaishnava_year)?;
            dict.set_item("vaishnava_month", ek.vaishnava_month)?;
            dict.set_item("vaishnava_day", ek.vaishnava_day)?;
            dict.set_item("vaishnava_sunrise_jd", ek.vaishnava_sunrise_jd)?;
            dict.set_item("reasoning", &ek.reasoning)?;
            Ok(dict)
        })
        .collect()
}

/// Compute Vrat (fasting) dates for a year.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_compute_vrat_dates<'py>(
    py: Python<'py>,
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    ephemeris::init(None);

    let results = festival::compute_vrat_dates(year, lat, lng, alt, utc_offset);

    results
        .iter()
        .map(|v| {
            let dict = PyDict::new(py);
            dict.set_item("vrat_type", &v.vrat_type)?;
            dict.set_item("name", &v.name)?;
            dict.set_item("year", v.year)?;
            dict.set_item("month", v.month)?;
            dict.set_item("day", v.day)?;
            dict.set_item("sunrise_jd", v.sunrise_jd)?;
            dict.set_item("lunar_month_name", v.lunar_month_name)?;
            dict.set_item("paksha", v.paksha)?;
            Ok(dict)
        })
        .collect()
}

// ============================================================================
// Batch computation
// ============================================================================

/// Compute panchang for every day of a year. Returns list of dicts.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_compute_batch_year<'py>(
    py: Python<'py>,
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    ephemeris::init(None);

    let results = batch::compute_year(year, lat, lng, alt, utc_offset);
    batch_results_to_dicts(py, &results)
}

/// Compute panchang for a date range. Returns list of dicts.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_compute_batch_range<'py>(
    py: Python<'py>,
    start_year: i32,
    start_month: u32,
    start_day: u32,
    end_year: i32,
    end_month: u32,
    end_day: u32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    ephemeris::init(None);

    let results = batch::compute_range(
        start_year,
        start_month,
        start_day,
        end_year,
        end_month,
        end_day,
        lat,
        lng,
        alt,
        utc_offset,
    );
    batch_results_to_dicts(py, &results)
}

// ============================================================================
// Samvat (era year) + Shraddha
// ============================================================================

/// Compute era year from Gregorian year + offset.
#[pyfunction]
fn py_era_year(gregorian_year: i32, offset: i32, new_year_passed: bool) -> i32 {
    samvat::era_year_from_offset(gregorian_year, offset, new_year_passed)
}

/// Compute 60-year Jovian cycle index (0-59).
#[pyfunction]
fn py_jovian_cycle_index(gregorian_year: i32, epoch_year: i32) -> u32 {
    samvat::jovian_cycle_index(gregorian_year, epoch_year)
}

/// Compute Shraddha (death anniversary) date.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_compute_shraddha<'py>(
    py: Python<'py>,
    death_year: i32,
    death_month: u32,
    death_day: u32,
    target_year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> PyResult<Option<Bound<'py, PyDict>>> {
    ephemeris::init(None);

    let result = shraddha::compute_shraddha(
        death_year,
        death_month,
        death_day,
        target_year,
        lat,
        lng,
        alt,
        utc_offset,
    );

    match result {
        Some(r) => {
            let dict = PyDict::new(py);
            dict.set_item("tithi", r.tithi)?;
            dict.set_item("lunar_month", r.lunar_month)?;
            dict.set_item("lunar_month_name", r.lunar_month_name)?;
            dict.set_item("year", r.year)?;
            dict.set_item("month", r.month)?;
            dict.set_item("day", r.day)?;
            dict.set_item("sunrise_jd", r.sunrise_jd)?;
            dict.set_item("reasoning", &r.reasoning)?;
            Ok(Some(dict))
        }
        None => Ok(None),
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn batch_results_to_dicts<'py>(
    py: Python<'py>,
    results: &[batch::BatchDayResult],
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    results
        .iter()
        .map(|r| {
            let dict = PyDict::new(py);
            dict.set_item("year", r.year)?;
            dict.set_item("month", r.month)?;
            dict.set_item("day", r.day)?;

            // Sun data
            let sun_dict = PyDict::new(py);
            sun_dict.set_item("sunrise_jd", r.panchang.sun.sunrise_jd)?;
            sun_dict.set_item("sunset_jd", r.panchang.sun.sunset_jd)?;
            sun_dict.set_item("day_duration_hours", r.panchang.sun.day_duration_hours)?;
            dict.set_item("sun", sun_dict)?;

            // Vara
            let vara_dict = PyDict::new(py);
            vara_dict.set_item("number", r.panchang.vara.number)?;
            vara_dict.set_item("name", r.panchang.vara.name)?;
            vara_dict.set_item("english", r.panchang.vara.english)?;
            dict.set_item("vara", vara_dict)?;

            // Tithi
            let tithi_dict = PyDict::new(py);
            tithi_dict.set_item("number", r.panchang.tithi.number)?;
            tithi_dict.set_item("name", &r.panchang.tithi.name)?;
            tithi_dict.set_item("paksha", r.panchang.tithi.paksha)?;
            dict.set_item("tithi", tithi_dict)?;

            // Nakshatra
            let nak_dict = PyDict::new(py);
            nak_dict.set_item("number", r.panchang.nakshatra.number)?;
            nak_dict.set_item("name", r.panchang.nakshatra.name)?;
            nak_dict.set_item("pada", r.panchang.nakshatra.pada)?;
            nak_dict.set_item("lord", r.panchang.nakshatra.lord)?;
            dict.set_item("nakshatra", nak_dict)?;

            // Yoga
            let yoga_dict = PyDict::new(py);
            yoga_dict.set_item("number", r.panchang.yoga.number)?;
            yoga_dict.set_item("name", r.panchang.yoga.name)?;
            dict.set_item("yoga", yoga_dict)?;

            // Karana
            let karana_dict = PyDict::new(py);
            karana_dict.set_item("number", r.panchang.karana.number)?;
            karana_dict.set_item("name", r.panchang.karana.name)?;
            dict.set_item("karana", karana_dict)?;

            Ok(dict)
        })
        .collect()
}

fn window_to_dict<'py>(py: Python<'py>, w: &types::TimeWindow) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("name", &w.name)?;
    dict.set_item("start_jd", w.start_jd)?;
    dict.set_item("end_jd", w.end_jd)?;
    dict.set_item("is_auspicious", w.is_auspicious)?;
    Ok(dict)
}

/// Extract a String from a PyDict, raising KeyError if missing.
fn extract_str(d: &Bound<'_, PyDict>, key: &str) -> PyResult<String> {
    d.get_item(key)?
        .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_string()))?
        .extract()
}

/// Extract a u32 from a PyDict, raising KeyError if missing.
fn extract_u32(d: &Bound<'_, PyDict>, key: &str) -> PyResult<u32> {
    d.get_item(key)?
        .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_string()))?
        .extract()
}

/// Extract a u32 from a PyDict, returning a default if missing.
fn extract_u32_or(d: &Bound<'_, PyDict>, key: &str, default: u32) -> PyResult<u32> {
    match d.get_item(key)? {
        Some(val) => val.extract().or(Ok(default)),
        None => Ok(default),
    }
}

/// Extract an optional u32 from a PyDict. Returns None if key missing or value is None.
fn extract_opt_u32(d: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<u32>> {
    match d.get_item(key)? {
        Some(val) => {
            if val.is_none() {
                Ok(None)
            } else {
                Ok(Some(val.extract()?))
            }
        }
        None => Ok(None),
    }
}
