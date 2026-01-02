//! Animation configuration.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Re-export EasingFunction from tui-widgets (conceptually).
/// In the actual implementation, this would be imported from tui-widgets.
/// For now, we define a compatible type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EasingFunction {
    /// Linear interpolation
    Linear,
    /// Slow start
    EaseIn,
    /// Slow end
    EaseOut,
    /// Slow start and end
    EaseInOut,
    /// Bounce effect
    Bounce,
    /// Elastic spring effect
    Elastic,
    /// Custom cubic bezier (x1, y1, x2, y2)
    #[serde(rename = "cubic-bezier")]
    CubicBezier(f32, f32, f32, f32),
}

impl Default for EasingFunction {
    fn default() -> Self {
        Self::EaseInOut
    }
}

/// Animation timing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Fast duration (e.g., 100ms)
    #[serde(with = "duration_ms")]
    pub duration_fast: Duration,
    /// Normal duration (e.g., 200ms)
    #[serde(with = "duration_ms")]
    pub duration_normal: Duration,
    /// Slow duration (e.g., 400ms)
    #[serde(with = "duration_ms")]
    pub duration_slow: Duration,
    /// Default easing function
    pub easing: EasingFunction,
    /// Whether animations are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Speed multiplier (1.0 = normal, 2.0 = twice as fast)
    #[serde(default = "default_one")]
    pub speed_multiplier: f32,
}

fn default_true() -> bool {
    true
}

fn default_one() -> f32 {
    1.0
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration_fast: Duration::from_millis(100),
            duration_normal: Duration::from_millis(200),
            duration_slow: Duration::from_millis(400),
            easing: EasingFunction::default(),
            enabled: true,
            speed_multiplier: 1.0,
        }
    }
}

impl AnimationConfig {
    /// Get the effective fast duration (with multiplier applied).
    pub fn fast(&self) -> Duration {
        self.apply_multiplier(self.duration_fast)
    }

    /// Get the effective normal duration (with multiplier applied).
    pub fn normal(&self) -> Duration {
        self.apply_multiplier(self.duration_normal)
    }

    /// Get the effective slow duration (with multiplier applied).
    pub fn slow(&self) -> Duration {
        self.apply_multiplier(self.duration_slow)
    }

    fn apply_multiplier(&self, duration: Duration) -> Duration {
        if self.speed_multiplier == 1.0 {
            duration
        } else {
            Duration::from_secs_f64(duration.as_secs_f64() / self.speed_multiplier as f64)
        }
    }

    /// Create a config with animations disabled.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create a config with reduced motion (slower, simpler animations).
    pub fn reduced_motion() -> Self {
        Self {
            duration_fast: Duration::from_millis(200),
            duration_normal: Duration::from_millis(400),
            duration_slow: Duration::from_millis(800),
            easing: EasingFunction::Linear,
            enabled: true,
            speed_multiplier: 0.5,
        }
    }
}

/// Serde helper for Duration as milliseconds.
mod duration_ms {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ms = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnimationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.duration_fast, Duration::from_millis(100));
        assert_eq!(config.duration_normal, Duration::from_millis(200));
        assert_eq!(config.duration_slow, Duration::from_millis(400));
    }

    #[test]
    fn test_speed_multiplier() {
        let mut config = AnimationConfig::default();
        config.speed_multiplier = 2.0;

        assert_eq!(config.fast(), Duration::from_millis(50));
        assert_eq!(config.normal(), Duration::from_millis(100));
        assert_eq!(config.slow(), Duration::from_millis(200));
    }

    #[test]
    fn test_disabled_config() {
        let config = AnimationConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_reduced_motion() {
        let config = AnimationConfig::reduced_motion();
        assert!(config.enabled);
        assert!(config.duration_fast > AnimationConfig::default().duration_fast);
    }
}
