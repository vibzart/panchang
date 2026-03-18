# Algorithm Documentation

Technical documentation for the astronomical and calendrical algorithms used in Lilavati.

## Contents

| Document | Description |
|---|---|
| [panchang.md](panchang.md) | The 5 Panchanga elements: Tithi, Nakshatra, Yoga, Karana, Vara |
| [sankranti.md](sankranti.md) | Sankranti (solar ingress) computation |
| [lunar-month.md](lunar-month.md) | Lunar month determination (Amant and Purnimant systems) |
| [festival-resolution.md](festival-resolution.md) | Festival date resolution using Sankranti-based search |
| [ekadashi.md](ekadashi.md) | Ekadashi rules (Smartha and Vaishnava) |
| [batch.md](batch.md) | Batch computation for full-year calendar generation |

## Architecture

All computation is performed in Rust (`crates/lilavati-core/src/`). Python provides thin typed wrappers via PyO3.

The ephemeris engine is the Swiss Ephemeris (Moshier analytical model), vendored as C and accessed via FFI. No external ephemeris data files are needed.

## Key Constants

- **Ayanamsa**: Lahiri/Chitrapaksha (Government of Bhārat standard), ~24.22 degrees in 2026
- **Sunrise model**: Hindu rising (disc center at horizon, Bhāratīya atmospheric model)
- **Bisection precision**: 1 second (1/86400 Julian Day), max 50 iterations
- **Search bracket**: 1-hour steps forward/backward

## Validation

All algorithms are cross-validated against [Drik Panchang](https://www.drikpanchang.com/) for Delhi (28.6139N, 77.2090E). Timing differences of 1-5 minutes are expected due to different sunrise geometric models.
