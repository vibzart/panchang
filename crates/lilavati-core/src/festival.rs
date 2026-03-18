//! Festival date resolution engine.
//!
//! Resolves festival definitions (passed from Python/YAML) to Gregorian dates
//! using lunar month + tithi lookup. Each result includes a `reasoning` string
//! explaining WHY the festival falls on that specific date.
//!
//! Three resolution rules are supported:
//! - `tithi_at_sunrise`: The target tithi must prevail at sunrise in the specified
//!   lunar month. Adhik (intercalary) months are skipped — festivals are observed
//!   in the NIJA (regular) month only.
//! - `sankranti`: The festival date is when the Sun enters a specific Rashi.
//! - `nakshatra_at_sunrise`: The target nakshatra must prevail at sunrise,
//!   anchored around a Sankranti for seasonal context (nakshatras repeat ~monthly).
//!
//! Ekadashi computation adds the Vaishnava rule: Dashami must have ended before
//! Arunodaya (96 minutes before sunrise). If not, the Vaishnava date shifts by one day.

use crate::angles::normalize;
use crate::constants::{LUNAR_MONTH_NAMES, NAKSHATRA_NAMES, SANKRANTI_RASHI_INDEX, TITHI_NAMES};
use crate::ephemeris::{self, Planet};
use crate::julian;
use crate::lunar_month::{self, CalendarSystem, LunarMonthInfo};
use crate::sankranti::{self, SankrantiInfo};
use crate::sun;

const TITHI_SPAN: f64 = 12.0; // 360° / 30 tithis
const NAKSHATRA_SPAN: f64 = 360.0 / 27.0; // 13°20'

/// Minutes before sunrise for Arunodaya (Vaishnava Ekadashi rule).
const ARUNODAYA_MINUTES: f64 = 96.0;

// ============================================================================
// Types — passed from Python (loaded from YAML)
// ============================================================================

/// Festival definition passed from Python (loaded from YAML).
#[derive(Debug, Clone)]
pub struct FestivalDef {
    pub id: String,
    pub name: String,
    /// "tithi_at_sunrise", "sankranti", or "nakshatra_at_sunrise"
    pub rule: String,
    /// 1-12 lunar month number (for tithi_at_sunrise rule)
    pub lunar_month: u32,
    /// 1-30 tithi number (for tithi_at_sunrise rule)
    pub tithi: u32,
    /// 0-11 Sankranti index (for sankranti rule and nakshatra_at_sunrise anchor)
    pub sankranti_index: Option<u32>,
    /// 1-27 nakshatra number (for nakshatra_at_sunrise rule)
    pub nakshatra: Option<u32>,
}

/// Resolved festival occurrence with reasoning.
#[derive(Debug, Clone)]
pub struct FestivalOccurrence {
    pub festival_id: String,
    pub festival_name: String,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub sunrise_jd: f64,
    pub tithi_at_sunrise: u32,
    pub lunar_month_name: &'static str,
    pub is_adhik_month: bool,
    pub reasoning: String,
}

/// Ekadashi definition passed from Python.
#[derive(Debug, Clone)]
pub struct EkadashiDef {
    /// Lunar month 1-12.
    pub month: u32,
    /// Shukla Paksha Ekadashi name (e.g. "Kamada").
    pub shukla_name: String,
    /// Krishna Paksha Ekadashi name (e.g. "Varuthini").
    pub krishna_name: String,
}

/// Resolved Ekadashi with Smartha and Vaishnava dates.
#[derive(Debug, Clone)]
pub struct EkadashiOccurrence {
    pub name: String,
    pub lunar_month: u32,
    pub lunar_month_name: &'static str,
    pub paksha: &'static str,
    pub smartha_year: i32,
    pub smartha_month: u32,
    pub smartha_day: u32,
    pub smartha_sunrise_jd: f64,
    pub vaishnava_year: i32,
    pub vaishnava_month: u32,
    pub vaishnava_day: u32,
    pub vaishnava_sunrise_jd: f64,
    pub reasoning: String,
}

/// Resolved Vrat (fasting) date.
#[derive(Debug, Clone)]
pub struct VratOccurrence {
    pub vrat_type: String,
    pub name: String,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub sunrise_jd: f64,
    pub lunar_month_name: &'static str,
    pub paksha: &'static str,
}

// ============================================================================
// Helpers
// ============================================================================

