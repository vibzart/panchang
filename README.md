# Panchang

[![PyPI](https://img.shields.io/pypi/v/panchang)](https://pypi.org/project/panchang/)
[![Python](https://img.shields.io/pypi/pyversions/panchang)](https://pypi.org/project/panchang/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/vibzart/lilavati-oss/actions/workflows/ci.yml/badge.svg)](https://github.com/vibzart/lilavati-oss/actions/workflows/ci.yml)

**Bhāratīya calendar infrastructure for developers** — Panchang, festivals, muhurat, regional calendars, and batch computation.

Built on the Lilavati computation engine, named after Bhāskarāchārya's 12th-century mathematical treatise. Not an astrology API. It answers: *"What is happening in the Bhāratīya calendar right now, at this location?"*

```python
from datetime import date
from panchang import panchang, calendar, Location

delhi = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")

# Daily Panchang
today = panchang.compute(date.today(), delhi)
print(today.tithi.name)        # "Shukla Dvitiya"
print(today.nakshatra.name)    # "Pushya"
print(today.nakshatra.pada)    # 3
print(today.sunrise)           # 2026-03-03 06:42:18+05:30

# Festival dates
festivals = calendar.compute_festivals(2026, delhi)
for f in festivals[:3]:
    print(f"{f.name}: {f.date}")
# Makar Sankranti: 2026-01-14
# Vasant Panchami: 2026-02-01
# Maha Shivaratri: 2026-02-15
```

## Install

```bash
pip install panchang
```

Requires Python 3.11+. Wheels available for Linux, macOS, and Windows.

## Features

### Panchang (Daily Calendar)
All 5 Panchanga elements with precise transition times (start/end to the second):
- **Tithi** (lunar day) — with Paksha (Shukla/Krishna)
- **Nakshatra** (lunar mansion) — with Pada (1-4)
- **Yoga** (Sun-Moon combination)
- **Karana** (half-tithi)
- **Vara** (weekday)

### Sun & Moon
- Sunrise/sunset using Hindu rising model (disc center at horizon, Bhāratīya atmospheric refraction)
- Location-aware computation for any lat/lng/timezone

### Muhurat (Auspicious Windows)
- Rahu Kalam, Yama Gandam, Gulika Kalam
- Abhijit Muhurat
- Choghadiya (16 windows per day — 8 day + 8 night)

### Festivals
55+ Hindu festivals astronomically computed for any year:
- **Tithi-based**: Diwali, Holi, Janmashtami, Ram Navami, Ganesh Chaturthi, Navaratri, ...
- **Sankranti-based**: Makar Sankranti, Pongal, Vishu, Bihu, ...
- **Nakshatra-based**: Onam (Thiruvonam)
- **Ekadashi**: All 24 per year with Smartha and Vaishnava dates
- **Vrat dates**: Pradosh, Sankashti Chaturthi, Amavasya, Purnima (~60 per year)

Festival definitions are data-driven (YAML, not hardcoded) with year-agnostic astronomical rules. Each resolved date includes a reasoning string explaining the determination.

### Regional Calendars
8 regional calendar systems with proper era numbering:
- **Solar**: Tamil, Bengali, Malayalam, Kannada
- **Lunar**: Hindi, Marathi, Telugu, Gujarati
- Era support: Vikram Samvat, Shaka Samvat, Bangabda, Kollavarsham, Thiruvalluvar, 60-year Jovian cycle

### Lunar Months
- Both Amant (South Bhārat) and Purnimant (North Bhārat) systems
- Adhik Maas (intercalary month) and Kshaya Maas detection

### Shraddha Tithi
Death anniversary date resolution — given a death date, computes the Shraddha date for any target year using the lunar tithi and month.

### Batch Computation
Full-year or date-range Panchang in a single call:
```python
from panchang import batch, Location

delhi = Location(lat=28.6139, lng=77.2090, tz="Asia/Kolkata")
year_data = batch.compute_year(2026, delhi)  # 365 days of Panchang
```

## Accuracy

All computations use the **Swiss Ephemeris** (Moshier analytical model) with **Lahiri/Chitrapaksha Ayanamsa** — the Government of Bhārat standard.

Cross-validated against Drik Panchang for 2026-02-24, Delhi:

| Element | Drik Panchang | Panchang | Delta |
|---|---|---|---|
| Sunrise | 06:51 | 06:55 | ~4 min |
| Tithi | Shukla Saptami until 07:01 | Shukla Saptami until 07:02 | ~1 min |
| Nakshatra | Krittika until 15:07 | Krittika until 15:07 | exact |
| Yoga | Indra until 07:24 | Indra until 07:23 | ~1 min |
| Karana | Vanija until 07:01 | Vanija until 07:02 | ~1 min |
| Rahu Kalam | 15:26-16:52 | 15:24-16:48 | ~2 min |

All element **names match exactly**. Timing differences are 1-5 minutes due to sunrise geometric model variations.

## Architecture

Rust core + Python API. All astronomical math runs in Rust via PyO3, giving C-level performance with a Pythonic interface.

```
Python (pydantic models, typed API)
  └── Rust via PyO3 (panchang, festivals, muhurat, batch)
        └── Swiss Ephemeris C (planetary positions via FFI)
```

**Performance** (Rust benchmarks):
- Full Panchang: ~3.7 ms
- Sunrise: ~29 µs
- All 9 planets: ~7 µs

## Development

```bash
# Setup
uv venv
uv pip install -e ".[dev]"

# Build Rust extension
maturin develop --uv

# Run tests
uv run pytest tests/ -v
cargo test --manifest-path crates/lilavati-core/Cargo.toml -- --test-threads=1

# Lint
uv run ruff check python/ tests/
cargo clippy --manifest-path crates/lilavati-core/Cargo.toml -- -D warnings
```

## Tech Stack

| Layer | Technology |
|---|---|
| Computation | Rust + PyO3 |
| Ephemeris | Swiss Ephemeris (vendored C, Moshier model) |
| Python | 3.11+ with Pydantic v2 |
| Build | maturin + uv |
| Testing | pytest + proptest + criterion |
| CI | GitHub Actions + maturin-action |

## Contributing

Contributions welcome! Please open an issue first to discuss what you'd like to change.

```bash
# Run the full check suite before submitting
uv run ruff check python/ tests/
uv run pytest tests/ -v
cargo test --manifest-path crates/lilavati-core/Cargo.toml -- --test-threads=1
cargo clippy --manifest-path crates/lilavati-core/Cargo.toml -- -D warnings
```

## License

MIT
