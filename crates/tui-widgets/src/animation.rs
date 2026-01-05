//! Animation easing functions for smooth transitions.
//!
//! Used for smooth scrolling, expand/collapse transitions, and selection highlights.

use std::f32::consts::PI;

/// Easing function for animations.
///
/// Determines how an animation progresses over time. The input `t` is a value
/// from 0.0 to 1.0 representing the progress of the animation, and the output
/// is the eased value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    /// Linear interpolation (no easing)
    Linear,
    /// Slow start, fast end
    EaseIn,
    /// Fast start, slow end
    EaseOut,
    /// Slow start and end, fast middle
    EaseInOut,
    /// Bouncy effect at the end
    Bounce,
    /// Elastic/spring effect
    Elastic,
    /// Custom cubic bezier curve with control points (CSS-style)
    ///
    /// The four values represent the x1, y1, x2, y2 control points of the curve.
    /// Common presets:
    /// - ease: (0.25, 0.1, 0.25, 1.0)
    /// - ease-in: (0.42, 0.0, 1.0, 1.0)
    /// - ease-out: (0.0, 0.0, 0.58, 1.0)
    /// - ease-in-out: (0.42, 0.0, 0.58, 1.0)
    CubicBezier(f32, f32, f32, f32),
}

impl Default for EasingFunction {
    fn default() -> Self {
        Self::EaseInOut
    }
}

impl EasingFunction {
    /// Apply the easing function to a progress value.
    ///
    /// # Arguments
    /// * `t` - Progress value from 0.0 to 1.0
    ///
    /// # Returns
    /// The eased value, typically also in 0.0 to 1.0 range
    /// (though some functions like Bounce and Elastic may overshoot)
    pub fn ease(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,
            Self::EaseIn => t * t * t,
            Self::EaseOut => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            Self::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c4 = (2.0 * PI) / 3.0;
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Self::CubicBezier(x1, y1, x2, y2) => {
                cubic_bezier_ease(t, *x1, *y1, *x2, *y2)
            }
        }
    }

    /// Common CSS ease preset
    pub fn css_ease() -> Self {
        Self::CubicBezier(0.25, 0.1, 0.25, 1.0)
    }

    /// Common CSS ease-in preset
    pub fn css_ease_in() -> Self {
        Self::CubicBezier(0.42, 0.0, 1.0, 1.0)
    }

    /// Common CSS ease-out preset
    pub fn css_ease_out() -> Self {
        Self::CubicBezier(0.0, 0.0, 0.58, 1.0)
    }

    /// Common CSS ease-in-out preset
    pub fn css_ease_in_out() -> Self {
        Self::CubicBezier(0.42, 0.0, 0.58, 1.0)
    }
}

/// Calculate cubic bezier easing.
///
/// Uses Newton-Raphson iteration to solve for t given x, then evaluates y at that t.
fn cubic_bezier_ease(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Find t for given x using Newton-Raphson
    let mut t = x;
    for _ in 0..8 {
        let x_at_t = cubic_bezier(t, x1, x2);
        let dx = x - x_at_t;
        if dx.abs() < 1e-6 {
            break;
        }
        let derivative = cubic_bezier_derivative(t, x1, x2);
        if derivative.abs() < 1e-6 {
            break;
        }
        t += dx / derivative;
        t = t.clamp(0.0, 1.0);
    }

    // Evaluate y at found t
    cubic_bezier(t, y1, y2)
}

/// Evaluate cubic bezier at t for one axis (either x or y)
fn cubic_bezier(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    // B(t) = (1-t)³·P0 + 3(1-t)²t·P1 + 3(1-t)t²·P2 + t³·P3
    // With P0 = 0 and P3 = 1:
    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

/// Derivative of cubic bezier at t for one axis
fn cubic_bezier_derivative(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    // B'(t) = 3(1-t)²·(P1-P0) + 6(1-t)t·(P2-P1) + 3t²·(P3-P2)
    // With P0 = 0 and P3 = 1:
    3.0 * mt2 * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t2 * (1.0 - p2)
}

/// Animation state for a single animated value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Animation {
    /// Starting value
    pub from: f32,
    /// Target value
    pub to: f32,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Elapsed time in milliseconds
    pub elapsed_ms: u64,
    /// Easing function
    pub easing: EasingFunction,
}

#[allow(dead_code)]
impl Animation {
    /// Create a new animation.
    pub fn new(from: f32, to: f32, duration_ms: u64, easing: EasingFunction) -> Self {
        Self {
            from,
            to,
            duration_ms,
            elapsed_ms: 0,
            easing,
        }
    }

    /// Update the animation with elapsed time.
    ///
    /// Returns the current animated value.
    pub fn update(&mut self, delta_ms: u64) -> f32 {
        self.elapsed_ms = (self.elapsed_ms + delta_ms).min(self.duration_ms);
        self.current_value()
    }

    /// Get the current animated value without updating.
    pub fn current_value(&self) -> f32 {
        if self.duration_ms == 0 {
            return self.to;
        }

        let progress = self.elapsed_ms as f32 / self.duration_ms as f32;
        let eased = self.easing.ease(progress);
        self.from + (self.to - self.from) * eased
    }

    /// Check if the animation is complete.
    pub fn is_complete(&self) -> bool {
        self.elapsed_ms >= self.duration_ms
    }

    /// Reset the animation to the beginning.
    pub fn reset(&mut self) {
        self.elapsed_ms = 0;
    }

    /// Reverse the animation direction.
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
        self.elapsed_ms = self.duration_ms.saturating_sub(self.elapsed_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_easing() {
        let easing = EasingFunction::Linear;
        assert!((easing.ease(0.0) - 0.0).abs() < 0.001);
        assert!((easing.ease(0.5) - 0.5).abs() < 0.001);
        assert!((easing.ease(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ease_in_starts_slow() {
        let easing = EasingFunction::EaseIn;
        // At 50% progress, should be less than 50% of the way
        assert!(easing.ease(0.5) < 0.5);
    }

    #[test]
    fn test_ease_out_ends_slow() {
        let easing = EasingFunction::EaseOut;
        // At 50% progress, should be more than 50% of the way
        assert!(easing.ease(0.5) > 0.5);
    }

    #[test]
    fn test_animation_progress() {
        let mut anim = Animation::new(0.0, 100.0, 1000, EasingFunction::Linear);

        assert!((anim.current_value() - 0.0).abs() < 0.001);

        anim.update(500);
        assert!((anim.current_value() - 50.0).abs() < 0.001);

        anim.update(500);
        assert!((anim.current_value() - 100.0).abs() < 0.001);
        assert!(anim.is_complete());
    }

    #[test]
    fn test_animation_reverse() {
        let mut anim = Animation::new(0.0, 100.0, 1000, EasingFunction::Linear);
        anim.update(250);
        assert!((anim.current_value() - 25.0).abs() < 0.001);

        anim.reverse();
        assert!((anim.current_value() - 25.0).abs() < 0.001);

        anim.update(250);
        // After reverse, we continue from where we were
        assert!((anim.current_value() - 0.0).abs() < 0.001);
    }
}