/// Tithi angle: Moon tropical − Sun tropical, normalized to [0, 360).
fn tithi_angle(jd: f64) -> f64 {
    let moon = ephemeris::tropical_longitude(jd, Planet::Moon);
    let sun = ephemeris::tropical_longitude(jd, Planet::Sun);
    normalize(moon - sun)
}

/// Tithi number (1-30) at a given JD.
pub fn tithi_at_jd(jd: f64) -> u32 {
    let angle = tithi_angle(jd);
    (angle / TITHI_SPAN) as u32 + 1
}

/// Nakshatra number (1-27) at a given JD, based on Moon's sidereal longitude.
fn nakshatra_at_jd(jd: f64) -> u32 {
    let moon_sid = ephemeris::sidereal_longitude(jd, Planet::Moon);
    (moon_sid / NAKSHATRA_SPAN) as u32 + 1
}

/// Convert JD to local date components using UTC offset.
fn local_date(jd: f64, utc_offset: i32) -> julian::DateTimeComponents {
    let local_jd = jd + (utc_offset as f64) / 86400.0;
    julian::jd_to_datetime(local_jd)
}

/// Format a JD as local time "HH:MM".
fn format_local_time(jd: f64, utc_offset: i32) -> String {
    let dt = local_date(jd, utc_offset);
    format!("{:02}:{:02}", dt.hour, dt.minute)
}

/// Human-readable tithi name: "Shukla Saptami" or "Krishna Ashtami".
fn tithi_display_name(tithi_num: u32) -> String {
    let paksha = if tithi_num <= 15 { "Shukla" } else { "Krishna" };
    let name = TITHI_NAMES[(tithi_num - 1) as usize];
    format!("{} {}", paksha, name)
}

