# Lilavati — SOTA Computation Architecture

## Core Principle

**Rust computes. Python exposes.**

The computation hot path lives in Rust, compiled to a native Python extension via PyO3/maturin. The Python package (`pip install lilavati`) is the public API — types, convenience functions, documentation — but every expensive calculation dispatches to Rust.

This is the same pattern used by Polars, Pydantic v2, Ruff, cryptography, and orjson — the SOTA pattern for high-performance Python libraries in 2026.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  User Code (Python)                                 │
│  from lilavati import panchang                      │
│  result = panchang.compute(date(2026, 2, 24), loc)  │
└────────────────────┬────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────┐
│  lilavati (Python package)                          │
│  - Public API: panchang.compute(), choghadiya()     │
│  - Pydantic models: PanchangData, SunData, etc.     │
│  - Timezone handling, input validation               │
│  - Calls into Rust via lilavati._core                │
└────────────────────┬────────────────────────────────┘
                     │ PyO3 FFI boundary
┌────────────────────▼────────────────────────────────┐
│  lilavati-core (Rust crate)                         │
│  - Swiss Ephemeris bindings (swisseph-rs or FFI)    │
│  - Angle arithmetic, normalization, interpolation   │
│  - Transition search (bisection + Newton-Raphson)   │
│  - Panchang engine (tithi, nakshatra, yoga, karana) │
│  - Sunrise/sunset with Hindu rising method          │
│  - Muhurat window computation                       │
│  - Topographic horizon (SRTM elevation data)        │
│  - Batch operations (full year in milliseconds)     │
└─────────────────────────────────────────────────────┘
                     │
                     ▼  also compiles to
┌─────────────────────────────────────────────────────┐
│  lilavati-wasm (WebAssembly)                        │
│  - Same Rust core compiled via wasm-pack            │
│  - First Vedic calendar library in the browser      │
│  - npm install @lilavati/core                       │
└─────────────────────────────────────────────────────┘
```

## Project Structure

```
/Workspace/lilavati/
├── crates/
│   └── lilavati-core/          # Rust computation engine
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs           # Crate root, PyO3 module definition
│       │   ├── ephemeris.rs     # Swiss Ephemeris wrapper
│       │   ├── panchang.rs      # 5 Panchanga elements
│       │   ├── sun.rs           # Sunrise/sunset (Hindu rising)
│       │   ├── moon.rs          # Moon position, phase
│       │   ├── muhurat.rs       # Rahu Kalam, Choghadiya, etc.
│       │   ├── angles.rs        # Angle math, normalization, crossing
│       │   ├── julian.rs        # Julian Day <-> datetime conversion
│       │   ├── search.rs        # Bisection + Newton-Raphson transition search
│       │   ├── types.rs         # Internal Rust types
│       │   ├── constants.rs     # Nakshatra names, Tithi names, etc.
│       │   └── topographic.rs   # SRTM horizon profile (future)
│       └── tests/
│           ├── test_ephemeris.rs
│           ├── test_panchang.rs
│           └── test_sun.rs
├── lilavati/                    # Python package (thin wrapper)
│   ├── __init__.py
│   ├── _core.pyi               # Type stubs for Rust extension
│   ├── panchang.py              # Public API
│   ├── types.py                 # Pydantic models
│   ├── core/                    # Pure Python fallback (optional)
│   └── muhurat/
├── tests/                       # Python integration tests
├── pyproject.toml               # maturin build backend
├── Cargo.toml                   # Workspace root
└── docs/
```

## Rust Crate Design

### `angles.rs` — Angle Arithmetic

All angle math in one place. Branchless where possible.

```rust
/// Normalize angle to [0, 360)
pub fn normalize(angle: f64) -> f64 {
    ((angle % 360.0) + 360.0) % 360.0
}

/// Angular distance (shortest arc) between two angles
pub fn angular_distance(a: f64, b: f64) -> f64 {
    let diff = normalize(b - a);
    if diff > 180.0 { 360.0 - diff } else { diff }
}

