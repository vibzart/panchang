//! Observance rules for tithi-based festivals — shastra-sammat resolution.
//!
//! Hindu festivals that fall on a tithi often have a secondary rule determining
//! which of two candidate days is observed when the tithi spans two sunrises.
//! This module models those rules explicitly, following the vocabulary used in
//! classical Dharmashastra (Smriti Kaustubha, Nirnaya Sindhu, Dharma Sindhu)
//! and the `adyatithi` corpus.
//!
//! # The two orthogonal dimensions
//!
//! 1. **Priority** — which day to pick when tithi spans two sunrises:
//!    - `Paraviddha`: the later day (target tithi at sunrise). This is the
//!      udayatithi default used by most festivals.
//!    - `Puurvaviddha`: the earlier day (target tithi *begins* during the day,
//!      even if not at sunrise). Used for a small set of festivals like
//!      Mahatara Jayanti.
//!    - `Vyapti`: the day on which the tithi is present during a specified
//!      `Kaala` window (see below). If both days qualify, the later day wins
//!      (paraviddha fallback). If only the earlier qualifies, it wins.
//!
//! 2. **Kaala** — the time-window during which the tithi must be present
//!    (relevant only for the `Vyapti` priority). Daytime kaalas are computed
//!    from the classical panchama-vibhaga (5-fold division of daylight).
//!
//! # Akshaya Tritiya — the canonical example
//!
//! Drik Panchang and most Indian sources observe AT on the day Tritiya is
//! present during `Aparahna` kaala (afternoon, 4th 1/5 of daylight) — this is
//! `Vyapti(Aparahna)`. adyatithi / Smriti Kaustubha uses `Paraviddha` with
//! full-day sparsha. These two conventions can give different dates for the
//! same year. Defaulting to `Vyapti(Aparahna)` matches Drik Panchang and
//! popular observance.

use crate::julian::{self, DateTimeComponents};
use crate::sun;

// ============================================================================
// Enums — observance vocabulary
// ============================================================================

/// Which of two candidate days to pick when a tithi spans two sunrises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Priority {
    /// Later day — target tithi at sunrise. Default udayatithi rule.
    #[default]
    Paraviddha,
    /// Earlier day — target tithi begins during the day, even if not at sunrise.
    Puurvaviddha,
    /// Pick the day on which the tithi is present during the specified kaala.
    /// If both days qualify, later day wins. If only earlier qualifies, it wins.
    Vyapti,
}

impl Priority {
    /// Parse a label from YAML/user input into the enum, defaulting on unknown.
    /// This is intentionally infallible — unknown values map to the default
    /// variant rather than failing loudly, so adding new aliases never breaks
    /// existing datasets.
    pub fn from_label(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "puurvaviddha" | "purvaviddha" | "purva_viddha" | "earlier" => Self::Puurvaviddha,
            "vyapti" | "vyaapti" | "vyApti" => Self::Vyapti,
            _ => Self::Paraviddha,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Paraviddha => "paraviddha",
            Self::Puurvaviddha => "puurvaviddha",
            Self::Vyapti => "vyapti",
        }
    }
}

/// Time-window during the day (or around it). Used by the `Vyapti` priority
/// to specify when the tithi must be present.
///
/// The panchama-vibhaga (5-fold day division) is computed from local sunrise
/// and sunset, so each kaala is ~2 hours wide in India and wider near the
/// equinoxes at latitudes far from the equator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kaala {
    /// Moment of sunrise (udayatithi check).
    #[default]
    Sunrise,
    /// First 1/5 of daylight (~dawn to ~mid-morning).
    Praatah,
    /// Second 1/5 of daylight (~mid-morning to ~noon).
    Sangava,
    /// Third 1/5 of daylight (~noon, the midday window).
    Madhyahna,
    /// Fourth 1/5 of daylight (~early afternoon).
    Aparahna,
    /// Fifth 1/5 of daylight (~late afternoon to sunset).
    Saayaahna,
    /// First half of daylight (sunrise to solar noon).
    Poorvahna,
    /// Pradosha kaala — 3 muhurtas (~144 min) after sunset.
    /// Used for Shivaratri, Pradosha Vrata, etc.
    Pradosha,
    /// Nishita kaala — midnight window (~±24 min around local midnight).
    /// Used for Krishna Janmashtami.
    Nishita,
    /// Full daytime (sunrise to sunset). Adyatithi's "साङ्गवः with sparsha".
    /// Distinct from the 5-fold Sangava above — use for conventions that
    /// require the tithi to touch any part of the day.
    FullDay,
}