/// Find the first day within a lunar month where the given tithi prevails at sunrise.
/// Returns (sunrise_jd, local_date_components) or None.
fn find_tithi_at_sunrise(
    target_tithi: u32,
    lunar_month: &LunarMonthInfo,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<(f64, julian::DateTimeComponents)> {
    // Convert month start to local date for iteration
    let start_dt = local_date(lunar_month.start_jd, utc_offset);
    let base_midnight =
        julian::midnight_jd(start_dt.year, start_dt.month, start_dt.day, utc_offset);

    for day_offset in 0..35 {
        let midnight = base_midnight + day_offset as f64;
        let sunrise = sun::sunrise_jd(midnight, lat, lng, alt);

        // Only consider sunrises within the lunar month boundaries
        if sunrise > lunar_month.start_jd && sunrise <= lunar_month.end_jd {
            let t = tithi_at_jd(sunrise);
            if t == target_tithi {
                let dt = local_date(sunrise, utc_offset);
                return Some((sunrise, dt));
            }
        }
    }

    None
}

/// Find the day where the given tithi prevails at sunrise, searching ±20 days
/// around a Sankranti JD. Returns the match closest to the Sankranti.
///
/// This avoids Amant/Purnimant boundary issues by anchoring the search to the
/// Sankranti that names the month rather than to month boundary points.
fn find_tithi_at_sunrise_near_sankranti(
    target_tithi: u32,
    sankranti_jd: f64,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<(f64, julian::DateTimeComponents)> {
    let search_start_dt = local_date(sankranti_jd - 20.0, utc_offset);
    let base_midnight = julian::midnight_jd(
        search_start_dt.year,
        search_start_dt.month,
        search_start_dt.day,
        utc_offset,
    );

    let mut best: Option<(f64, f64, julian::DateTimeComponents)> = None; // (distance, sunrise, dt)

    for day_offset in 0..40 {
        let midnight = base_midnight + day_offset as f64;
        let sunrise = sun::sunrise_jd(midnight, lat, lng, alt);
        let t = tithi_at_jd(sunrise);
        if t == target_tithi {
            let distance = (sunrise - sankranti_jd).abs();
            if best.is_none() || distance < best.as_ref().unwrap().0 {
                let dt = local_date(sunrise, utc_offset);
                best = Some((distance, sunrise, dt));
            }
        }
    }

    best.map(|(_, sunrise, dt)| (sunrise, dt))
}

/// Find the day where the given nakshatra prevails at sunrise, searching ±20 days
/// around a Sankranti JD. Returns the match closest to the Sankranti.
///
/// Nakshatras repeat every ~27 days, so there will typically be 1 match within
/// the ±20 day window. Picking the closest ensures we get the right seasonal occurrence.
fn find_nakshatra_at_sunrise_near_sankranti(
    target_nakshatra: u32,
    sankranti_jd: f64,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<(f64, julian::DateTimeComponents)> {
    let search_start_dt = local_date(sankranti_jd - 20.0, utc_offset);
    let base_midnight = julian::midnight_jd(
        search_start_dt.year,
        search_start_dt.month,
        search_start_dt.day,
        utc_offset,
    );

    let mut best: Option<(f64, f64, julian::DateTimeComponents)> = None;

    for day_offset in 0..40 {
        let midnight = base_midnight + day_offset as f64;
        let sunrise = sun::sunrise_jd(midnight, lat, lng, alt);
        let n = nakshatra_at_jd(sunrise);
        if n == target_nakshatra {
            let distance = (sunrise - sankranti_jd).abs();
            if best.is_none() || distance < best.as_ref().unwrap().0 {
                let dt = local_date(sunrise, utc_offset);
                best = Some((distance, sunrise, dt));
            }
        }
    }

    best.map(|(_, sunrise, dt)| (sunrise, dt))
}

// ============================================================================
// Festival resolution
// ============================================================================

/// Resolve a tithi-at-sunrise festival to a Gregorian date.
///
/// Uses Sankranti-based search: find the Sankranti that names the lunar month,
/// then search ±20 days around it for the target tithi at sunrise. This avoids
/// all Amant/Purnimant boundary edge cases while giving correct results for both
/// naming conventions (since both name months after the same Sankranti).
fn resolve_tithi_festival(
    def: &FestivalDef,
    sankrantis: &[SankrantiInfo],
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<FestivalOccurrence> {
    // Map lunar month → Rashi index → Sankranti index
    let rashi_idx = (def.lunar_month - 1) as usize;
    let sankranti_idx = SANKRANTI_RASHI_INDEX.iter().position(|&r| r == rashi_idx)?;
    let sankranti = sankrantis.get(sankranti_idx)?;

    let (sunrise, dt) =
        find_tithi_at_sunrise_near_sankranti(def.tithi, sankranti.jd, lat, lng, alt, utc_offset)?;

    let month_name = LUNAR_MONTH_NAMES[(def.lunar_month - 1) as usize];
    let s_dt = local_date(sankranti.jd, utc_offset);

    let reasoning = format!(
        "{} {} (Tithi {}) prevails at sunrise ({}) on {}-{:02}-{:02}. \
         Lunar month {} determined by {} ({}) on {}-{:02}-{:02}.",
        month_name,
        tithi_display_name(def.tithi),
        def.tithi,
        format_local_time(sunrise, utc_offset),
        dt.year,
        dt.month,
        dt.day,
        month_name,
        sankranti.name,
        sankranti.rashi,
        s_dt.year,
        s_dt.month,
        s_dt.day,
    );

    Some(FestivalOccurrence {
        festival_id: def.id.clone(),
        festival_name: def.name.clone(),
        year: dt.year,
        month: dt.month,
        day: dt.day,
        sunrise_jd: sunrise,
        tithi_at_sunrise: def.tithi,
        lunar_month_name: month_name,
        is_adhik_month: false,
        reasoning,
    })
}

/// Resolve a Sankranti-based festival to a Gregorian date.
fn resolve_sankranti_festival(
    def: &FestivalDef,
    sankrantis: &[SankrantiInfo],
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<FestivalOccurrence> {
    let idx = def.sankranti_index? as usize;
    let s = sankrantis.get(idx)?;

    let dt = local_date(s.jd, utc_offset);

    // Compute sunrise for that date to report tithi
    let midnight = julian::midnight_jd(dt.year, dt.month, dt.day, utc_offset);
    let sunrise = sun::sunrise_jd(midnight, lat, lng, alt);
    let tithi_num = tithi_at_jd(sunrise);

    let reasoning = format!(
        "Sun enters {} (sidereal {:.1}°) on {}-{:02}-{:02} at {}.",
        s.rashi,
        s.target_longitude,
        dt.year,
        dt.month,
        dt.day,
        format_local_time(s.jd, utc_offset),
    );

    Some(FestivalOccurrence {
        festival_id: def.id.clone(),
        festival_name: def.name.clone(),
        year: dt.year,
        month: dt.month,
        day: dt.day,
        sunrise_jd: sunrise,
        tithi_at_sunrise: tithi_num,
        lunar_month_name: "",
        is_adhik_month: false,
        reasoning,
    })
}

/// Resolve a nakshatra-at-sunrise festival to a Gregorian date.
///
/// Searches ±20 days around the Sankranti anchor for the day when the target
/// nakshatra prevails at sunrise. The Sankranti anchor provides seasonal context
/// since nakshatras repeat every ~27 days.
fn resolve_nakshatra_festival(
    def: &FestivalDef,
    sankrantis: &[SankrantiInfo],
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<FestivalOccurrence> {
    let target_nak = def.nakshatra?;
    let anchor_idx = def.sankranti_index? as usize;
    let sankranti = sankrantis.get(anchor_idx)?;

    let (sunrise, dt) = find_nakshatra_at_sunrise_near_sankranti(
        target_nak,
        sankranti.jd,
        lat,
        lng,
        alt,
        utc_offset,
    )?;

    let nak_name = NAKSHATRA_NAMES[(target_nak - 1) as usize];
    let tithi_num = tithi_at_jd(sunrise);
    let s_dt = local_date(sankranti.jd, utc_offset);

    let reasoning = format!(
        "Nakshatra {} (#{}) prevails at sunrise ({}) on {}-{:02}-{:02}. \
         Anchored to {} ({}) on {}-{:02}-{:02}.",
        nak_name,
        target_nak,
        format_local_time(sunrise, utc_offset),
        dt.year,
        dt.month,
        dt.day,
        sankranti.name,
        sankranti.rashi,
        s_dt.year,
        s_dt.month,
        s_dt.day,
    );

    Some(FestivalOccurrence {
        festival_id: def.id.clone(),
        festival_name: def.name.clone(),
        year: dt.year,
        month: dt.month,
        day: dt.day,
        sunrise_jd: sunrise,
        tithi_at_sunrise: tithi_num,
        lunar_month_name: "",
        is_adhik_month: false,
        reasoning,
    })
}

// ============================================================================
// Public API
// ============================================================================

/// Compute festival dates for the given year from festival definitions.
///
/// Festival definitions are passed from Python (loaded from YAML).
/// Both `tithi_at_sunrise` and `sankranti` rules are supported.
pub fn compute_festivals(
    defs: &[FestivalDef],
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Vec<FestivalOccurrence> {
    ephemeris::init(None);

    // Sankranti-based search: find the Sankranti that names each festival's
    // lunar month, then search around it for the target tithi at sunrise.
    // This avoids Amant/Purnimant boundary edge cases entirely.
    let sankrantis = sankranti::compute_sankrantis(year);

    let mut results = Vec::with_capacity(defs.len());

    for def in defs {
        let occurrence = match def.rule.as_str() {
            "tithi_at_sunrise" => {
                resolve_tithi_festival(def, &sankrantis, lat, lng, alt, utc_offset)
            }
            "sankranti" => resolve_sankranti_festival(def, &sankrantis, lat, lng, alt, utc_offset),
            "nakshatra_at_sunrise" => {
                resolve_nakshatra_festival(def, &sankrantis, lat, lng, alt, utc_offset)
            }
            _ => None,
        };

        if let Some(occ) = occurrence {
            // Only include festivals that fall within the requested year
            if occ.year == year {
                results.push(occ);
            }
        }
    }

    // Sort chronologically
    results.sort_by(|a, b| a.sunrise_jd.partial_cmp(&b.sunrise_jd).unwrap());
    results
}

/// Compute all 24 Ekadashis for a year (2 per lunar month).
///
/// Each Ekadashi has both Smartha and Vaishnava dates.
/// The Vaishnava date may differ by one day if Dashami persists at Arunodaya.
pub fn compute_ekadashis(
    ekadashi_defs: &[EkadashiDef],
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Vec<EkadashiOccurrence> {
    ephemeris::init(None);

    // Use Sankranti-based search, same approach as compute_festivals.
    let sankrantis = sankranti::compute_sankrantis(year);
    let mut results = Vec::new();

    for def in ekadashi_defs {
        // Map lunar month → Sankranti
        let rashi_idx = (def.month - 1) as usize;
        let sankranti_idx = match SANKRANTI_RASHI_INDEX.iter().position(|&r| r == rashi_idx) {
            Some(i) => i,
            None => continue,
        };
        let sankranti = match sankrantis.get(sankranti_idx) {
            Some(s) => s,
            None => continue,
        };

        let month_name = LUNAR_MONTH_NAMES[(def.month - 1) as usize];

        // Shukla Ekadashi (tithi 11)
        if let Some(ek) = resolve_ekadashi(
            sankranti.jd,
            def.month,
            month_name,
            true,
            &def.shukla_name,
            lat,
            lng,
            alt,
            utc_offset,
        ) {
            if ek.smartha_year == year {
                results.push(ek);
            }
        }

        // Krishna Ekadashi (tithi 26)
        if let Some(ek) = resolve_ekadashi(
            sankranti.jd,
            def.month,
            month_name,
            false,
            &def.krishna_name,
            lat,
            lng,
            alt,
            utc_offset,
        ) {
            if ek.smartha_year == year {
                results.push(ek);
            }
        }
    }

    results.sort_by(|a, b| {
        a.smartha_sunrise_jd
            .partial_cmp(&b.smartha_sunrise_jd)
            .unwrap()
    });
    results
}

/// Resolve a single Ekadashi using Sankranti-based search.
#[allow(clippy::too_many_arguments)]
fn resolve_ekadashi(
    sankranti_jd: f64,
    lunar_month_num: u32,
    lunar_month_name: &'static str,
    is_shukla: bool,
    name: &str,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<EkadashiOccurrence> {
    let target_tithi: u32 = if is_shukla { 11 } else { 26 };
    let dashami_tithi: u32 = target_tithi - 1;

    let (smartha_sunrise, smartha_dt) = find_tithi_at_sunrise_near_sankranti(
        target_tithi,
        sankranti_jd,
        lat,
        lng,
        alt,
        utc_offset,
    )?;

    // Vaishnava rule: check if Dashami persists at Arunodaya
    let arunodaya_jd = smartha_sunrise - ARUNODAYA_MINUTES / 1440.0;
    let tithi_at_arunodaya = tithi_at_jd(arunodaya_jd);
    let dashami_at_arunodaya = tithi_at_arunodaya == dashami_tithi;

    let (vaishnava_sunrise, vaishnava_dt) = if dashami_at_arunodaya {
        // Shift to next day
        let next_midnight = julian::midnight_jd(
            smartha_dt.year,
            smartha_dt.month,
            smartha_dt.day,
            utc_offset,
        ) + 1.0;
        let next_sunrise = sun::sunrise_jd(next_midnight, lat, lng, alt);
        let next_dt = local_date(next_sunrise, utc_offset);
        (next_sunrise, next_dt)
    } else {
        (smartha_sunrise, smartha_dt)
    };

    let paksha: &'static str = if is_shukla { "Shukla" } else { "Krishna" };

    let reasoning = if dashami_at_arunodaya {
        format!(
            "{} Ekadashi (Tithi {}) prevails at sunrise ({}) on {}-{:02}-{:02}. \
             Dashami (Tithi {}) persists at Arunodaya (96 min before sunrise), \
             so Vaishnava Ekadashi shifts to {}-{:02}-{:02}. \
             Lunar month: {}.",
            paksha,
            target_tithi,
            format_local_time(smartha_sunrise, utc_offset),
            smartha_dt.year,
            smartha_dt.month,
            smartha_dt.day,
            dashami_tithi,
            vaishnava_dt.year,
            vaishnava_dt.month,
            vaishnava_dt.day,
            lunar_month_name,
        )
    } else {
        format!(
            "{} Ekadashi (Tithi {}) prevails at sunrise ({}) on {}-{:02}-{:02}. \
             Dashami ended before Arunodaya — Smartha and Vaishnava dates are the same. \
             Lunar month: {}.",
            paksha,
            target_tithi,
            format_local_time(smartha_sunrise, utc_offset),
            smartha_dt.year,
            smartha_dt.month,
            smartha_dt.day,
            lunar_month_name,
        )
    };

    Some(EkadashiOccurrence {
        name: name.to_string(),
        lunar_month: lunar_month_num,
        lunar_month_name,
        paksha,
        smartha_year: smartha_dt.year,
        smartha_month: smartha_dt.month,
        smartha_day: smartha_dt.day,
        smartha_sunrise_jd: smartha_sunrise,
        vaishnava_year: vaishnava_dt.year,
        vaishnava_month: vaishnava_dt.month,
        vaishnava_day: vaishnava_dt.day,
        vaishnava_sunrise_jd: vaishnava_sunrise,
        reasoning,
    })
}

/// Compute monthly Vrat (fasting) dates for a year.
///
/// Returns Pradosh Vrat (Shukla & Krishna Trayodashi), Sankashti Chaturthi
/// (Krishna Chaturthi), Amavasya, and Purnima dates for each lunar month.
pub fn compute_vrat_dates(
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Vec<VratOccurrence> {
    ephemeris::init(None);

    let lunar_months = lunar_month::compute_lunar_months(year, CalendarSystem::Amant);
    let mut results = Vec::new();

    // (tithi, vrat_type, paksha)
    let vrat_tithis: [(u32, &str, &str); 5] = [
        (13, "Pradosh Vrat", "Shukla"),         // Shukla Trayodashi
        (28, "Pradosh Vrat", "Krishna"),        // Krishna Trayodashi
        (19, "Sankashti Chaturthi", "Krishna"), // Krishna Chaturthi
        (30, "Amavasya", "Krishna"),            // Amavasya
        (15, "Purnima Vrat", "Shukla"),         // Purnima
    ];

    for month_info in &lunar_months {
        if month_info.is_adhik {
            continue;
        }

        for &(target_tithi, vrat_type, paksha) in &vrat_tithis {
            if let Some((sunrise, dt)) =
                find_tithi_at_sunrise(target_tithi, month_info, lat, lng, alt, utc_offset)
            {
                if dt.year == year {
                    results.push(VratOccurrence {
                        vrat_type: vrat_type.to_string(),
                        name: format!("{} ({})", vrat_type, month_info.name),
                        year: dt.year,
                        month: dt.month,
                        day: dt.day,
                        sunrise_jd: sunrise,
                        lunar_month_name: month_info.name,
                        paksha,
                    });
                }
            }
        }
    }

    results.sort_by(|a, b| a.sunrise_jd.partial_cmp(&b.sunrise_jd).unwrap());
    results
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Delhi coordinates
    const LAT: f64 = 28.6139;
    const LNG: f64 = 77.2090;
    const ALT: f64 = 0.0;
    const IST: i32 = 19800; // +5:30

    fn ekadashi_defs() -> Vec<EkadashiDef> {
        vec![
            EkadashiDef {
                month: 1,
                shukla_name: "Kamada".into(),
                krishna_name: "Varuthini".into(),
            },
            EkadashiDef {
                month: 2,
                shukla_name: "Mohini".into(),
                krishna_name: "Apara".into(),
            },
            EkadashiDef {
                month: 3,
                shukla_name: "Nirjala".into(),
                krishna_name: "Yogini".into(),
            },
            EkadashiDef {
                month: 4,
                shukla_name: "Devshayani".into(),
                krishna_name: "Kamika".into(),
            },
            EkadashiDef {
                month: 5,
                shukla_name: "Shravana Putrada".into(),
                krishna_name: "Aja".into(),
            },
            EkadashiDef {
                month: 6,
                shukla_name: "Parsva".into(),
                krishna_name: "Indira".into(),
            },
            EkadashiDef {
                month: 7,
                shukla_name: "Papankusha".into(),
                krishna_name: "Rama".into(),
            },
            EkadashiDef {
                month: 8,
                shukla_name: "Prabodhini".into(),
                krishna_name: "Utpanna".into(),
            },
            EkadashiDef {
                month: 9,
                shukla_name: "Mokshada".into(),
                krishna_name: "Saphala".into(),
            },
            EkadashiDef {
                month: 10,
                shukla_name: "Pausha Putrada".into(),
                krishna_name: "Shattila".into(),
            },
            EkadashiDef {
                month: 11,
                shukla_name: "Jaya".into(),
                krishna_name: "Vijaya".into(),
            },
            EkadashiDef {
                month: 12,
                shukla_name: "Amalaki".into(),
                krishna_name: "Papamochani".into(),
            },
        ]
    }

    #[test]
    fn test_diwali_2026() {
        ephemeris::init(None);
        let defs = vec![FestivalDef {
            id: "diwali".into(),
            name: "Diwali".into(),
            rule: "tithi_at_sunrise".into(),
            lunar_month: 8, // Kartik
            tithi: 30,      // Amavasya
            sankranti_index: None,
            nakshatra: None,
        }];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 1);
        let diwali = &results[0];

        // Diwali 2026 — 2026 has an Adhik Maas, so Diwali shifts later.
        // Expected around Oct-Nov (Kartik Amavasya in Amant Ashwin month).
        assert!(
            diwali.month >= 10 && diwali.month <= 11,
            "Diwali month: {} (expected Oct or Nov)",
            diwali.month
        );
        assert_eq!(diwali.tithi_at_sunrise, 30);
        assert!(diwali.reasoning.contains("Kartik"));
    }

    #[test]
    fn test_holi_2026() {
        ephemeris::init(None);
        let defs = vec![FestivalDef {
            id: "holi".into(),
            name: "Holi".into(),
            rule: "tithi_at_sunrise".into(),
            lunar_month: 12, // Phalguna
            tithi: 15,       // Purnima
            sankranti_index: None,
            nakshatra: None,
        }];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 1);
        let holi = &results[0];

        // Holi 2026 — Phalguna Purnima, expected Feb or Mar
        assert!(
            holi.month >= 2 && holi.month <= 3,
            "Holi month: {} (expected Feb or Mar)",
            holi.month
        );
        assert!(holi.reasoning.contains("Phalguna"));
    }

    #[test]
    fn test_makar_sankranti_festival() {
        ephemeris::init(None);
        let defs = vec![FestivalDef {
            id: "makar_sankranti".into(),
            name: "Makar Sankranti".into(),
            rule: "sankranti".into(),
            lunar_month: 0,
            tithi: 0,
            sankranti_index: Some(0),
            nakshatra: None,
        }];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 1);
        let ms = &results[0];

        // Makar Sankranti 2026 ~Jan 14
        assert_eq!(ms.month, 1);
        assert!(
            ms.day >= 13 && ms.day <= 15,
            "Makar Sankranti day: {}",
            ms.day
        );
        assert!(ms.reasoning.contains("Makara"));
    }

    #[test]
    fn test_multiple_festivals_sorted() {
        ephemeris::init(None);
        let defs = vec![
            FestivalDef {
                id: "diwali".into(),
                name: "Diwali".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 8,
                tithi: 30,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "holi".into(),
                name: "Holi".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 12,
                tithi: 15,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "ram_navami".into(),
                name: "Ram Navami".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 1, // Chaitra
                tithi: 9,       // Shukla Navami
                sankranti_index: None,
                nakshatra: None,
            },
        ];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 3, "Expected 3 festivals");

        // Should be sorted chronologically
        for i in 1..results.len() {
            assert!(
                results[i].sunrise_jd > results[i - 1].sunrise_jd,
                "{} should come after {}",
                results[i].festival_name,
                results[i - 1].festival_name
            );
        }

        // All should have reasoning
        for r in &results {
            assert!(
                !r.reasoning.is_empty(),
                "Reasoning for {} should not be empty",
                r.festival_name
            );
        }
    }

    #[test]
    fn test_all_festivals_have_valid_dates() {
        ephemeris::init(None);
        let defs = vec![
            FestivalDef {
                id: "ganesh_chaturthi".into(),
                name: "Ganesh Chaturthi".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 6,
                tithi: 4,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "dussehra".into(),
                name: "Dussehra".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 7,
                tithi: 10,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "janmashtami".into(),
                name: "Janmashtami".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 6,
                tithi: 23,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "maha_shivaratri".into(),
                name: "Maha Shivaratri".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 11,
                tithi: 29,
                sankranti_index: None,
                nakshatra: None,
            },
        ];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST);

        // All 4 should resolve (might be 3 if one falls across a year boundary)
        assert!(
            results.len() >= 3 && results.len() <= 4,
            "Expected 3-4 festivals, got {}",
            results.len()
        );

        for r in &results {
            assert_eq!(r.year, 2026);
            assert!(r.month >= 1 && r.month <= 12);
            assert!(r.day >= 1 && r.day <= 31);
            assert!(!r.reasoning.is_empty());
        }
    }

    #[test]
    fn debug_print_festival_dates() {
        ephemeris::init(None);

        let defs = vec![
            FestivalDef {
                id: "diwali".into(),
                name: "Diwali".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 8,
                tithi: 30,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "holi".into(),
                name: "Holi".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 12,
                tithi: 15,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "janmashtami".into(),
                name: "Janmashtami".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 6,
                tithi: 23,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "ganesh_chaturthi".into(),
                name: "Ganesh Chaturthi".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 6,
                tithi: 4,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "dussehra".into(),
                name: "Dussehra".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 7,
                tithi: 10,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "maha_shivaratri".into(),
                name: "Maha Shivaratri".into(),
                rule: "tithi_at_sunrise".into(),
                lunar_month: 11,
                tithi: 29,
                sankranti_index: None,
                nakshatra: None,
            },
            FestivalDef {
                id: "makar_sankranti".into(),
                name: "Makar Sankranti".into(),
                rule: "sankranti".into(),
                lunar_month: 0,
                tithi: 0,
                sankranti_index: Some(0),
                nakshatra: None,
            },
        ];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST);
        eprintln!("\n--- Resolved Festivals (Sankranti-based) ---");
        for r in &results {
            eprintln!(
                "  {}: {}-{:02}-{:02} | {}",
                r.festival_name, r.year, r.month, r.day, r.reasoning
            );
        }
        eprintln!("  Total: {}", results.len());
    }

    #[test]
    fn test_ekadashis_count() {
        ephemeris::init(None);
        let defs = ekadashi_defs();
        let results = compute_ekadashis(&defs, 2026, LAT, LNG, ALT, IST);

        // 2 per month × 12 months = 24, but some may not fall in 2026
        assert!(
            results.len() >= 20 && results.len() <= 26,
            "Got {} Ekadashis, expected 20-26",
            results.len()
        );

        // All should have names and reasoning
        for ek in &results {
            assert!(!ek.name.is_empty());
            assert!(!ek.reasoning.is_empty());
            assert!(ek.paksha == "Shukla" || ek.paksha == "Krishna");
        }
    }

    #[test]
    fn test_ekadashi_vaishnava_dates() {
        ephemeris::init(None);
        let defs = ekadashi_defs();
        let results = compute_ekadashis(&defs, 2026, LAT, LNG, ALT, IST);

        for ek in &results {
            // Vaishnava date should be same as or 1 day after Smartha
            let smartha_jd = ek.smartha_sunrise_jd;
            let vaishnava_jd = ek.vaishnava_sunrise_jd;
            let diff = vaishnava_jd - smartha_jd;
            assert!(
                diff >= -0.1 && diff <= 1.5,
                "Vaishnava - Smartha diff: {:.2} days for {}",
                diff,
                ek.name
            );
        }
    }

    #[test]
    fn test_vrat_dates_count() {
        ephemeris::init(None);
        let results = compute_vrat_dates(2026, LAT, LNG, ALT, IST);

        // 5 vrat types × ~12 months ≈ 60, minus boundary months
        assert!(
            results.len() >= 45 && results.len() <= 65,
            "Got {} vrat dates, expected 45-65",
            results.len()
        );

        for v in &results {
            assert_eq!(v.year, 2026, "Year for {}", v.name);
            assert!(v.paksha == "Shukla" || v.paksha == "Krishna");
        }
    }

    #[test]
    fn test_nakshatra_onam_2026() {
        ephemeris::init(None);
        let defs = vec![FestivalDef {
            id: "onam".into(),
            name: "Onam".into(),
            rule: "nakshatra_at_sunrise".into(),
            lunar_month: 0,
            tithi: 0,
            sankranti_index: Some(7), // Simha Sankranti (~Aug 17)
            nakshatra: Some(22),      // Shravana (Thiruvonam)
        }];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 1, "Onam should resolve");
        let onam = &results[0];

        // Onam 2026 should be around Aug-Sep (Simha month)
        assert!(
            onam.month >= 8 && onam.month <= 9,
            "Onam month: {} (expected Aug or Sep)",
            onam.month
        );
        assert!(onam.reasoning.contains("Shravana"));
        assert!(onam.reasoning.contains("Simha"));
        eprintln!(
            "Onam 2026: {}-{:02}-{:02} | {}",
            onam.year, onam.month, onam.day, onam.reasoning
        );
    }

    #[test]
    fn test_nakshatra_festival_resolution() {
        ephemeris::init(None);
        // Test Mrigashira nakshatra near Mesha Sankranti
        let defs = vec![FestivalDef {
            id: "test_nak".into(),
            name: "Test Nakshatra".into(),
            rule: "nakshatra_at_sunrise".into(),
            lunar_month: 0,
            tithi: 0,
            sankranti_index: Some(3), // Mesha Sankranti (~Apr 14)
            nakshatra: Some(5),       // Mrigashira
        }];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST);
        assert_eq!(results.len(), 1, "Nakshatra festival should resolve");
        assert!(results[0].reasoning.contains("Mrigashira"));
        assert_eq!(results[0].year, 2026);
    }
}
