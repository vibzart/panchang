//! Samvat (era) year computation.
//!
//! Hindu calendars use different eras for year numbering:
//! - **Vikram Samvat**: CE + 57 (North Bhārat, Gujarat)
//! - **Shaka Samvat**: CE - 78 (South Bhārat, Bhāratīya National Calendar)
//! - **Bangabda**: CE - 593 (Bengali calendar)
//! - **Kollavarsham**: CE - 825 (Malayalam calendar)
//! - **Thiruvalluvar Aandu**: CE + 31 (Tamil calendar)
//! - **60-year Jovian cycle**: Named years repeating every 60 years
//!
//! The year transition does NOT happen on January 1 — it happens at the
//! regional New Year (e.g., Chaitra Shukla Pratipada for Vikram Samvat).
//! For a given Gregorian date, the Samvat year depends on whether the
//! regional New Year has already occurred.

/// Compute the era year from a Gregorian year using a simple offset.
///
/// `offset` is added to `gregorian_year`. For example:
/// - Vikram Samvat: offset = +57
/// - Shaka Samvat: offset = -78
///
/// The `new_year_passed` flag indicates whether the regional new year
/// has already occurred in the current Gregorian year. If not, the
/// era year may be one less (for eras where the new year falls after Jan 1).
pub fn era_year_from_offset(gregorian_year: i32, offset: i32, new_year_passed: bool) -> i32 {
    let base = gregorian_year + offset;
    if offset > 0 && !new_year_passed {
        // For eras ahead of CE (like Vikram Samvat), before the new year
        // we're still in the previous era year
        base
    } else if offset < 0 && !new_year_passed {
        // For eras behind CE (like Shaka), before the new year
        // we're still in the previous era year
        base - 1
    } else {
        base
    }
}

/// Compute the 60-year Jovian cycle year index (0-59) for a Gregorian year.
///
/// The cycle epoch: 1987 CE = year 1 (Prabhava).
/// `cycle_index = (gregorian_year - 1987) % 60`, adjusted to [0, 59].
pub fn jovian_cycle_index(gregorian_year: i32, epoch_year: i32) -> u32 {
    let diff = gregorian_year - epoch_year;
    ((diff % 60 + 60) % 60) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vikram_samvat_2026() {
        // 2026 CE = Vikram Samvat 2082-2083
        // Before Chaitra S1 (~March): VS 2082
        // After Chaitra S1: VS 2083
        assert_eq!(era_year_from_offset(2026, 57, false), 2083);
        assert_eq!(era_year_from_offset(2026, 57, true), 2083);
    }

    #[test]
    fn test_shaka_samvat_2026() {
        // 2026 CE = Shaka 1947-1948
        // Before Chaitra S1: Shaka 1947
        // After Chaitra S1: Shaka 1948
        assert_eq!(era_year_from_offset(2026, -78, false), 1947);
        assert_eq!(era_year_from_offset(2026, -78, true), 1948);
    }

    #[test]
    fn test_bangabda_2026() {
        // 2026 CE = Bangabda 1432-1433
        assert_eq!(era_year_from_offset(2026, -593, false), 1432);
        assert_eq!(era_year_from_offset(2026, -593, true), 1433);
    }

    #[test]
    fn test_kollavarsham_2026() {
        // 2026 CE = Kollavarsham 1200-1201
        assert_eq!(era_year_from_offset(2026, -825, false), 1200);
        assert_eq!(era_year_from_offset(2026, -825, true), 1201);
    }

    #[test]
    fn test_thiruvalluvar_2026() {
        // 2026 CE = Thiruvalluvar 2057
        assert_eq!(era_year_from_offset(2026, 31, false), 2057);
        assert_eq!(era_year_from_offset(2026, 31, true), 2057);
    }

    #[test]
    fn test_jovian_cycle_2026() {
        // 2026 - 1987 = 39 → index 39 (0-based)
        assert_eq!(jovian_cycle_index(2026, 1987), 39);
    }

    #[test]
    fn test_jovian_cycle_epoch() {
        // At epoch year itself → index 0
        assert_eq!(jovian_cycle_index(1987, 1987), 0);
    }

    #[test]
    fn test_jovian_cycle_wraps() {
        // 1987 + 60 = 2047 → should wrap to 0
        assert_eq!(jovian_cycle_index(2047, 1987), 0);
        // 1987 - 1 = 1986 → should be 59
        assert_eq!(jovian_cycle_index(1986, 1987), 59);
    }
}