impl Kaala {
    /// Parse a label from YAML/user input into the enum, defaulting on unknown.
    /// This is intentionally infallible — unknown values map to the default
    /// variant rather than failing loudly, so adding new aliases never breaks
    /// existing datasets.
    pub fn from_label(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "praatah" | "pratah" | "प्रातः" => Self::Praatah,
            "sangava" | "saangava" | "साङ्गवः" => Self::Sangava,
            "madhyahna" | "madhyaahna" | "मध्याह्नः" => Self::Madhyahna,
            "aparahna" | "aparaahna" | "अपराह्णः" => Self::Aparahna,
            "saayaahna" | "sayahna" | "सायाह्नः" => Self::Saayaahna,
            "poorvahna" | "purvahna" | "पूर्वाह्णः" => Self::Poorvahna,
            "pradosha" | "प्रदोषः" => Self::Pradosha,
            "nishita" | "nishitha" | "निशीथः" => Self::Nishita,
            "fullday" | "full_day" | "sangava_full" => Self::FullDay,
            _ => Self::Sunrise,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sunrise => "sunrise",
            Self::Praatah => "praatah",
            Self::Sangava => "sangava",
            Self::Madhyahna => "madhyahna",
            Self::Aparahna => "aparahna",
            Self::Saayaahna => "saayaahna",
            Self::Poorvahna => "poorvahna",
            Self::Pradosha => "pradosha",
            Self::Nishita => "nishita",
            Self::FullDay => "full_day",
        }
    }
}

/// How to handle extra (adhika) lunar months.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdhikaMaasa {
    /// Observe only in the regular (nija) month. Adhika month is skipped.
    /// Default for most festivals.
    #[default]
    Nija,
    /// Observe only in the adhika month. Skip the regular month entirely.
    Adhika,
    /// Observe in BOTH months when an adhika month exists (e.g., Yugadi).
    AdhikaAndNija,
    /// Prefer the adhika month; fall back to nija if no adhika this year.
    AdhikaIfExists,
}

impl AdhikaMaasa {
    /// Parse a label from YAML/user input into the enum, defaulting on unknown.
    /// This is intentionally infallible — unknown values map to the default
    /// variant rather than failing loudly, so adding new aliases never breaks
    /// existing datasets.
    pub fn from_label(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "adhika" | "adhika_only" => Self::Adhika,
            "adhika_and_nija" | "both" => Self::AdhikaAndNija,
            "adhika_if_exists" => Self::AdhikaIfExists,
            _ => Self::Nija,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nija => "nija",
            Self::Adhika => "adhika",
            Self::AdhikaAndNija => "adhika_and_nija",
            Self::AdhikaIfExists => "adhika_if_exists",
        }
    }
}

// ============================================================================
// Kaala window computation
// ============================================================================

/// Compute the JD window `[start, end]` for a given kaala on the date that
/// contains the given `sunrise_jd`.
///
/// Daytime kaalas are derived from the 5-fold division (panchama-vibhaga)
/// of the daylight span `[sunrise, sunset]`. Night-time kaalas require
/// knowing the *next* sunrise; for simplicity we approximate using local
/// midnight (sunset + 0.5 * (next_sunrise - sunset)).
///
/// Returns `(window_start_jd, window_end_jd)`.
pub fn kaala_window(
    sunrise_jd: f64,
    sunset_jd: f64,
    next_sunrise_jd: f64,
    kaala: Kaala,
) -> (f64, f64) {
    let day = sunset_jd - sunrise_jd;
    let night = next_sunrise_jd - sunset_jd;

    // Panchama-vibhaga: 5 equal parts of daylight.
    let one_fifth = day / 5.0;
    let p1_end = sunrise_jd + one_fifth;
    let p2_end = sunrise_jd + 2.0 * one_fifth;
    let p3_end = sunrise_jd + 3.0 * one_fifth;
    let p4_end = sunrise_jd + 4.0 * one_fifth;

    match kaala {
        // Sunrise is a point; use a ±1-min window for robustness.
        Kaala::Sunrise => (sunrise_jd - 1.0 / 1440.0, sunrise_jd + 1.0 / 1440.0),
        Kaala::Praatah => (sunrise_jd, p1_end),
        Kaala::Sangava => (p1_end, p2_end),
        Kaala::Madhyahna => (p2_end, p3_end),
        Kaala::Aparahna => (p3_end, p4_end),
        Kaala::Saayaahna => (p4_end, sunset_jd),
        Kaala::Poorvahna => (sunrise_jd, sunrise_jd + day / 2.0),
        Kaala::FullDay => (sunrise_jd, sunset_jd),
        // Pradosha: 3 muhurtas = 144 minutes after sunset.
        // (Classical definition is more nuanced but this is the widely-used approximation.)
        Kaala::Pradosha => (sunset_jd, sunset_jd + 144.0 / 1440.0),
        // Nishita: local midnight ± 24 minutes (~4 muhurtas span centered at midnight).
        Kaala::Nishita => {
            let local_midnight = sunset_jd + night / 2.0;
            (
                local_midnight - 24.0 / 1440.0,
                local_midnight + 24.0 / 1440.0,
            )
        }
    }
}