/// Check if angle `x` is between `start` and `end` on the circle
/// Handles wraparound (e.g., start=350, end=10 contains 355)
pub fn angle_between(x: f64, start: f64, end: f64) -> bool {
    let x = normalize(x);
    let s = normalize(start);
    let e = normalize(end);
    if s <= e { x >= s && x < e } else { x >= s || x < e }
}
```

### `search.rs` — Transition Time Search

The critical algorithm: finding exact moments when an angular quantity crosses a boundary.

```rust
/// Find the Julian Day when `angle_fn(jd)` crosses `target_angle`.
///
/// Uses bisection with optional Newton-Raphson refinement.
/// Precision: 1 second (1/86400 of a Julian Day).
///
/// The angle function must be monotonically increasing (modulo 360)
/// over the search interval.
pub fn find_crossing(
    jd_start: f64,
    jd_end: f64,
    target_angle: f64,
    angle_fn: impl Fn(f64) -> f64,
    speed_fn: Option<impl Fn(f64) -> f64>,
) -> f64 {
    const PRECISION: f64 = 1.0 / 86400.0; // 1 second
    const MAX_ITER: usize = 50;

    let mut lo = jd_start;
    let mut hi = jd_end;

    for _ in 0..MAX_ITER {
        if (hi - lo) < PRECISION {
            break;
        }

        let mid = (lo + hi) / 2.0;
        let angle = angle_fn(mid);

        if crossed(angle_fn(lo), target_angle, angle) {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    // Optional Newton-Raphson refinement if speed function provided
    if let Some(speed) = speed_fn {
        let jd = (lo + hi) / 2.0;
        let angle = angle_fn(jd);
        let delta = angular_delta(angle, target_angle);
        let refined = jd + delta / speed(jd) / 360.0;
        if refined > jd_start && refined < jd_end {
            return refined;
        }
    }

    (lo + hi) / 2.0
}
```

### `panchang.rs` — The 5 Elements

```rust
pub struct PanchangResult {
    pub tithi: TithiInfo,
    pub nakshatra: NakshatraInfo,
    pub yoga: YogaInfo,
    pub karana: KaranaInfo,
    pub vara: VaraInfo,
}

/// Compute Tithi number (1-30) at a given Julian Day.
/// Tithi = floor((moon_tropical - sun_tropical) % 360 / 12) + 1
pub fn tithi_at_jd(jd: f64, ephe: &Ephemeris) -> u8 {
    let sun = ephe.tropical_longitude(jd, Planet::Sun);
    let moon = ephe.tropical_longitude(jd, Planet::Moon);
    let diff = normalize(moon - sun);
    (diff / 12.0).floor() as u8 + 1
}

/// Compute Nakshatra number (1-27) at a given Julian Day.
/// Based on Moon's sidereal longitude.
pub fn nakshatra_at_jd(jd: f64, ephe: &Ephemeris) -> u8 {
    let moon_sid = ephe.sidereal_longitude(jd, Planet::Moon);
    (moon_sid / NAKSHATRA_SPAN).floor() as u8 + 1
}

/// Compute complete Panchang with transition times.
pub fn compute(jd_sunrise: f64, ephe: &Ephemeris) -> PanchangResult {
    // Current values at sunrise
    let tithi_num = tithi_at_jd(jd_sunrise, ephe);
    let nak_num = nakshatra_at_jd(jd_sunrise, ephe);
    let yoga_num = yoga_at_jd(jd_sunrise, ephe);
    let karana_num = karana_at_jd(jd_sunrise, ephe);

    // Find transition times using search::find_crossing
    let tithi_end = find_tithi_end(jd_sunrise, tithi_num, ephe);
    let tithi_start = find_tithi_start(jd_sunrise, tithi_num, ephe);
    // ... same for nakshatra, yoga, karana

    PanchangResult { /* ... */ }
}
```

### `sun.rs` — Sunrise/Sunset

```rust
/// Compute sunrise using Swiss Ephemeris swe_rise_trans.
/// Uses BIT_HINDU_RISING: disc center at horizon, Bhāratīya atmospheric model.
pub fn sunrise(jd_midnight: f64, lat: f64, lng: f64, alt: f64) -> f64 {
    // Calls swe_rise_trans with CALC_RISE | BIT_HINDU_RISING
    // Returns Julian Day of sunrise in UT
}

/// Compute sunset.
pub fn sunset(jd_noon: f64, lat: f64, lng: f64, alt: f64) -> f64 {
    // Calls swe_rise_trans with CALC_SET | BIT_HINDU_RISING
    // Returns Julian Day of sunset in UT
}
```

### `julian.rs` — Julian Day Conversions

```rust
/// Convert (year, month, day, hour, min, sec) in UTC to Julian Day.
/// Uses the standard astronomical algorithm (Meeus, Astronomical Algorithms).
pub fn datetime_to_jd(year: i32, month: u8, day: u8, hour: u8, min: u8, sec: f64) -> f64 {
    // Standard algorithm
}

/// Convert Julian Day back to UTC components.
pub fn jd_to_datetime(jd: f64) -> (i32, u8, u8, u8, u8, f64) {
    // Inverse algorithm
}
```

## Swiss Ephemeris in Rust

Two options for Swiss Ephemeris access from Rust:

### Option A: `swisseph-rs` crate (preferred if mature)
Direct Rust bindings to the Swiss Ephemeris C library. Check crates.io for availability and quality.

### Option B: Raw FFI to libswe
Bind directly to the Swiss Ephemeris C library using `cc` or `bindgen`:

```toml
# Cargo.toml
[build-dependencies]
cc = "1.0"
```

```rust
// build.rs — compile Swiss Ephemeris C source
fn main() {
    cc::Build::new()
        .files(["vendor/swisseph/swecl.c", "vendor/swisseph/swedate.c", ...])
        .compile("swisseph");
}
```

This gives us full control and removes the pyswisseph dependency from the Python side.

## PyO3 Bridge

The Rust crate exposes a Python module via PyO3:

```rust
use pyo3::prelude::*;

#[pyfunction]
fn compute_panchang(
    year: i32, month: u8, day: u8,
    lat: f64, lng: f64, alt: f64,
    tz: &str,
) -> PyResult<PyObject> {
    // 1. Convert date + tz to JD at local midnight
    // 2. Compute sunrise
    // 3. Compute all 5 elements at sunrise JD
    // 4. Return structured dict/dataclass
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_panchang, m)?)?;
    m.add_function(wrap_pyfunction!(compute_sunrise, m)?)?;
    m.add_function(wrap_pyfunction!(compute_sunset, m)?)?;
    // ...
    Ok(())
}
```

Python side:

```python
# lilavati/panchang.py
from lilavati._core import compute_panchang as _compute_raw
from lilavati.types import PanchangData

