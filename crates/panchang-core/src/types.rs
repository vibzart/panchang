//! Internal Rust types for computation results.
//!
//! These are converted to Python dicts at the PyO3 boundary.

/// Sunrise/sunset data.
#[derive(Debug, Clone)]
pub struct SunData {
    pub sunrise_jd: f64,
    pub sunset_jd: f64,
    pub day_duration_hours: f64,
}

/// Tithi (lunar day) information.
#[derive(Debug, Clone)]
pub struct TithiInfo {
    /// 1-30
    pub number: u32,
    /// e.g. "Shukla Saptami"
    pub name: String,
    /// "Shukla" or "Krishna"
    pub paksha: &'static str,
    /// Julian Day of tithi start
    pub start_jd: f64,
    /// Julian Day of tithi end
    pub end_jd: f64,
}

/// Nakshatra (lunar mansion) information.
#[derive(Debug, Clone)]
pub struct NakshatraInfo {
    /// 1-27
    pub number: u32,
    /// e.g. "Rohini"
    pub name: &'static str,
    /// 1-4
    pub pada: u32,
    /// Ruling planet
    pub lord: &'static str,
    /// Julian Day of nakshatra start
    pub start_jd: f64,
    /// Julian Day of nakshatra end
    pub end_jd: f64,
}

/// Yoga (Sun-Moon combination) information.
#[derive(Debug, Clone)]
pub struct YogaInfo {
    /// 1-27
    pub number: u32,
    /// e.g. "Siddhi"
    pub name: &'static str,
    /// Julian Day of yoga start
    pub start_jd: f64,
    /// Julian Day of yoga end
    pub end_jd: f64,
}

/// Karana (half-tithi) information.
#[derive(Debug, Clone)]
pub struct KaranaInfo {
    /// 1-11 (index into KARANA_NAMES)
    pub number: u32,
    /// e.g. "Bava"
    pub name: &'static str,
    /// Julian Day of karana start
    pub start_jd: f64,
    /// Julian Day of karana end
    pub end_jd: f64,
}

/// Vara (weekday) information.
#[derive(Debug, Clone)]
pub struct VaraInfo {
    /// 0-6 (Sunday=0)
    pub number: u32,
    /// Sanskrit name
    pub name: &'static str,
    /// English name
    pub english: &'static str,
}

/// Time window for muhurat calculations.
#[derive(Debug, Clone)]
pub struct TimeWindow {
    pub name: String,
    pub start_jd: f64,
    pub end_jd: f64,
    pub is_auspicious: bool,
}

/// Complete Panchang result.
#[derive(Debug, Clone)]
pub struct PanchangResult {
    pub sun: SunData,
    pub vara: VaraInfo,
    pub tithi: TithiInfo,
    pub nakshatra: NakshatraInfo,
    pub yoga: YogaInfo,
    pub karana: KaranaInfo,
}
