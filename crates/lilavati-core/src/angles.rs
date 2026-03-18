//! Angle arithmetic for astronomical calculations.
//!
//! All angles are in degrees [0, 360). Handles wraparound correctly.

#![allow(dead_code)]

/// Normalize an angle to [0, 360).
#[inline]
pub fn normalize(angle: f64) -> f64 {
    ((angle % 360.0) + 360.0) % 360.0
}

/// Forward angular distance from `a` to `target` on a circle.
/// Always returns a value in [0, 360).
#[inline]
pub fn forward_distance(a: f64, target: f64) -> f64 {
    (normalize(target) - normalize(a) + 360.0) % 360.0
}

/// Check if the angle crossed the target between `angle_a` and `angle_b`.
///
/// Assumes forward motion (angle increasing). Handles 360° wraparound.
/// The target is "crossed" if it lies within the forward sweep from a to b,
/// and the sweep is less than 180° (reasonable for 1-hour steps).
pub fn crossed_target(angle_a: f64, angle_b: f64, target: f64) -> bool {
    let a = normalize(angle_a);
    let b = normalize(angle_b);
    let t = normalize(target);

    // Forward distance from a to b
    let fwd_ab = (b - a + 360.0) % 360.0;
    // Forward distance from a to target
    let fwd_at = (t - a + 360.0) % 360.0;

    // Target is crossed if it lies within the forward sweep from a to b
    // and the sweep is less than 180° (sanity check for 1-hour steps)
    fwd_ab < 180.0 && fwd_at <= fwd_ab
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert!((normalize(370.0) - 10.0).abs() < 1e-10);
        assert!((normalize(-10.0) - 350.0).abs() < 1e-10);
        assert!((normalize(0.0) - 0.0).abs() < 1e-10);
        assert!((normalize(360.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_forward_distance() {
        assert!((forward_distance(350.0, 10.0) - 20.0).abs() < 1e-10);
        assert!((forward_distance(10.0, 350.0) - 340.0).abs() < 1e-10);
    }

    #[test]
    fn test_crossed_target() {
        assert!(crossed_target(10.0, 20.0, 15.0));
        assert!(!crossed_target(10.0, 20.0, 25.0));
        // Wraparound: 350 -> 10 should cross 0/360
        assert!(crossed_target(350.0, 10.0, 5.0));
        assert!(crossed_target(350.0, 10.0, 355.0));
    }
}
