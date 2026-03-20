//! Name tables and lookup constants for Panchang elements.

/// 27 Nakshatra names (0-indexed).
pub const NAKSHATRA_NAMES: [&str; 27] = [
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashira",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Mula",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishta",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];

/// Nakshatra lords (Vimshottari Dasha rulers), repeating 3 × 9 cycle.
pub const NAKSHATRA_LORDS: [&str; 27] = [
    "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury", "Ketu",
    "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury", "Ketu", "Venus", "Sun",
    "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury",
];

/// 30 Tithi names (0-indexed). First 15 = Shukla, last 15 = Krishna.
pub const TITHI_NAMES: [&str; 30] = [
    "Pratipada",
    "Dvitiya",
    "Tritiya",
    "Chaturthi",
    "Panchami",
    "Shashthi",
    "Saptami",
    "Ashtami",
    "Navami",
    "Dashami",
    "Ekadashi",
    "Dvadashi",
    "Trayodashi",
    "Chaturdashi",
    "Purnima",
    // Krishna paksha
    "Pratipada",
    "Dvitiya",
    "Tritiya",
    "Chaturthi",
    "Panchami",
    "Shashthi",
    "Saptami",
    "Ashtami",
    "Navami",
    "Dashami",
    "Ekadashi",
    "Dvadashi",
    "Trayodashi",
    "Chaturdashi",
    "Amavasya",
];

/// 27 Yoga names (0-indexed).
pub const YOGA_NAMES: [&str; 27] = [
    "Vishkambha",
    "Priti",
    "Ayushman",
    "Saubhagya",
    "Shobhana",
    "Atiganda",
    "Sukarma",
    "Dhriti",
    "Shula",
    "Ganda",
    "Vriddhi",
    "Dhruva",
    "Vyaghata",
    "Harshana",
    "Vajra",
    "Siddhi",
    "Vyatipata",
    "Variyan",
    "Parigha",
    "Shiva",
    "Siddha",
    "Sadhya",
    "Shubha",
    "Shukla",
    "Brahma",
    "Indra",
    "Vaidhriti",
];

/// 11 Karana names. First 7 are rotating, last 4 are fixed.
pub const KARANA_NAMES: [&str; 11] = [
    "Bava",
    "Balava",
    "Kaulava",
    "Taitila",
    "Garaja",
    "Vanija",
    "Vishti",      // 7 rotating (Vishti = Bhadra, inauspicious)
    "Shakuni",     // Fixed
    "Chatushpada", // Fixed
    "Nagava",      // Fixed
    "Kimstughna",  // Fixed
];

/// Sanskrit Vara (weekday) names. Sunday=0.
pub const VARA_NAMES: [&str; 7] = [
    "Ravivara",    // Sunday
    "Somavara",    // Monday
    "Mangalavara", // Tuesday
    "Budhavara",   // Wednesday
    "Guruvara",    // Thursday
    "Shukravara",  // Friday
    "Shanivara",   // Saturday
];

/// English weekday names. Sunday=0.
pub const VARA_ENGLISH: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Choghadiya period names (7 rotating).
pub const CHOGHADIYA_NAMES: [&str; 7] = ["Udveg", "Char", "Labh", "Amrit", "Kaal", "Shubh", "Rog"];

/// Day Choghadiya starting index by weekday (Sunday=0).
pub const DAY_CHOGHADIYA_START: [usize; 7] = [0, 3, 6, 2, 5, 1, 4];

/// Night Choghadiya starting index by weekday (Sunday=0).
pub const NIGHT_CHOGHADIYA_START: [usize; 7] = [5, 1, 4, 6, 0, 3, 2];

/// Rahu Kalam slot index (0-7, which 1/8th of daytime) by weekday (Sunday=0).
pub const RAHU_KALAM_SLOT: [u32; 7] = [7, 1, 6, 4, 5, 3, 2];

/// Yama Gandam slot index by weekday (Sunday=0).
pub const YAMA_GANDAM_SLOT: [u32; 7] = [4, 3, 5, 6, 7, 1, 0];

/// Gulika Kalam slot index by weekday (Sunday=0).
pub const GULIKA_KALAM_SLOT: [u32; 7] = [6, 5, 4, 3, 2, 1, 0];

// ============================================================================
// Rashi (zodiac sign) names — sidereal order starting from Mesha (0°)
// ============================================================================

/// 12 Rashi names. Index 0 = Mesha (Aries, 0°–30°).
pub const RASHI_NAMES: [&str; 12] = [
    "Mesha",     // Aries       (0-30°)
    "Vrishabha", // Taurus      (30-60°)
    "Mithuna",   // Gemini      (60-90°)
    "Karka",     // Cancer      (90-120°)
    "Simha",     // Leo         (120-150°)
    "Kanya",     // Virgo       (150-180°)
    "Tula",      // Libra       (180-210°)
    "Vrischika", // Scorpio     (210-240°)
    "Dhanu",     // Sagittarius (240-270°)
    "Makara",    // Capricorn   (270-300°)
    "Kumbha",    // Aquarius    (300-330°)
    "Meena",     // Pisces      (330-360°)
];

// ============================================================================
// Sankranti constants — ordered by calendar year (Makar ~Jan 14 first)
// ============================================================================

/// Sankranti names in calendar-year order.
pub const SANKRANTI_NAMES: [&str; 12] = [
    "Makar Sankranti",     // Sun enters Makara  (270°)
    "Kumbha Sankranti",    // Sun enters Kumbha  (300°)
    "Meena Sankranti",     // Sun enters Meena   (330°)
    "Mesha Sankranti",     // Sun enters Mesha   (0°)
    "Vrishabha Sankranti", // Sun enters Vrishabha (30°)
    "Mithuna Sankranti",   // Sun enters Mithuna (60°)
    "Karka Sankranti",     // Sun enters Karka   (90°)
    "Simha Sankranti",     // Sun enters Simha   (120°)
    "Kanya Sankranti",     // Sun enters Kanya   (150°)
    "Tula Sankranti",      // Sun enters Tula    (180°)
    "Vrischika Sankranti", // Sun enters Vrischika (210°)
    "Dhanu Sankranti",     // Sun enters Dhanu   (240°)
];

/// Target sidereal longitudes for each Sankranti, in calendar-year order.
pub const SANKRANTI_TARGET_LONGITUDES: [f64; 12] = [
    270.0, // Makar (Capricorn)
    300.0, // Kumbha (Aquarius)
    330.0, // Meena (Pisces)
    0.0,   // Mesha (Aries)
    30.0,  // Vrishabha (Taurus)
    60.0,  // Mithuna (Gemini)
    90.0,  // Karka (Cancer)
    120.0, // Simha (Leo)
    150.0, // Kanya (Virgo)
    180.0, // Tula (Libra)
    210.0, // Vrischika (Scorpio)
    240.0, // Dhanu (Sagittarius)
];

/// Rashi index (into RASHI_NAMES) for each Sankranti, in calendar-year order.
pub const SANKRANTI_RASHI_INDEX: [usize; 12] = [9, 10, 11, 0, 1, 2, 3, 4, 5, 6, 7, 8];

/// Approximate (month, day) to start searching for each Sankranti.
/// Search starts ~2 weeks before expected date.
pub const SANKRANTI_APPROX_DATES: [(u32, u32); 12] = [
    (1, 1),  // Makar     ~Jan 14
    (1, 30), // Kumbha    ~Feb 13
    (2, 28), // Meena     ~Mar 14
    (3, 31), // Mesha     ~Apr 14
    (4, 30), // Vrishabha ~May 14
    (6, 1),  // Mithuna   ~Jun 15
    (7, 2),  // Karka     ~Jul 16
    (8, 3),  // Simha     ~Aug 17
    (9, 3),  // Kanya     ~Sep 17
    (10, 3), // Tula      ~Oct 17
    (11, 2), // Vrischika ~Nov 16
    (12, 2), // Dhanu     ~Dec 16
];

// ============================================================================
// Lunar month names and Sankranti-to-month mapping
// ============================================================================

/// 12 lunar month names. Index 0 = Chaitra (1st month of Hindu year).
pub const LUNAR_MONTH_NAMES: [&str; 12] = [
    "Chaitra",      // 1  (Mar-Apr)
    "Vaishakha",    // 2  (Apr-May)
    "Jyeshtha",     // 3  (May-Jun)
    "Ashadha",      // 4  (Jun-Jul)
    "Shravana",     // 5  (Jul-Aug)
    "Bhadrapada",   // 6  (Aug-Sep)
    "Ashwin",       // 7  (Sep-Oct)
    "Kartik",       // 8  (Oct-Nov)
    "Margashirsha", // 9  (Nov-Dec)
    "Pausha",       // 10 (Dec-Jan)
    "Magha",        // 11 (Jan-Feb)
    "Phalguna",     // 12 (Feb-Mar)
];

/// Maps Rashi index (0=Mesha) to lunar month number (1=Chaitra).
/// When Sun enters Rashi R, the lunar month containing that Sankranti
/// is named LUNAR_MONTH_NAMES[SANKRANTI_TO_LUNAR_MONTH[R] - 1].
pub const SANKRANTI_TO_LUNAR_MONTH: [u32; 12] = [
    // Mesha Vrishabha Mithuna Karka Simha Kanya Tula Vrischika Dhanu Makara Kumbha Meena
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
];
