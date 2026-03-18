"""Python-side benchmark script for Lilavati.

Measures the PyO3 call overhead by timing the same operations
that the Rust criterion benchmarks measure.

Usage:
    uv run python benchmarks/compare.py
"""

import time
import statistics

from panchang._core import (
    py_init,
    py_close,
    py_compute_sun_data,
    py_compute_panchang,
    py_compute_muhurat,
    py_compute_choghadiya,
    py_tropical_longitude,
    py_sidereal_longitude,
    py_ayanamsa,
)

# Delhi, 2026-02-24
YEAR, MONTH, DAY = 2026, 2, 24
LAT, LNG, ALT = 28.6139, 77.2090, 216.0
IST_OFFSET = 19800
WEEKDAY = 2  # Tuesday


def bench(name: str, fn, iterations: int = 1000) -> float:
    """Run a function `iterations` times and return median time in microseconds."""
    times = []
    for _ in range(iterations):
        start = time.perf_counter_ns()
        fn()
        elapsed = time.perf_counter_ns() - start
        times.append(elapsed / 1000)  # ns -> µs
    median = statistics.median(times)
    mean = statistics.mean(times)
    p99 = sorted(times)[int(len(times) * 0.99)]
    print(f"  {name:40s}  median: {median:>10.2f} µs  mean: {mean:>10.2f} µs  p99: {p99:>10.2f} µs")
    return median


def main():
    py_init(None)

    # Get sun data for subsequent benchmarks
    sun_raw = py_compute_sun_data(YEAR, MONTH, DAY, LAT, LNG, ALT, IST_OFFSET)
    sunrise_jd = sun_raw["sunrise_jd"]
    sunset_jd = sun_raw["sunset_jd"]
    day_duration = sun_raw["day_duration_hours"]

    # JD for ~6AM Delhi
    jd_sunrise = sunrise_jd

    print("=" * 90)
    print("Lilavati Python Benchmarks (PyO3 boundary overhead)")
    print("=" * 90)
    print()

    # Ephemeris operations
    print("Ephemeris:")
    bench("tropical_longitude (Moon)", lambda: py_tropical_longitude(jd_sunrise, 1))
    bench("sidereal_longitude (Moon)", lambda: py_sidereal_longitude(jd_sunrise, 1))
    bench("ayanamsa", lambda: py_ayanamsa(jd_sunrise))
    print()

    # Sun data
    print("Sun:")
    bench(
        "compute_sun_data",
        lambda: py_compute_sun_data(YEAR, MONTH, DAY, LAT, LNG, ALT, IST_OFFSET),
        iterations=500,
    )
    print()

    # Full panchang
    print("Panchang:")
    bench(
        "compute_panchang (all 5 elements)",
        lambda: py_compute_panchang(YEAR, MONTH, DAY, LAT, LNG, ALT, IST_OFFSET, WEEKDAY),
        iterations=200,
    )
    print()

    # Muhurat
    print("Muhurat:")
    bench(
        "compute_muhurat (4 windows)",
        lambda: py_compute_muhurat(WEEKDAY, sunrise_jd, day_duration),
    )
    bench(
        "compute_choghadiya (16 windows)",
        lambda: py_compute_choghadiya(WEEKDAY, sunrise_jd, sunset_jd, day_duration),
    )
    print()

    py_close()

    print("=" * 90)
    print("Note: These measure Rust computation + PyO3 serialization overhead.")
    print("Compare with `cargo bench` for pure Rust numbers.")
    print("=" * 90)


if __name__ == "__main__":
    main()
