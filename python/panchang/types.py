"""Data types for Lilavati — Bhāratīya calendar infrastructure."""

from __future__ import annotations

from datetime import date as Date  # noqa: N812 — alias needed to avoid Pydantic field-name clash
from datetime import datetime
from enum import IntEnum, StrEnum
from typing import Optional

from pydantic import BaseModel, Field

# --- Enums ---


class Paksha(StrEnum):
    """Lunar fortnight — Shukla (waxing) or Krishna (waning)."""

    SHUKLA = "Shukla"
    KRISHNA = "Krishna"


class Vara(IntEnum):
    """Weekday (Vara). Sunday=0 through Saturday=6."""

    RAVIVARA = 0  # Sunday
    SOMAVARA = 1  # Monday
    MANGALAVARA = 2  # Tuesday
    BUDHAVARA = 3  # Wednesday
    GURUVARA = 4  # Thursday
    SHUKRAVARA = 5  # Friday
    SHANIVARA = 6  # Saturday


VARA_NAMES = {
    Vara.RAVIVARA: "Ravivara",
    Vara.SOMAVARA: "Somavara",
    Vara.MANGALAVARA: "Mangalavara",
    Vara.BUDHAVARA: "Budhavara",
    Vara.GURUVARA: "Guruvara",
    Vara.SHUKRAVARA: "Shukravara",
    Vara.SHANIVARA: "Shanivara",
}

VARA_ENGLISH = {
    Vara.RAVIVARA: "Sunday",
    Vara.SOMAVARA: "Monday",
    Vara.MANGALAVARA: "Tuesday",
    Vara.BUDHAVARA: "Wednesday",
    Vara.GURUVARA: "Thursday",
    Vara.SHUKRAVARA: "Friday",
    Vara.SHANIVARA: "Saturday",
}

# --- Constants ---

NAKSHATRA_NAMES = [
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
]

NAKSHATRA_LORDS = [
    "Ketu",
    "Venus",
    "Sun",
    "Moon",
    "Mars",
    "Rahu",
    "Jupiter",
    "Saturn",
    "Mercury",
    "Ketu",
    "Venus",
    "Sun",
    "Moon",
    "Mars",
    "Rahu",
    "Jupiter",
    "Saturn",
    "Mercury",
    "Ketu",
    "Venus",
    "Sun",
    "Moon",
    "Mars",
    "Rahu",
    "Jupiter",
    "Saturn",
    "Mercury",
]

TITHI_NAMES = [
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
    "Purnima",  # Full moon (end of Shukla paksha)
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
    "Amavasya",  # New moon (end of Krishna paksha)
]

YOGA_NAMES = [
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
]

# 11 Karanas: 4 fixed + 7 rotating
KARANA_NAMES = [
    "Bava",
    "Balava",
    "Kaulava",
    "Taitila",
    "Garaja",
    "Vanija",
    "Vishti",  # 7 rotating (Vishti = Bhadra, inauspicious)
    "Shakuni",  # Fixed: Krishna Chaturdashi, 2nd half
    "Chatushpada",  # Fixed: Amavasya, 1st half
    "Nagava",  # Fixed: Amavasya, 2nd half (sometimes called Naga)
    "Kimstughna",  # Fixed: Shukla Pratipada, 1st half
]


# --- Location ---


class Location(BaseModel):
    """Geographic location for calendar computations."""

    lat: float = Field(..., ge=-90, le=90, description="Latitude in decimal degrees")
    lng: float = Field(..., ge=-180, le=180, description="Longitude in decimal degrees")
    altitude: float = Field(default=0.0, ge=0, description="Altitude in meters above sea level")
    tz: str = Field(default="UTC", description="IANA timezone string (e.g. 'Asia/Kolkata')")


# --- Panchang element models ---


class TithiInfo(BaseModel):
    """Tithi (lunar day) information."""

    number: int = Field(..., ge=1, le=30, description="Tithi number (1-30)")
    name: str = Field(..., description="Tithi name (e.g. 'Ashtami')")
    paksha: Paksha = Field(..., description="Shukla (waxing) or Krishna (waning)")
    start: Optional[datetime] = Field(None, description="Tithi start time (UTC)")
    end: Optional[datetime] = Field(None, description="Tithi end time (UTC)")


