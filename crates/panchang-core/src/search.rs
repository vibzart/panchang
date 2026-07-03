//! Bisection search for finding exact angular transition times.
//!
//! Used to find when tithi, nakshatra, yoga, and karana boundaries are crossed.

use crate::angles::crossed_target;

/// Precision: ~1 second in Julian Days.
const BISECT_PRECISION_JD: f64 = 1.0 / 86400.0;
const MAX_BISECT_ITER: u32 = 50;
/// Default step size for bracket search: 1 hour.
const BRACKET_STEP: f64 = 1.0 / 24.0;
/// Coarse bracket step: 1 day. Safe for any angle that sweeps < 180° per
/// step (`crossed_target` handles wraparound for sweeps below a half turn):
/// the tithi angle moves ~12-14°/day and the Sun's sidereal longitude
/// ~1°/day, so day-sized brackets are safe for amavasya/purnima and
/// sankranti searches and cut the ephemeris-call count ~24x.
pub const BRACKET_STEP_DAY: f64 = 1.0;

/// Find the next forward crossing of `target_angle` by `angle_fn`.
///
/// Searches forward from `jd` up to `max_days` ahead.
/// Returns `None` if no crossing is found.
pub fn find_crossing_forward(
    jd: f64,
    target_angle: f64,
    angle_fn: &dyn Fn(f64) -> f64,
    max_days: f64,
) -> Option<f64> {
    find_crossing_forward_step(jd, target_angle, angle_fn, max_days, BRACKET_STEP)
}

/// `find_crossing_forward` with a caller-chosen bracket step.
///
/// The step must keep the angle sweep per step below 180° so
/// `crossed_target` can distinguish a crossing from wraparound.
pub fn find_crossing_forward_step(
    jd: f64,
    target_angle: f64,
    angle_fn: &dyn Fn(f64) -> f64,
    max_days: f64,
    step: f64,
) -> Option<f64> {
    let jd_end = jd + max_days;
    let mut jd_a = jd;
    let mut jd_b = jd_a + step;
    // Carry the previous step's end value forward — halves ephemeris calls.
    let mut angle_a = angle_fn(jd_a);

    while jd_b <= jd_end {
        let angle_b = angle_fn(jd_b);

        if crossed_target(angle_a, angle_b, target_angle) {
            return Some(bisect_crossing(jd_a, jd_b, angle_a, target_angle, angle_fn));
        }

        jd_a = jd_b;
        angle_a = angle_b;
        jd_b += step;
    }

    None
}

/// Find the most recent backward crossing of `target_angle` by `angle_fn`.
///
/// Searches backward from `jd` up to `max_days` back.
/// Returns `None` if no crossing is found.
pub fn find_crossing_backward(
    jd: f64,
    target_angle: f64,
    angle_fn: &dyn Fn(f64) -> f64,
    max_days: f64,
) -> Option<f64> {
    let jd_start = jd - max_days;
    let mut jd_b = jd;
    let mut jd_a = jd_b - BRACKET_STEP;
    let mut angle_b = angle_fn(jd_b);

    while jd_a >= jd_start {
        let angle_a = angle_fn(jd_a);

        if crossed_target(angle_a, angle_b, target_angle) {
            return Some(bisect_crossing(jd_a, jd_b, angle_a, target_angle, angle_fn));
        }

        jd_b = jd_a;
        angle_b = angle_a;
        jd_a -= BRACKET_STEP;
    }

    None
}

/// Bisection search: refine a bracket [jd_a, jd_b] to find the exact
/// moment the angle function crosses the target.
///
/// `angle_a` is the already-computed value at `jd_a`; the low-end value
/// only changes when the low bracket moves, so it is carried instead of
/// recomputed each iteration.
fn bisect_crossing(
    mut jd_a: f64,
    mut jd_b: f64,
    mut angle_a: f64,
    target: f64,
    angle_fn: &dyn Fn(f64) -> f64,
) -> f64 {
    for _ in 0..MAX_BISECT_ITER {
        if (jd_b - jd_a) < BISECT_PRECISION_JD {
            break;
        }
        let jd_mid = (jd_a + jd_b) / 2.0;
        let angle_mid = angle_fn(jd_mid);

        if crossed_target(angle_a, angle_mid, target) {
            jd_b = jd_mid;
        } else {
            jd_a = jd_mid;
            angle_a = angle_mid;
        }
    }
    (jd_a + jd_b) / 2.0
}
