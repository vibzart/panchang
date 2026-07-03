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
use crate::observance::{self, AdhikaMaasa, Kaala, Priority};
use crate::sankranti::{self, SankrantiInfo};
use crate::search;
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
    /// Observance priority — paraviddha (default), puurvaviddha, or vyapti.
    /// Only applies to `tithi_at_sunrise` festivals.
    pub priority: Priority,
    /// Kaala window for the `vyapti` priority. Ignored for other priorities.
    pub kaala: Kaala,
    /// Tie-break when the tithi has kaala-vyapti on BOTH candidate days.
    /// `false` (default) → later day wins (paraviddha fallback, e.g.
    /// Akshaya Tritiya). `true` → earlier day wins (e.g. Vijayadashami:
    /// "dinadvaye aparahna-vyaptau purva", Nirnaya Sindhu).
    pub vyapti_tie_purva: bool,
    /// How to handle the adhika (intercalary) month for this festival.
    pub adhika_maasa: AdhikaMaasa,
}

impl FestivalDef {
    /// Construct a basic `tithi_at_sunrise` festival with default observance
    /// rules (paraviddha / sunrise / nija). Convenience for tests and simple
    /// callers; YAML-driven construction fills in all fields explicitly.
    pub fn new_tithi(
        id: impl Into<String>,
        name: impl Into<String>,
        lunar_month: u32,
        tithi: u32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            rule: "tithi_at_sunrise".to_string(),
            lunar_month,
            tithi,
            sankranti_index: None,
            nakshatra: None,
            priority: Priority::default(),
            kaala: Kaala::default(),
            vyapti_tie_purva: false,
            adhika_maasa: AdhikaMaasa::default(),
        }
    }

    /// Apply observance overrides to a tithi festival built with `new_tithi`.
    pub fn with_observance(mut self, priority: Priority, kaala: Kaala) -> Self {
        self.priority = priority;
        self.kaala = kaala;
        self
    }
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
    /// Priority rule that produced this date (paraviddha/puurvaviddha/vyapti).
    pub priority_applied: &'static str,
    /// Kaala that was checked for vyapti resolution. Empty for non-vyapti rules.
    pub kaala_applied: &'static str,
    /// Alternate date that the *opposite* convention would have produced.
    /// For vyapti/puurvaviddha results, this is the paraviddha day.
    /// For paraviddha results where a shastric alternate exists, this is the
    /// earlier-day candidate. `None` when there is no meaningful alternate
    /// (e.g., tithi at sunrise on exactly one day in the window).
    pub alternate: Option<AlternateObservance>,
}