def compute(dt, location, *, include_muhurat=True):
    raw = _compute_raw(
        dt.year, dt.month, dt.day,
        location.lat, location.lng, location.altitude,
        location.tz,
    )
    return PanchangData(**raw)  # Pydantic validation + serialization
```

## Build System

### pyproject.toml (maturin backend)

```toml
[build-system]
requires = ["maturin>=1.5"]
build-backend = "maturin"

[project]
name = "lilavati"
version = "0.1.0"
requires-python = ">=3.10"
dependencies = ["pydantic>=2.0"]
# Note: pyswisseph is NO LONGER a dependency — Swiss Ephemeris
# is compiled directly into the Rust extension.

[tool.maturin]
features = ["pyo3/extension-module"]
module-name = "lilavati._core"
```

### Cargo.toml (workspace root)

```toml
[workspace]
members = ["crates/lilavati-core"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
```

### crates/lilavati-core/Cargo.toml

```toml
[package]
name = "lilavati-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
name = "_core"
crate-type = ["cdylib"]  # Python extension

[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }

[build-dependencies]
cc = "1.0"  # Compile Swiss Ephemeris C sources
```

### Build commands

```bash
# Development (builds Rust + installs Python package)
maturin develop --release

# Build wheel for distribution
maturin build --release

# Install with uv
uv pip install -e .
```

## Performance Targets

| Operation | Current (Python) | Target (Rust) | Speedup |
|-----------|-----------------|---------------|---------|
| Single Panchang | ~50ms | <1ms | 50x |
| Sunrise/Sunset | ~10ms | <0.2ms | 50x |
| Transition search (bisection) | ~20ms | <0.5ms | 40x |
| Full year Panchang (365 days) | ~18s | <200ms | 90x |
| Batch: 10 cities × 365 days | ~180s | <2s | 90x |

*Note: Swiss Ephemeris C calls are already fast. The speedup comes from eliminating Python overhead in the hot loop (angle math, bisection iterations, datetime conversions).*

## WASM Target (Phase 2)

The same Rust crate compiles to WebAssembly:

```toml
# crates/lilavati-wasm/Cargo.toml
[dependencies]
lilavati-core = { path = "../lilavati-core" }
wasm-bindgen = "0.2"
```

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn panchang(year: i32, month: u8, day: u8, lat: f64, lng: f64) -> JsValue {
    let result = lilavati_core::panchang::compute(/* ... */);
    serde_wasm_bindgen::to_value(&result).unwrap()
}
```

```bash
wasm-pack build --target web
# Produces: pkg/lilavati_wasm.js + .wasm
```

This makes Lilavati the first Vedic calendar library that runs natively in the browser.

---

## SOTA Differentiators

### 1. Topographic Sunrise (Future — after core Rust port)

Use NASA SRTM 30m elevation data to compute actual horizon profiles:

```rust
/// Compute the horizon elevation angle at a given azimuth from a location.
/// Uses SRTM DEM (Digital Elevation Model) tiles.
pub fn horizon_angle(lat: f64, lng: f64, azimuth: f64, dem: &SrtmTile) -> f64 {
    // Ray-march along azimuth direction
    // Find maximum elevation angle to terrain
    // Return angle in degrees above mathematical horizon
}

/// Adjusted sunrise: when sun disc center clears the actual terrain horizon.
pub fn topographic_sunrise(jd: f64, lat: f64, lng: f64, dem: &SrtmTile) -> f64 {
    let geometric = sunrise(jd, lat, lng, 0.0);
    let sun_azimuth = sun_azimuth_at(geometric, lat, lng);
    let horizon = horizon_angle(lat, lng, sun_azimuth, dem);
    // Adjust sunrise time for terrain obstruction
    // Sun needs to clear `horizon` degrees instead of 0 degrees
}
```

No Panchang engine in existence does this. For temple cities in valleys (Haridwar, Varanasi ghats, Tirupati), this can shift sunrise by 5-15 minutes — enough to change Tithi boundaries.

### 2. Uncertainty Quantification

Every computation returns a value + confidence interval:

```rust
pub struct UncertainTime {
    pub jd: f64,          // Best estimate
    pub sigma_seconds: f64, // 1-sigma uncertainty
    pub source: UncertaintySource,
}

pub enum UncertaintySource {
    AtmosphericRefraction,  // ±30-60s for sunrise/sunset
    AyanamsaModel,          // ±6 arcmin between Lahiri variants
    EphemerisPrecision,     // <1 arcsec for DE431 in modern era
    TopographicModel,       // Depends on DEM resolution
}
```

### 3. Continuous Validation Pipeline

GitHub Actions workflow that runs daily:

```yaml
# .github/workflows/accuracy.yml
name: Daily Accuracy Validation
on:
  schedule:
    - cron: '30 0 * * *'  # Daily at 00:30 UTC

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - name: Compute Panchang for 10 cities
      - name: Compare against Drik Panchang reference
      - name: Compare against Rashtriya Panchang
      - name: Update accuracy dashboard
      - name: Alert on regression (>2 minute deviation)
```

### 4. Property-Based Testing

Using `proptest` in Rust (equivalent of Hypothesis for Python):

```rust
use proptest::prelude::*;

proptest! {
    /// Tithis are always 1-30
    #[test]
    fn tithi_always_valid(jd in 2451545.0..2488070.0) {
        let t = tithi_at_jd(jd, &ephe);
        prop_assert!(t >= 1 && t <= 30);
    }

    /// Ketu is always exactly 180° from Rahu
    #[test]
    fn ketu_opposite_rahu(jd in 2451545.0..2488070.0) {
        let rahu = ephe.tropical_longitude(jd, Planet::Rahu);
        let ketu = ephe.tropical_longitude(jd, Planet::Ketu);
        let diff = angular_distance(rahu, ketu);
        prop_assert!((diff - 180.0).abs() < 0.001);
    }

    /// Sunrise is always before sunset for non-polar locations
    #[test]
    fn sunrise_before_sunset(
        jd in 2451545.0..2488070.0,
        lat in -60.0..60.0f64,
        lng in -180.0..180.0f64,
    ) {
        let rise = sunrise(jd, lat, lng, 0.0);
        let set = sunset(jd, lat, lng, 0.0);
        prop_assert!(rise < set);
    }

    /// Day duration on equinox is near 12 hours everywhere
    #[test]
    fn equinox_near_12_hours(
        lat in -60.0..60.0f64,
        lng in -180.0..180.0f64,
    ) {
        let equinox_jd = 2461383.0; // ~March 20, 2026
        let rise = sunrise(equinox_jd, lat, lng, 0.0);
        let set = sunset(equinox_jd, lat, lng, 0.0);
        let hours = (set - rise) * 24.0;
        prop_assert!(hours > 11.0 && hours < 13.0);
    }
}
```

---

## Migration Strategy

### Phase 1: Rust core with PyO3 (current focus)
1. Set up Cargo workspace + maturin build
2. Vendor Swiss Ephemeris C source, compile via `cc` crate
3. Port `ephemeris.py` → `ephemeris.rs` (planetary positions, ayanamsa)
4. Port `sun.py` → `sun.rs` (sunrise/sunset with Hindu rising)
5. Port angle math + transition search → `angles.rs` + `search.rs`
6. Port `panchang.py` → `panchang.rs` (5 elements with transition times)
7. Port `muhurat/windows.py` → `muhurat.rs`
8. PyO3 bridge: expose all functions to Python
9. Update Python layer to call `lilavati._core` instead of pyswisseph
10. All 40 existing Python tests must still pass
11. Add Rust-side tests with `proptest`

### Phase 2: WASM target
12. Create `lilavati-wasm` crate
13. Compile to WebAssembly via `wasm-pack`
14. Publish `@lilavati/core` to npm

### Phase 3: Topographic sunrise
15. SRTM tile loader
16. Horizon profile computation
17. Adjusted sunrise/sunset

### Phase 4: Accuracy pipeline
18. GitHub Actions daily validation
19. Public accuracy dashboard