class NakshatraInfo(BaseModel):
    """Nakshatra (lunar mansion) information."""

    number: int = Field(..., ge=1, le=27, description="Nakshatra number (1-27)")
    name: str = Field(..., description="Nakshatra name (e.g. 'Rohini')")
    pada: int = Field(..., ge=1, le=4, description="Pada (quarter) 1-4")
    lord: str = Field(..., description="Ruling planet")
    start: Optional[datetime] = Field(None, description="Nakshatra start time (UTC)")
    end: Optional[datetime] = Field(None, description="Nakshatra end time (UTC)")


class YogaInfo(BaseModel):
    """Yoga (Sun-Moon combination) information."""

    number: int = Field(..., ge=1, le=27, description="Yoga number (1-27)")
    name: str = Field(..., description="Yoga name (e.g. 'Siddhi')")
    start: Optional[datetime] = Field(None, description="Yoga start time (UTC)")
    end: Optional[datetime] = Field(None, description="Yoga end time (UTC)")


class KaranaInfo(BaseModel):
    """Karana (half-tithi) information."""

    number: int = Field(..., ge=1, le=11, description="Karana number (1-11)")
    name: str = Field(..., description="Karana name (e.g. 'Bava')")
    start: Optional[datetime] = Field(None, description="Karana start time (UTC)")
    end: Optional[datetime] = Field(None, description="Karana end time (UTC)")


class VaraInfo(BaseModel):
    """Vara (weekday) information."""

    number: int = Field(..., ge=0, le=6, description="Weekday number (0=Sunday)")
    name: str = Field(..., description="Sanskrit weekday name")
    english: str = Field(..., description="English weekday name")


# --- Sun/Moon data ---


class SunData(BaseModel):
    """Sunrise, sunset, and related solar data for a location and date."""

    sunrise: datetime = Field(..., description="Sunrise time (local timezone)")
    sunset: datetime = Field(..., description="Sunset time (local timezone)")
    day_duration_hours: float = Field(..., description="Duration of daytime in hours")
    sunrise_jd: float = Field(default=0.0, description="Sunrise Julian Day (internal)")
    sunset_jd: float = Field(default=0.0, description="Sunset Julian Day (internal)")


class MoonData(BaseModel):
    """Moon position and phase data."""

    longitude: float = Field(..., description="Moon sidereal longitude in degrees")
    phase_angle: float = Field(..., description="Sun-Moon angle (0=new, 180=full)")


# --- Time windows ---


class TimeWindow(BaseModel):
    """A named time window with start and end times."""

    name: str = Field(..., description="Window name (e.g. 'Rahu Kalam')")
    start: datetime = Field(..., description="Start time (local timezone)")
    end: datetime = Field(..., description="End time (local timezone)")
    is_auspicious: bool = Field(..., description="Whether this window is auspicious")


# --- Composite Panchang result ---


class PanchangData(BaseModel):
    """Complete Panchang data for a date and location."""

    date: str = Field(..., description="Date in ISO format (YYYY-MM-DD)")
    location: Location
    sun: SunData
    vara: VaraInfo
    tithi: TithiInfo
    nakshatra: NakshatraInfo
    yoga: YogaInfo
    karana: KaranaInfo
    rahu_kalam: Optional[TimeWindow] = None
    yama_gandam: Optional[TimeWindow] = None
    gulika_kalam: Optional[TimeWindow] = None
    abhijit_muhurat: Optional[TimeWindow] = None


# --- Calendar system ---


class CalendarSystem(StrEnum):
    """Hindu lunar calendar system."""

    AMANT = "amant"
    PURNIMANT = "purnimant"


# --- Sankranti ---


class SankrantiData(BaseModel):
    """Sankranti (solar ingress) information."""

    index: int = Field(..., ge=0, le=11, description="Sankranti index (0=Makar)")
    name: str = Field(..., description="Sankranti name (e.g. 'Makar Sankranti')")
    rashi: str = Field(..., description="Rashi being entered (e.g. 'Makara')")
    target_longitude: float = Field(..., description="Target sidereal longitude in degrees")
    date: Date = Field(..., description="Date of transit")


# --- Lunar month ---