/// A secondary observance date produced by the opposite convention.
/// Attached to a `FestivalOccurrence` so callers can surface both to the user.
#[derive(Debug, Clone)]
pub struct AlternateObservance {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub sunrise_jd: f64,
    pub priority: &'static str,
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
    /// True when this Ekadashi falls in an Adhika (intercalary) month.
    /// Adhika-month Ekadashis carry the universal names Padmini (Shukla)
    /// and Parama (Krishna) regardless of which month is doubled.
    pub is_adhik: bool,
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

/// Search for a specific tithi at sunrise within a JD range [start_jd, end_jd].
/// Returns the first match found (scanning day by day from start to end).
/// Search for a specific tithi at sunrise within a JD range [start_jd, end_jd].
///
/// If the tithi is found prevailing at sunrise on any day, returns that day.
///
/// If the tithi is **kshaya** (it starts after one sunrise and ends before the
/// next — never prevailing at any sunrise), the Dharmashastra rule applies:
/// the festival is observed on the day whose sunrise has the **preceding tithi**.
/// This is the standard panchang convention used by Drik Panchang and others.
fn find_tithi_at_sunrise_in_range(
    target_tithi: u32,
    start_jd: f64,
    end_jd: f64,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<(f64, julian::DateTimeComponents)> {
    let start_dt = local_date(start_jd, utc_offset);
    let mut midnight = julian::midnight_jd(start_dt.year, start_dt.month, start_dt.day, utc_offset);

    let preceding_tithi = if target_tithi == 1 {
        30
    } else {
        target_tithi - 1
    };
    let mut preceding_candidate: Option<(f64, julian::DateTimeComponents)> = None;

    while midnight <= end_jd {
        let sunrise = sun::sunrise_jd(midnight, lat, lng, alt);
        if sunrise > end_jd {
            break;
        }
        let t = tithi_at_jd(sunrise);

        // A sunrise BEFORE the month-start moment (the month's first civil
        // day, when the boundary Amavasya ends later that day) carries the
        // PREVIOUS month's tithi. It can never be an exact match — for
        // tithi-30 festivals it would wrongly match the previous month's
        // Amavasya (Diwali 2026 resolved to Oct 10 instead of Nov 8 that
        // way). It IS still the valid kshaya-fallback day for tithi 1:
        // Pratipada that begins after this sunrise and ends before the
        // next one (e.g. Chaitra Navaratri 2026 = Mar 19).
        if t == target_tithi && sunrise >= start_jd {
            let dt = local_date(sunrise, utc_offset);
            return Some((sunrise, dt));
        }

        // Track FIRST day with the preceding tithi (for kshaya fallback).
        // We want the first occurrence because a kshaya tithi is skipped
        // right after its preceding tithi — not at the end of the month.
        if t == preceding_tithi && preceding_candidate.is_none() {
            let dt = local_date(sunrise, utc_offset);
            preceding_candidate = Some((sunrise, dt));
        }

        midnight += 1.0;
    }

    // Tithi was kshaya (never prevailed at sunrise) — use preceding tithi day
    preceding_candidate
}

/// Resolve a tithi-at-sunrise festival to a Gregorian date.
///
/// Two-pass algorithm:
///
/// 1. Find the **natural paraviddha day** using lunar-month-bounded search.
///    This is the day where the target tithi prevails at sunrise.
/// 2. Apply the `priority` observance rule to shift (or keep) that day:
///    - `Paraviddha` → keep the natural day (udayatithi default).
///    - `Puurvaviddha` → shift one day earlier (where tithi begins).
///    - `Vyapti(kaala)` → check which of (earlier, natural) has target tithi
///      present during the specified `kaala` window. Earlier day wins only if
///      it alone has vyapti; otherwise natural day wins.
///
/// The natural-day search falls back to the Sankranti-based ±20 day search
/// when lunar-month boundaries don't cover the edge case.
#[allow(clippy::too_many_arguments)]
fn resolve_tithi_festival(
    def: &FestivalDef,
    sankrantis: &[SankrantiInfo],
    lunar_months: &[LunarMonthInfo],
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
    calendar_system: CalendarSystem,
) -> Option<FestivalOccurrence> {
    // Month numbers in festival definitions are AMANT. For display, the
    // Purnimant system names the Krishna paksha of amant month N as month
    // N+1 (e.g. amant Ashwin Krishna Trayodashi = purnimant Kartik Krishna
    // Trayodashi — "Dhanteras in Kartik").
    let month_name = if calendar_system == CalendarSystem::Purnimant && def.tithi > 15 {
        LUNAR_MONTH_NAMES[(def.lunar_month % 12) as usize]
    } else {
        LUNAR_MONTH_NAMES[(def.lunar_month - 1) as usize]
    };

    // Determine the anchor Sankranti for this lunar month
    let rashi_idx = (def.lunar_month - 1) as usize;
    let sankranti_idx = SANKRANTI_RASHI_INDEX.iter().position(|&r| r == rashi_idx)?;
    let sankranti = sankrantis.get(sankranti_idx)?;
    let s_dt = local_date(sankranti.jd, utc_offset);

    // Always use Amant boundaries for festival date resolution — Amant month N
    // (Amavasya→Amavasya) contains both Shukla + Krishna Paksha of month N.
    // The `calendar_system` parameter only affects month labeling on display.
    // The windows are computed once by the caller and shared across defs.

    // ---- Select target lunar month(s) per adhika_maasa policy ----
    let target_month = select_target_month(
        lunar_months,
        def.lunar_month,
        def.adhika_maasa,
        sankranti.jd,
    );

    // ---- Step 1: find natural (paraviddha) day ----
    let (natural_sunrise, natural_dt, is_fallback, is_adhik) = if let Some(lm) = target_month {
        match find_tithi_at_sunrise_in_range(
            def.tithi,
            lm.start_jd,
            lm.end_jd,
            lat,
            lng,
            alt,
            utc_offset,
        ) {
            Some((s, d)) => (s, d, false, lm.is_adhik),
            None => {
                let (s, d) = find_tithi_at_sunrise_near_sankranti(
                    def.tithi,
                    sankranti.jd,
                    lat,
                    lng,
                    alt,
                    utc_offset,
                )?;
                (s, d, true, false)
            }
        }
    } else {
        // AdhikaMaasa::Adhika but no adhika month this year, or similar — skip.
        return None;
    };

    // ---- Step 2: apply observance rule ----
    apply_observance_rule(
        def,
        natural_sunrise,
        natural_dt,
        month_name,
        &s_dt,
        sankranti,
        is_fallback,
        is_adhik,
        lat,
        lng,
        alt,
        utc_offset,
    )
}

/// Pick the lunar month to search within, honoring the `adhika_maasa` policy.
///
/// Returns `None` when the policy cannot be satisfied (e.g., `Adhika` requested
/// but no adhika month exists this year). When policy calls for *both* nija and
/// adhika observance (`AdhikaAndNija`), this returns the nija month; emitting
/// two occurrences for that case is a future enhancement and not yet wired.
fn select_target_month(
    lunar_months: &[LunarMonthInfo],
    month_num: u32,
    policy: AdhikaMaasa,
    anchor_jd: f64,
) -> Option<&LunarMonthInfo> {
    // The same month number can appear TWICE in a year's window list (one
    // instance at each year edge — e.g. Margashirsha ending in early January
    // belongs to both year N-1's and year N's lists). Pick the instance that
    // contains the anchor Sankranti (a Sankranti always falls inside the
    // month it names); fall back to the first instance for safety.
    let nija = lunar_months
        .iter()
        .find(|m| {
            m.number == month_num && !m.is_adhik && m.start_jd <= anchor_jd && anchor_jd < m.end_jd
        })
        .or_else(|| {
            lunar_months
                .iter()
                .find(|m| m.number == month_num && !m.is_adhik)
        });
    // An adhika month contains no Sankranti by definition; it immediately
    // precedes its nija twin, so pick the one whose window ends closest
    // before the anchor.
    let adhika = lunar_months
        .iter()
        .filter(|m| m.number == month_num && m.is_adhik && m.start_jd <= anchor_jd)
        .max_by(|a, b| a.start_jd.partial_cmp(&b.start_jd).unwrap())
        .or_else(|| {
            lunar_months
                .iter()
                .find(|m| m.number == month_num && m.is_adhik)
        });

    match policy {
        AdhikaMaasa::Nija => nija,
        AdhikaMaasa::Adhika => adhika,
        AdhikaMaasa::AdhikaIfExists => adhika.or(nija),
        // AdhikaAndNija: return nija; adhika occurrence would need to be emitted
        // as a separate result, which is a larger refactor. Flag for follow-up.
        AdhikaMaasa::AdhikaAndNija => nija,
    }
}

/// Dispatch on `def.priority` to produce the final festival occurrence.
///
/// `natural_sunrise`/`natural_dt` is the paraviddha candidate (tithi at sunrise).
/// The other candidate is the day immediately before.
#[allow(clippy::too_many_arguments)]
fn apply_observance_rule(
    def: &FestivalDef,
    natural_sunrise: f64,
    natural_dt: julian::DateTimeComponents,
    month_name: &'static str,
    s_dt: &julian::DateTimeComponents,
    sankranti: &SankrantiInfo,
    is_fallback: bool,
    is_adhik: bool,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<FestivalOccurrence> {
    let paraviddha_reason = format_paraviddha_reason(
        month_name,
        def.tithi,
        natural_sunrise,
        &natural_dt,
        is_fallback,
        sankranti,
        s_dt,
        utc_offset,
    );

    match def.priority {
        Priority::Paraviddha => Some(FestivalOccurrence {
            festival_id: def.id.clone(),
            festival_name: def.name.clone(),
            year: natural_dt.year,
            month: natural_dt.month,
            day: natural_dt.day,
            sunrise_jd: natural_sunrise,
            tithi_at_sunrise: def.tithi,
            lunar_month_name: month_name,
            is_adhik_month: is_adhik,
            reasoning: paraviddha_reason,
            priority_applied: Priority::Paraviddha.as_str(),
            kaala_applied: "",
            alternate: None,
        }),

        Priority::Puurvaviddha => {
            // Shift one day earlier. The earlier day should have the preceding
            // tithi at sunrise; the target tithi begins during that day.
            let (earlier_sunrise, earlier_dt) =
                observance::previous_day_sunrise(natural_sunrise, utc_offset, lat, lng, alt);
            let earlier_tithi = tithi_at_jd(earlier_sunrise);

            let earlier_reason = format!(
                "{} {} (Tithi {}) begins during {}-{:02}-{:02} (tithi at its sunrise: {}). \
                 Puurvaviddha rule: observe on the day the tithi begins, not the day it \
                 prevails at sunrise. Paraviddha alternate would be {}-{:02}-{:02}.",
                month_name,
                tithi_display_name(def.tithi),
                def.tithi,
                earlier_dt.year,
                earlier_dt.month,
                earlier_dt.day,
                tithi_display_name(earlier_tithi),
                natural_dt.year,
                natural_dt.month,
                natural_dt.day,
            );

            Some(FestivalOccurrence {
                festival_id: def.id.clone(),
                festival_name: def.name.clone(),
                year: earlier_dt.year,
                month: earlier_dt.month,
                day: earlier_dt.day,
                sunrise_jd: earlier_sunrise,
                tithi_at_sunrise: earlier_tithi,
                lunar_month_name: month_name,
                is_adhik_month: is_adhik,
                reasoning: earlier_reason,
                priority_applied: Priority::Puurvaviddha.as_str(),
                kaala_applied: "",
                alternate: Some(AlternateObservance {
                    year: natural_dt.year,
                    month: natural_dt.month,
                    day: natural_dt.day,
                    sunrise_jd: natural_sunrise,
                    priority: Priority::Paraviddha.as_str(),
                    reasoning: paraviddha_reason,
                }),
            })
        }

        Priority::Vyapti => {
            // Find the day of the pair on which the target tithi is present
            // during the configured kaala. Prefer the earlier day if it alone
            // qualifies; otherwise fall back to the later (paraviddha) day.
            let (earlier_sunrise, earlier_dt) =
                observance::previous_day_sunrise(natural_sunrise, utc_offset, lat, lng, alt);
            let (later_sunrise, later_dt) = (natural_sunrise, natural_dt);
            let (next_sunrise_after_natural, _) =
                observance::next_day_sunrise(natural_sunrise, utc_offset, lat, lng, alt);

            // Compute sunsets from *actual local noon* (midnight + 0.5), not
            // a sunrise+0.5 approximation. This matters at high latitudes and
            // near equinoxes where day/night lengths are asymmetric.
            let earlier_noon = julian::midnight_jd(
                earlier_dt.year,
                earlier_dt.month,
                earlier_dt.day,
                utc_offset,
            ) + 0.5;
            let later_noon =
                julian::midnight_jd(later_dt.year, later_dt.month, later_dt.day, utc_offset) + 0.5;
            let earlier_sunset = sun::sunset_jd(earlier_noon, lat, lng, alt);
            let later_sunset = sun::sunset_jd(later_noon, lat, lng, alt);

            let earlier_window =
                observance::kaala_window(earlier_sunrise, earlier_sunset, later_sunrise, def.kaala);
            let later_window = observance::kaala_window(
                later_sunrise,
                later_sunset,
                next_sunrise_after_natural,
                def.kaala,
            );

            let earlier_has = tithi_present_in_window(def.tithi, earlier_window);
            let later_has = tithi_present_in_window(def.tithi, later_window);

            // Decision:
            // - If ONLY earlier qualifies → pick earlier (classic "tithi during
            //   kaala on prior day only" case, e.g., Akshaya Tritiya when
            //   Tritiya ends before later day's aparahna).
            // - If BOTH qualify → tie-break per def.vyapti_tie_purva
            //   (Vijayadashami takes the earlier day; most others the later).
            // - Otherwise → pick later (paraviddha default).
            let pick_earlier = earlier_has && (!later_has || def.vyapti_tie_purva);

            let kaala_name = def.kaala.as_str();
            if pick_earlier {
                let earlier_reason = if later_has {
                    format!(
                        "{} {} (Tithi {}) is present during the {} kaala on both \
                         {}-{:02}-{:02} and {}-{:02}-{:02}. \
                         Vyapti tie-break: earlier day observed (purva-vyapti \
                         convention for this festival). \
                         Paraviddha alternate (udayatithi rule): {}-{:02}-{:02}.",
                        month_name,
                        tithi_display_name(def.tithi),
                        def.tithi,
                        kaala_name,
                        earlier_dt.year,
                        earlier_dt.month,
                        earlier_dt.day,
                        later_dt.year,
                        later_dt.month,
                        later_dt.day,
                        later_dt.year,
                        later_dt.month,
                        later_dt.day,
                    )
                } else {
                    format!(
                        "{} {} (Tithi {}) is present during the {} kaala on {}-{:02}-{:02} \
                         but NOT during that kaala on {}-{:02}-{:02}. \
                         Vyapti rule: observe on the earlier day. \
                         Paraviddha alternate (udayatithi rule): {}-{:02}-{:02}.",
                        month_name,
                        tithi_display_name(def.tithi),
                        def.tithi,
                        kaala_name,
                        earlier_dt.year,
                        earlier_dt.month,
                        earlier_dt.day,
                        later_dt.year,
                        later_dt.month,
                        later_dt.day,
                        later_dt.year,
                        later_dt.month,
                        later_dt.day,
                    )
                };

                Some(FestivalOccurrence {
                    festival_id: def.id.clone(),
                    festival_name: def.name.clone(),
                    year: earlier_dt.year,
                    month: earlier_dt.month,
                    day: earlier_dt.day,
                    sunrise_jd: earlier_sunrise,
                    tithi_at_sunrise: tithi_at_jd(earlier_sunrise),
                    lunar_month_name: month_name,
                    is_adhik_month: is_adhik,
                    reasoning: earlier_reason,
                    priority_applied: Priority::Vyapti.as_str(),
                    kaala_applied: Kaala::from_label(kaala_name).as_str(),
                    alternate: Some(AlternateObservance {
                        year: later_dt.year,
                        month: later_dt.month,
                        day: later_dt.day,
                        sunrise_jd: later_sunrise,
                        priority: Priority::Paraviddha.as_str(),
                        reasoning: paraviddha_reason,
                    }),
                })
            } else {
                // Later day wins. If earlier day also had vyapti we still note
                // it as an alternate for user transparency.
                let reason = format!(
                    "{} {} (Tithi {}) is present during the {} kaala on {}-{:02}-{:02}. \
                     Vyapti rule: later day ({}) observed per paraviddha fallback.",
                    month_name,
                    tithi_display_name(def.tithi),
                    def.tithi,
                    kaala_name,
                    later_dt.year,
                    later_dt.month,
                    later_dt.day,
                    Priority::Paraviddha.as_str(),
                );

                let alt = if earlier_has {
                    Some(AlternateObservance {
                        year: earlier_dt.year,
                        month: earlier_dt.month,
                        day: earlier_dt.day,
                        sunrise_jd: earlier_sunrise,
                        priority: Priority::Puurvaviddha.as_str(),
                        reasoning: format!(
                            "Earlier day also has {} vyapti; some traditions prefer the earlier \
                             day (puurvaviddha) when both qualify.",
                            kaala_name
                        ),
                    })
                } else {
                    None
                };

                Some(FestivalOccurrence {
                    festival_id: def.id.clone(),
                    festival_name: def.name.clone(),
                    year: later_dt.year,
                    month: later_dt.month,
                    day: later_dt.day,
                    sunrise_jd: later_sunrise,
                    tithi_at_sunrise: def.tithi,
                    lunar_month_name: month_name,
                    is_adhik_month: is_adhik,
                    reasoning: reason,
                    priority_applied: Priority::Vyapti.as_str(),
                    kaala_applied: Kaala::from_label(kaala_name).as_str(),
                    alternate: alt,
                })
            }
        }
    }
}

/// Is the target tithi present anywhere in the given JD window?
///
/// Production-grade check: finds the exact JD boundaries of the target tithi
/// near the window (using arcsecond-precision bisection) and tests interval
/// overlap. This is correct even when the tithi begins or ends inside the
/// kaala window — a case that a midpoint-only probe would miss.
///
/// Algorithm:
/// 1. Probe the window midpoint for the current tithi.
/// 2. If the midpoint tithi IS the target → overlap exists; done.
/// 3. Otherwise, locate the target tithi's interval in the ±2-day neighborhood
///    by finding the forward crossing into the target tithi (its `start_jd`)
///    and the forward crossing out of it (its `end_jd`). Check if
///    `[start_jd, end_jd)` overlaps `[window_start, window_end]`.
///
/// A tithi lasts 19-26 hours in practice, so a 2-day search radius from the
/// window midpoint is always sufficient.
fn tithi_present_in_window(target_tithi: u32, window: (f64, f64)) -> bool {
    let (win_start, win_end) = window;
    let midpoint = (win_start + win_end) / 2.0;

    // Fast path: tithi at midpoint.
    if tithi_at_jd(midpoint) == target_tithi {
        return true;
    }

    // Target tithi starts when the tithi_angle crosses (target-1)*12°.
    // It ends when the angle crosses target*12°.
    let start_angle = ((target_tithi - 1) as f64) * TITHI_SPAN;
    let end_angle = (target_tithi as f64) * TITHI_SPAN;
    let angle_fn = |jd: f64| tithi_angle(jd);

    // Search backward from win_start for the START of the target tithi,
    // and forward from win_end for the END.
    let tithi_start = search::find_crossing_backward(win_start, start_angle, &angle_fn, 2.0)
        .or_else(|| search::find_crossing_forward(win_start, start_angle, &angle_fn, 2.0));
    let tithi_end = search::find_crossing_forward(win_start, end_angle, &angle_fn, 2.0);

    match (tithi_start, tithi_end) {
        (Some(ts), Some(te)) if te > ts => {
            // Interval overlap: [ts, te] ∩ [win_start, win_end] non-empty?
            ts < win_end && te > win_start
        }
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn format_paraviddha_reason(
    month_name: &'static str,
    tithi: u32,
    sunrise: f64,
    dt: &julian::DateTimeComponents,
    is_fallback: bool,
    sankranti: &SankrantiInfo,
    s_dt: &julian::DateTimeComponents,
    utc_offset: i32,
) -> String {
    if is_fallback {
        format!(
            "{} {} (Tithi {}) prevails at sunrise ({}) on {}-{:02}-{:02}. \
             Lunar month {} determined by {} ({}) on {}-{:02}-{:02} (fallback).",
            month_name,
            tithi_display_name(tithi),
            tithi,
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
        )
    } else {
        format!(
            "{} {} (Tithi {}) prevails at sunrise ({}) on {}-{:02}-{:02}. \
             Lunar month {} (Nija) boundaries used.",
            month_name,
            tithi_display_name(tithi),
            tithi,
            format_local_time(sunrise, utc_offset),
            dt.year,
            dt.month,
            dt.day,
            month_name,
        )
    }
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
        priority_applied: "sankranti",
        kaala_applied: "",
        alternate: None,
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
        priority_applied: "nakshatra_at_sunrise",
        kaala_applied: "",
        alternate: None,
    })
}

// ============================================================================
// Public API
// ============================================================================

/// Compute festival dates for the given year from festival definitions.
///
/// Festival definitions are passed from Python (loaded from YAML).
/// `tithi_at_sunrise`, `sankranti`, and `nakshatra_at_sunrise` rules are supported.
///
/// The `calendar_system` determines lunar month boundaries:
/// - **Purnimant** (North India default): month starts after Purnima
/// - **Amant** (South India): month starts after Amavasya
///
/// For Shukla Paksha festivals, both systems produce the same date.
/// For Krishna Paksha festivals, the month assignment may differ.
pub fn compute_festivals(
    defs: &[FestivalDef],
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
    calendar_system: CalendarSystem,
) -> Vec<FestivalOccurrence> {
    ephemeris::init(None);

    let sankrantis = sankranti::compute_sankrantis(year);

    // Amant month windows are identical for every tithi festival this year —
    // compute once here instead of once per def (~50x for a full year, which
    // used to dominate the entire computation).
    let lunar_months = lunar_month::compute_lunar_months(year, CalendarSystem::Amant);

    let mut results = Vec::with_capacity(defs.len());

    for def in defs {
        let occurrence = match def.rule.as_str() {
            "tithi_at_sunrise" => resolve_tithi_festival(
                def,
                &sankrantis,
                &lunar_months,
                lat,
                lng,
                alt,
                utc_offset,
                calendar_system,
            ),
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

/// Compute all Ekadashis for a year (2 per lunar month; 24 in a regular
/// year, 26 in an Adhika-maas year — the intercalary month contributes
/// Padmini (Shukla) and Parama (Krishna)).
///
/// Each Ekadashi has both Smartha and Vaishnava dates.
/// The Vaishnava date may differ by one day if Dashami persists at Arunodaya.
///
/// Iterates Amant lunar-month windows (month N = amavasya(N-1) → amavasya(N))
/// and resolves each Ekadashi inside its proper lunar-month bounds. Uses
/// ``find_tithi_at_sunrise_in_range`` which includes a tithi-kshaya fallback
/// (when Ekadashi never spans sunrise within the month, the Smarta date is
/// the preceding Dashami day per Dharmasindhu). Without this, months where
/// Ekadashi is kshaya — e.g. Kartik Shukla 2026 (Prabodhini) — were silently
/// dropped from the returned list.
pub fn compute_ekadashis(
    ekadashi_defs: &[EkadashiDef],
    year: i32,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Vec<EkadashiOccurrence> {
    ephemeris::init(None);

    // Use Amant boundaries — month N runs from Amavasya N-1 to Amavasya N,
    // which covers Shukla Paksha (first half) and Krishna Paksha (second half)
    // of month N. Pull year ± 1 to catch Ekadashis whose lunar month boundary
    // crosses Dec 31 / Jan 1.
    let mut lunar_months: Vec<lunar_month::LunarMonthInfo> = Vec::new();
    for y in [year - 1, year, year + 1] {
        lunar_months.extend(lunar_month::compute_lunar_months(y, CalendarSystem::Amant));
    }

    let mut results = Vec::new();

    for def in ekadashi_defs {
        let month_name = LUNAR_MONTH_NAMES[(def.month - 1) as usize];

        // Find all Amant lunar months with this number (could be two if adhika).
        // Resolve Ekadashi in each; results get filtered to `year` below.
        for lm in &lunar_months {
            if lm.number != def.month {
                continue;
            }

            // Adhika-month Ekadashis have universal names — Padmini (Shukla)
            // and Parama (Krishna) — regardless of which month is doubled.
            let (shukla_name, krishna_name) = if lm.is_adhik {
                ("Padmini", "Parama")
            } else {
                (def.shukla_name.as_str(), def.krishna_name.as_str())
            };

            // Shukla Ekadashi (tithi 11)
            if let Some(ek) = resolve_ekadashi(
                lm.start_jd,
                lm.end_jd,
                def.month,
                month_name,
                lm.is_adhik,
                true,
                shukla_name,
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
                lm.start_jd,
                lm.end_jd,
                def.month,
                month_name,
                lm.is_adhik,
                false,
                krishna_name,
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
    }

    // Dedupe: when pulling year ± 1, the same Ekadashi can be resolved twice
    // (e.g., via an AMANT month that appears in year-1's and year's slices).
    results.sort_by(|a, b| {
        a.smartha_sunrise_jd
            .partial_cmp(&b.smartha_sunrise_jd)
            .unwrap()
    });
    results.dedup_by(|a, b| {
        a.smartha_year == b.smartha_year
            && a.smartha_month == b.smartha_month
            && a.smartha_day == b.smartha_day
            && a.paksha == b.paksha
            && a.lunar_month == b.lunar_month
            && a.is_adhik == b.is_adhik
    });
    results
}

/// Resolve a single Ekadashi inside the given lunar-month window.
///
/// Uses the in-range search which handles tithi-kshaya by returning the
/// preceding Dashami day per Smarta convention. Returns ``None`` only if
/// neither the Ekadashi nor its preceding Dashami fall within the window —
/// which should not happen for a valid Amant lunar month.
#[allow(clippy::too_many_arguments)]
fn resolve_ekadashi(
    month_start_jd: f64,
    month_end_jd: f64,
    lunar_month_num: u32,
    lunar_month_name: &'static str,
    is_adhik: bool,
    is_shukla: bool,
    name: &str,
    lat: f64,
    lng: f64,
    alt: f64,
    utc_offset: i32,
) -> Option<EkadashiOccurrence> {
    let target_tithi: u32 = if is_shukla { 11 } else { 26 };
    let dashami_tithi: u32 = target_tithi - 1;

    let (first_sunrise, first_dt) = find_tithi_at_sunrise_in_range(
        target_tithi,
        month_start_jd,
        month_end_jd,
        lat,
        lng,
        alt,
        utc_offset,
    )?;

    // Dashami-vedha check: is Dashami still running at Arunodaya
    // (96 min before sunrise) on the first candidate day?
    let arunodaya_jd = first_sunrise - ARUNODAYA_MINUTES / 1440.0;
    let tithi_at_arunodaya = tithi_at_jd(arunodaya_jd);
    let dashami_at_arunodaya = tithi_at_arunodaya == dashami_tithi;

    let next_midnight =
        julian::midnight_jd(first_dt.year, first_dt.month, first_dt.day, utc_offset) + 1.0;
    let next_sunrise = sun::sunrise_jd(next_midnight, lat, lng, alt);
    let next_dt = local_date(next_sunrise, utc_offset);

    // Dharmasindhu nirnaya:
    // - Tithi-vriddhi (Ekadashi prevails at BOTH sunrises,
    //   "vriddhau uttara")                 → everyone observes day 2,
    //                                        vedha or not (Vijaya 2027 =
    //                                        Mar 4; Padmini 2026 = May 27).
    // - Vedha, single-sunrise Ekadashi     → Smartas keep day 1, Vaishnavas
    //                                        shift to day 2 (Dvadashi fast).
    // - No vedha, single sunrise           → everyone observes day 1.
    let ekadashi_at_next_sunrise = tithi_at_jd(next_sunrise) == target_tithi;

    let (smartha_sunrise, smartha_dt, vaishnava_sunrise, vaishnava_dt) = if ekadashi_at_next_sunrise
    {
        (next_sunrise, next_dt, next_sunrise, next_dt)
    } else if dashami_at_arunodaya {
        (first_sunrise, first_dt, next_sunrise, next_dt)
    } else {
        (first_sunrise, first_dt, first_sunrise, first_dt)
    };

    let paksha: &'static str = if is_shukla { "Shukla" } else { "Krishna" };

    let month_display = if is_adhik {
        format!("Adhika {}", lunar_month_name)
    } else {
        lunar_month_name.to_string()
    };

    let reasoning = if ekadashi_at_next_sunrise {
        format!(
            "{} Ekadashi (Tithi {}) prevails at sunrise on both \
             {}-{:02}-{:02} and the next day (tithi-vriddhi), so both \
             Smartha and Vaishnava observance fall on {}-{:02}-{:02} \
             (vriddhau uttara, Dharmasindhu). Lunar month: {}.",
            paksha,
            target_tithi,
            first_dt.year,
            first_dt.month,
            first_dt.day,
            smartha_dt.year,
            smartha_dt.month,
            smartha_dt.day,
            month_display,
        )
    } else if dashami_at_arunodaya {
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
            month_display,
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
            month_display,
        )
    };

    Some(EkadashiOccurrence {
        name: name.to_string(),
        lunar_month: lunar_month_num,
        lunar_month_name,
        is_adhik,
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
        // Month-recurring vrats ARE observed in adhika months (Drik Panchang
        // lists them with an "Adhika" prefix) — skipping them used to leave a
        // ~30-day vrat gap in every adhika year.
        let month_label = if month_info.is_adhik {
            format!("Adhika {}", month_info.name)
        } else {
            month_info.name.to_string()
        };

        for &(target_tithi, vrat_type, paksha) in &vrat_tithis {
            // In-range search includes the kshaya fallback: a Trayodashi or
            // Chaturthi that never spans sunrise would otherwise silently
            // drop that month's vrat.
            if let Some((sunrise, dt)) = find_tithi_at_sunrise_in_range(
                target_tithi,
                month_info.start_jd,
                month_info.end_jd,
                lat,
                lng,
                alt,
                utc_offset,
            ) {
                if dt.year == year {
                    results.push(VratOccurrence {
                        vrat_type: vrat_type.to_string(),
                        name: format!("{} ({})", vrat_type, month_label),
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

    // ─── Engine-level tests ─────────────────────────────────────────────
    // These use SYNTHETIC festival defs to test the resolution engine.
    // No duplication of festivals.yaml data.

    /// The engine resolves a tithi_at_sunrise rule and returns a valid date.
    #[test]
    fn test_engine_resolves_tithi_at_sunrise() {
        ephemeris::init(None);
        // Synthetic: Shukla Purnima (tithi 15) in Phalguna (month 12)
        let defs = vec![FestivalDef::new_tithi(
            "test_purnima",
            "Test Purnima",
            12,
            15,
        )];
        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].year, 2026);
        assert!(results[0].month >= 1 && results[0].month <= 12);
        assert!(!results[0].reasoning.is_empty());
    }

    /// The engine resolves a sankranti rule to a valid date.
    #[test]
    fn test_engine_resolves_sankranti() {
        ephemeris::init(None);
        let defs = vec![FestivalDef {
            id: "test_sankranti".into(),
            name: "Test Makar Sankranti".into(),
            rule: "sankranti".into(),
            lunar_month: 0,
            tithi: 0,
            sankranti_index: Some(0),
            nakshatra: None,
            priority: Priority::default(),
            kaala: Kaala::default(),
            vyapti_tie_purva: false,
            adhika_maasa: AdhikaMaasa::default(),
        }];
        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert_eq!(results.len(), 1);
        // Makar Sankranti is always in January
        assert_eq!(results[0].month, 1);
        assert!(results[0].day >= 13 && results[0].day <= 15);
    }

    /// Multiple festivals are returned sorted chronologically.
    #[test]
    fn test_engine_results_sorted_chronologically() {
        ephemeris::init(None);
        let defs = vec![
            FestivalDef::new_tithi("late", "Late Year", 8, 30),
            FestivalDef::new_tithi("early", "Early Year", 1, 9),
        ];
        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert_eq!(results.len(), 2);
        assert!(
            results[0].sunrise_jd < results[1].sunrise_jd,
            "Results should be chronological: {} before {}",
            results[0].festival_name,
            results[1].festival_name
        );
    }

    /// Kshaya tithi: when a tithi never prevails at sunrise, the engine
    /// falls back to the preceding tithi day (Dharmashastra convention).
    #[test]
    fn test_engine_kshaya_tithi_fallback() {
        ephemeris::init(None);
        // Chaitra Shukla Pratipada 2026 is kshaya (never prevails at sunrise).
        // Engine should fall back to the Amavasya day (March 19).
        let defs = vec![FestivalDef::new_tithi("test_kshaya", "Test Kshaya", 1, 1)];
        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].month, 3, "Should resolve to March");
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
                (-0.1..=1.5).contains(&diff),
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
            sankranti_index: Some(7),
            nakshatra: Some(22),
            priority: Priority::default(),
            kaala: Kaala::default(),
            vyapti_tie_purva: false,
            adhika_maasa: AdhikaMaasa::default(),
        }];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
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
            sankranti_index: Some(3),
            nakshatra: Some(5),
            priority: Priority::default(),
            kaala: Kaala::default(),
            vyapti_tie_purva: false,
            adhika_maasa: AdhikaMaasa::default(),
        }];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert_eq!(results.len(), 1, "Nakshatra festival should resolve");
        assert!(results[0].reasoning.contains("Mrigashira"));
        assert_eq!(results[0].year, 2026);
    }

    /// Validate key 2026 festival dates against Drik Panchang / web sources.
    /// This catches the Sankranti-search-window bug where early Shukla tithis
    /// (like Pratipada) resolved to the wrong month.
    #[test]
    fn test_chaitra_festivals_2026_exact_dates() {
        ephemeris::init(None);

        let defs = vec![
            FestivalDef::new_tithi("chaitra_navaratri", "Chaitra Navaratri", 1, 1),
            FestivalDef::new_tithi("ugadi", "Ugadi", 1, 1),
            FestivalDef::new_tithi("ram_navami", "Ram Navami", 1, 9),
        ];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);

        // Chaitra Navaratri & Ugadi: must be in March 2026, NOT April.
        // Web sources: March 19, 2026 (Chaitra Shukla Pratipada).
        let navaratri = results
            .iter()
            .find(|r| r.festival_id == "chaitra_navaratri")
            .unwrap();
        assert_eq!(
            navaratri.month, 3,
            "Chaitra Navaratri should be in March, got {}-{:02}-{:02}. Reasoning: {}",
            navaratri.year, navaratri.month, navaratri.day, navaratri.reasoning
        );
        // Allow March 18-20 (±1 day for tithi boundary)
        assert!(
            navaratri.day >= 18 && navaratri.day <= 20,
            "Chaitra Navaratri expected ~Mar 19, got Mar {}",
            navaratri.day
        );

        let ugadi = results.iter().find(|r| r.festival_id == "ugadi").unwrap();
        assert_eq!(
            ugadi.month, 3,
            "Ugadi should be in March, got {}-{:02}-{:02}",
            ugadi.year, ugadi.month, ugadi.day
        );

        // Ram Navami: web source says March 26-27.
        let ram_navami = results
            .iter()
            .find(|r| r.festival_id == "ram_navami")
            .unwrap();
        assert_eq!(
            ram_navami.month, 3,
            "Ram Navami should be in March, got {}-{:02}-{:02}",
            ram_navami.year, ram_navami.month, ram_navami.day
        );
        assert!(
            ram_navami.day >= 26 && ram_navami.day <= 28,
            "Ram Navami expected ~Mar 27, got Mar {}",
            ram_navami.day
        );
    }

    /// Cross-check other major 2026 festivals against web sources.
    #[test]
    fn test_major_festivals_2026_dates() {
        ephemeris::init(None);

        let defs = vec![
            FestivalDef::new_tithi("akshaya_tritiya", "Akshaya Tritiya", 2, 3),
            FestivalDef::new_tithi("dussehra", "Dussehra", 7, 10),
            // Diwali = Amavasya ending amant Ashwin (month 7). "Kartik
            // Amavasya" is the purnimant label for the same paksha.
            FestivalDef::new_tithi("diwali", "Diwali", 7, 30),
        ];

        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);

        // Akshaya Tritiya: web says Apr 19. Allow ±1 day.
        let at = results
            .iter()
            .find(|r| r.festival_id == "akshaya_tritiya")
            .unwrap();
        assert_eq!(at.month, 4, "Akshaya Tritiya should be in April");
        assert!(
            at.day >= 18 && at.day <= 20,
            "Akshaya Tritiya expected ~Apr 19, got Apr {}. Reasoning: {}",
            at.day,
            at.reasoning
        );

        // Dussehra: web says Oct 20. Allow ±1 day.
        let duss = results
            .iter()
            .find(|r| r.festival_id == "dussehra")
            .unwrap();
        assert_eq!(duss.month, 10, "Dussehra should be in October");
        assert!(
            duss.day >= 19 && duss.day <= 21,
            "Dussehra expected ~Oct 20, got Oct {}. Reasoning: {}",
            duss.day,
            duss.reasoning
        );

        // Diwali: web says Nov 8. Allow ±1 day.
        let diw = results.iter().find(|r| r.festival_id == "diwali").unwrap();
        assert_eq!(diw.month, 11, "Diwali should be in November");
        assert!(
            diw.day >= 7 && diw.day <= 9,
            "Diwali expected ~Nov 8, got Nov {}. Reasoning: {}",
            diw.day,
            diw.reasoning
        );
    }

    // ─── Observance-rule tests ─────────────────────────────────────────────

    /// Backward compatibility: default observance (paraviddha / sunrise / nija)
    /// reproduces the current behavior for existing festivals.
    #[test]
    fn test_default_observance_matches_paraviddha() {
        ephemeris::init(None);
        let def_default = FestivalDef::new_tithi("janmashtami", "Janmashtami", 5, 23);
        let def_explicit = FestivalDef::new_tithi("janmashtami_x", "Janmashtami", 5, 23)
            .with_observance(Priority::Paraviddha, Kaala::Sunrise);

        let r1 = compute_festivals(
            &[def_default],
            2026,
            LAT,
            LNG,
            ALT,
            IST,
            CalendarSystem::Purnimant,
        );
        let r2 = compute_festivals(
            &[def_explicit],
            2026,
            LAT,
            LNG,
            ALT,
            IST,
            CalendarSystem::Purnimant,
        );

        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 1);
        assert_eq!(
            (r1[0].year, r1[0].month, r1[0].day),
            (r2[0].year, r2[0].month, r2[0].day)
        );
        assert_eq!(r1[0].priority_applied, "paraviddha");
        assert_eq!(r1[0].kaala_applied, "");
    }

    /// Akshaya Tritiya 2026: Vyapti(Aparahna) rule should produce **April 19**
    /// (Sunday), matching Drik Panchang. The alternate should surface April 20
    /// (Monday — paraviddha / udayatithi).
    ///
    /// Rationale: Tritiya tithi ends during early afternoon on April 20, so
    /// it is NOT present during Aparahna (late afternoon) on April 20. Tritiya
    /// IS present during the Aparahna of April 19 — so the vyapti rule picks
    /// April 19.
    #[test]
    fn test_akshaya_tritiya_2026_vyapti_aparahna() {
        ephemeris::init(None);
        let def = FestivalDef::new_tithi("akshaya_tritiya", "Akshaya Tritiya", 2, 3)
            .with_observance(Priority::Vyapti, Kaala::Aparahna);

        let results =
            compute_festivals(&[def], 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert_eq!(results.len(), 1);
        let at = &results[0];

        assert_eq!(
            (at.year, at.month, at.day),
            (2026, 4, 19),
            "AT 2026 should be April 19 (vyapti/aparahna, matches Drik Panchang). Reasoning: {}",
            at.reasoning
        );
        assert_eq!(at.priority_applied, "vyapti");
        assert_eq!(at.kaala_applied, "aparahna");

        // The paraviddha alternate should be the following day (April 20).
        let alt = at
            .alternate
            .as_ref()
            .expect("vyapti result should expose paraviddha alternate");
        assert_eq!((alt.year, alt.month, alt.day), (2026, 4, 20));
        assert_eq!(alt.priority, "paraviddha");
    }

    /// With Priority::Paraviddha (udayatithi / default), AT 2026 resolves to
    /// **April 20** (Monday), matching the strict Smriti Kaustubha reading.
    /// This documents the convention split so downstream code can surface both.
    #[test]
    fn test_akshaya_tritiya_2026_paraviddha() {
        ephemeris::init(None);
        let def = FestivalDef::new_tithi("akshaya_tritiya", "Akshaya Tritiya", 2, 3);

        let results =
            compute_festivals(&[def], 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert_eq!(results.len(), 1);
        let at = &results[0];

        assert_eq!(
            (at.year, at.month, at.day),
            (2026, 4, 20),
            "AT 2026 paraviddha rule should be April 20. Reasoning: {}",
            at.reasoning
        );
        assert_eq!(at.priority_applied, "paraviddha");
        // No alternate for the default paraviddha resolution.
        assert!(at.alternate.is_none());
    }

    /// Puurvaviddha rule shifts the observance to the day BEFORE the natural
    /// paraviddha day. Using a synthetic festival so we don't conflict with
    /// any real festival's shastric rule.
    #[test]
    fn test_puurvaviddha_shifts_one_day_earlier() {
        ephemeris::init(None);
        let para = FestivalDef::new_tithi("synth_para", "Synth Para", 5, 8);
        let purva = FestivalDef::new_tithi("synth_purva", "Synth Purva", 5, 8)
            .with_observance(Priority::Puurvaviddha, Kaala::Sunrise);

        let r1 = compute_festivals(&[para], 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        let r2 = compute_festivals(
            &[purva],
            2026,
            LAT,
            LNG,
            ALT,
            IST,
            CalendarSystem::Purnimant,
        );

        assert_eq!(r1.len(), 1);
        assert_eq!(r2.len(), 1);

        // purva should be exactly one day before para.
        let para_jd = r1[0].sunrise_jd;
        let purva_jd = r2[0].sunrise_jd;
        let delta_days = para_jd - purva_jd;
        assert!(
            (delta_days - 1.0).abs() < 0.05, // within ~1 hour of exactly 1 day
            "Puurvaviddha should shift 1 day earlier; got {} day delta (para={}-{:02}-{:02}, purva={}-{:02}-{:02})",
            delta_days,
            r1[0].year, r1[0].month, r1[0].day,
            r2[0].year, r2[0].month, r2[0].day,
        );
        assert_eq!(r2[0].priority_applied, "puurvaviddha");
        // Puurvaviddha should always expose the paraviddha alternate.
        assert!(r2[0].alternate.is_some());
    }

    /// `adhika_maasa = Adhika` with no adhika month this year returns no result.
    /// 2026 has no adhika month in Vaishakha, so this should be empty.
    #[test]
    fn test_adhika_maasa_policy_adhika_only() {
        ephemeris::init(None);
        let mut def = FestivalDef::new_tithi("synth_adhika", "Synth Adhika", 2, 3);
        def.adhika_maasa = AdhikaMaasa::Adhika;

        let results =
            compute_festivals(&[def], 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert!(
            results.is_empty(),
            "adhika-only policy should skip non-adhika years; got {} results",
            results.len()
        );
    }

    /// All the Chaitra festivals that were previously broken (April 18 instead
    /// of March 19) must still resolve to March 19 after observance refactor.
    #[test]
    fn test_chaitra_regression_still_march_19() {
        ephemeris::init(None);
        let defs = vec![
            FestivalDef::new_tithi("chaitra_navaratri", "Chaitra Navaratri", 1, 1),
            FestivalDef::new_tithi("ugadi", "Ugadi", 1, 1),
            FestivalDef::new_tithi("gudi_padwa", "Gudi Padwa", 1, 1),
        ];
        let results = compute_festivals(&defs, 2026, LAT, LNG, ALT, IST, CalendarSystem::Purnimant);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.month, 3, "{} should be in March 2026", r.festival_name);
            assert!(
                r.day >= 18 && r.day <= 20,
                "{} should be ~March 19; got March {}. Reasoning: {}",
                r.festival_name,
                r.day,
                r.reasoning
            );
        }
    }
}