// ============================================================================
// Date-shift helpers
// ============================================================================

/// Sunrise JD of the day immediately preceding the given sunrise.
pub fn previous_day_sunrise(
    sunrise_jd: f64,
    utc_offset: i32,
    lat: f64,
    lng: f64,
    alt: f64,
) -> (f64, DateTimeComponents) {
    // Walk back by 24h, then let swe_rise_trans find the previous sunrise.
    let local_jd = sunrise_jd + (utc_offset as f64) / 86400.0;
    let dt = julian::jd_to_datetime(local_jd - 1.0);
    let midnight = julian::midnight_jd(dt.year, dt.month, dt.day, utc_offset);
    let prev_sunrise = sun::sunrise_jd(midnight, lat, lng, alt);
    let local = prev_sunrise + (utc_offset as f64) / 86400.0;
    (prev_sunrise, julian::jd_to_datetime(local))
}

/// Sunrise JD of the day immediately following the given sunrise.
pub fn next_day_sunrise(
    sunrise_jd: f64,
    utc_offset: i32,
    lat: f64,
    lng: f64,
    alt: f64,
) -> (f64, DateTimeComponents) {
    let local_jd = sunrise_jd + (utc_offset as f64) / 86400.0;
    let dt = julian::jd_to_datetime(local_jd + 1.0);
    let midnight = julian::midnight_jd(dt.year, dt.month, dt.day, utc_offset);
    let next_sunrise = sun::sunrise_jd(midnight, lat, lng, alt);
    let local = next_sunrise + (utc_offset as f64) / 86400.0;
    (next_sunrise, julian::jd_to_datetime(local))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_from_str_aliases() {
        assert_eq!(Priority::from_label("paraviddha"), Priority::Paraviddha);
        assert_eq!(Priority::from_label("puurvaviddha"), Priority::Puurvaviddha);
        assert_eq!(Priority::from_label("purvaviddha"), Priority::Puurvaviddha);
        assert_eq!(Priority::from_label("vyapti"), Priority::Vyapti);
        assert_eq!(Priority::from_label("vyaapti"), Priority::Vyapti);
        assert_eq!(Priority::from_label("unknown"), Priority::Paraviddha);
    }

    #[test]
    fn kaala_from_str_aliases() {
        assert_eq!(Kaala::from_label("aparahna"), Kaala::Aparahna);
        assert_eq!(Kaala::from_label("aparaahna"), Kaala::Aparahna);
        assert_eq!(Kaala::from_label("अपराह्णः"), Kaala::Aparahna);
        assert_eq!(Kaala::from_label("pradosha"), Kaala::Pradosha);
        assert_eq!(Kaala::from_label("unknown"), Kaala::Sunrise);
    }

    #[test]
    fn kaala_window_panchama_vibhaga_covers_daytime() {
        let sunrise = 2460000.0;
        let sunset = sunrise + 0.5; // 12-hour day for simplicity
        let next_sunrise = sunrise + 1.0;

        let (p_start, p_end) = kaala_window(sunrise, sunset, next_sunrise, Kaala::Praatah);
        let (s_start, s_end) = kaala_window(sunrise, sunset, next_sunrise, Kaala::Sangava);
        let (m_start, m_end) = kaala_window(sunrise, sunset, next_sunrise, Kaala::Madhyahna);
        let (a_start, a_end) = kaala_window(sunrise, sunset, next_sunrise, Kaala::Aparahna);
        let (y_start, y_end) = kaala_window(sunrise, sunset, next_sunrise, Kaala::Saayaahna);

        // Contiguous, non-overlapping 5-fold division.
        assert!((p_start - sunrise).abs() < 1e-9);
        assert!((p_end - s_start).abs() < 1e-9);
        assert!((s_end - m_start).abs() < 1e-9);
        assert!((m_end - a_start).abs() < 1e-9);
        assert!((a_end - y_start).abs() < 1e-9);
        assert!((y_end - sunset).abs() < 1e-9);

        // Each window is day/5 = 0.1 JD wide.
        for (start, end) in [
            (p_start, p_end),
            (s_start, s_end),
            (m_start, m_end),
            (a_start, a_end),
            (y_start, y_end),
        ] {
            assert!(((end - start) - 0.1).abs() < 1e-9);
        }
    }

    #[test]
    fn kaala_window_pradosha_is_144_minutes() {
        let sunrise = 2460000.0;
        let sunset = sunrise + 0.5;
        let next_sunrise = sunrise + 1.0;

        let (start, end) = kaala_window(sunrise, sunset, next_sunrise, Kaala::Pradosha);
        assert!((start - sunset).abs() < 1e-9);
        assert!(((end - start) - 144.0 / 1440.0).abs() < 1e-9);
    }
}