class LunarMonthData(BaseModel):
    """Lunar month information."""

    number: int = Field(..., ge=0, le=12, description="Month number (1-12, Chaitra=1)")
    name: str = Field(..., description="Month name (e.g. 'Chaitra')")
    is_adhik: bool = Field(
        default=False,
        description="Whether this is an Adhik (intercalary) month",
    )
    is_kshaya: bool = Field(
        default=False,
        description="Whether this is a Kshaya (compressed) month",
    )
    start: datetime = Field(..., description="Month start time (UTC)")
    end: datetime = Field(..., description="Month end time (UTC)")


# --- Festival ---


class FestivalInfo(BaseModel):
    """Resolved festival date with reasoning."""

    id: str = Field(..., description="Festival identifier (e.g. 'diwali')")
    name: str = Field(..., description="Festival display name")
    date: Date = Field(..., description="Festival date")
    sunrise: Optional[datetime] = Field(None, description="Sunrise time on festival day")
    tithi_at_sunrise: int = Field(..., ge=0, le=30, description="Tithi number at sunrise")
    lunar_month: str = Field(..., description="Lunar month name")
    is_adhik_month: bool = Field(default=False, description="Whether in Adhik month")
    reasoning: str = Field(..., description="Explanation of date determination")


# --- Ekadashi ---


class EkadashiInfo(BaseModel):
    """Resolved Ekadashi with Smartha and Vaishnava dates."""

    name: str = Field(..., description="Ekadashi name (e.g. 'Kamada')")
    lunar_month: int = Field(..., ge=1, le=12, description="Lunar month number")
    lunar_month_name: str = Field(..., description="Lunar month name")
    paksha: Paksha = Field(..., description="Shukla or Krishna")
    smartha_date: Date = Field(..., description="Smartha Ekadashi date")
    vaishnava_date: Date = Field(..., description="Vaishnava Ekadashi date")
    reasoning: str = Field(..., description="Explanation of date determination")


# --- Vrat ---


class VratInfo(BaseModel):
    """Resolved Vrat (fasting) date."""

    vrat_type: str = Field(..., description="Type (e.g. 'Pradosh Vrat', 'Amavasya')")
    name: str = Field(..., description="Display name (e.g. 'Pradosh Vrat (Chaitra)')")
    date: Date = Field(..., description="Vrat date")
    lunar_month: str = Field(..., description="Lunar month name")
    paksha: Paksha = Field(..., description="Shukla or Krishna")


# --- Batch ---


class BatchDayData(BaseModel):
    """Panchang data for a single day in a batch computation."""

    date: Date
    vara: VaraInfo
    tithi: TithiInfo
    nakshatra: NakshatraInfo
    yoga: YogaInfo
    karana: KaranaInfo
    sun: SunData


# --- Regional Calendar ---


class RegionalMonthInfo(BaseModel):
    """A single month in a regional calendar."""

    name: str = Field(..., description="Regional month name (e.g. 'Chithirai')")
    standard_name: str = Field(..., description="Standard Sanskrit month name")
    start: Date = Field(..., description="Month start date")
    end: Date = Field(..., description="Month end date")


class RegionalCalendarData(BaseModel):
    """Complete regional calendar for a year."""

    id: str = Field(..., description="Calendar identifier (e.g. 'tamil')")
    name: str = Field(..., description="Calendar display name")
    language: str = Field(..., description="Language (e.g. 'Tamil')")
    calendar_type: str = Field(..., description="'solar' or 'lunar'")
    era_name: str = Field(..., description="Era name (e.g. 'Vikram Samvat')")
    era_year: int = Field(..., description="Year in the regional era")
    jovian_year_name: Optional[str] = Field(None, description="60-year Jovian cycle year name")
    months: list[RegionalMonthInfo] = Field(..., description="Months in regional order")
    new_year_date: Optional[Date] = Field(None, description="Regional New Year date")


# --- Shraddha ---


class ShraddhaData(BaseModel):
    """Shraddha (death anniversary) date resolution."""

    death_date: Date = Field(..., description="Original death date")
    tithi: int = Field(..., ge=1, le=30, description="Tithi number on death date")
    lunar_month: int = Field(..., ge=1, le=12, description="Lunar month number")
    lunar_month_name: str = Field(..., description="Lunar month name")
    shraddha_date: Date = Field(..., description="Resolved Shraddha date for target year")
    reasoning: str = Field(..., description="Explanation of date determination")
