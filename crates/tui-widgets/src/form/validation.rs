//! Validation for form fields.

use super::{FormData, Value};
use regex::Regex;

/// Validator for form fields.
#[derive(Clone)]
pub enum Validator {
    /// Field is required
    Required,
    /// Minimum length for strings
    MinLength(usize),
    /// Maximum length for strings
    MaxLength(usize),
    /// Regular expression pattern
    Regex(Regex),
    /// Minimum numeric value
    Min(f64),
    /// Maximum numeric value
    Max(f64),
    /// Custom validation function
    Custom(fn(&Value) -> Result<(), String>),
    /// Cross-field validation (has access to all form data)
    CrossField(fn(&Value, &FormData) -> Result<(), String>),
}

impl std::fmt::Debug for Validator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "Required"),
            Self::MinLength(n) => write!(f, "MinLength({})", n),
            Self::MaxLength(n) => write!(f, "MaxLength({})", n),
            Self::Regex(r) => write!(f, "Regex({})", r.as_str()),
            Self::Min(n) => write!(f, "Min({})", n),
            Self::Max(n) => write!(f, "Max({})", n),
            Self::Custom(_) => write!(f, "Custom(fn)"),
            Self::CrossField(_) => write!(f, "CrossField(fn)"),
        }
    }
}

impl Validator {
    /// Create a regex validator from a pattern string.
    pub fn pattern(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self::Regex(Regex::new(pattern)?))
    }

    /// Create a custom validator.
    pub fn custom(f: fn(&Value) -> Result<(), String>) -> Self {
        Self::Custom(f)
    }

    /// Create a cross-field validator.
    pub fn cross_field(f: fn(&Value, &FormData) -> Result<(), String>) -> Self {
        Self::CrossField(f)
    }

    /// Validate a value.
    pub fn validate(&self, value: &Value, form_data: &FormData) -> Result<(), String> {
        match self {
            Self::Required => {
                if value.is_empty() {
                    Err("This field is required".into())
                } else {
                    Ok(())
                }
            }
            Self::MinLength(min) => {
                let len = match value {
                    Value::String(s) => s.len(),
                    Value::List(l) => l.len(),
                    _ => 0,
                };
                if len < *min {
                    Err(format!("Must be at least {} characters", min))
                } else {
                    Ok(())
                }
            }
            Self::MaxLength(max) => {
                let len = match value {
                    Value::String(s) => s.len(),
                    Value::List(l) => l.len(),
                    _ => 0,
                };
                if len > *max {
                    Err(format!("Must be at most {} characters", max))
                } else {
                    Ok(())
                }
            }
            Self::Regex(regex) => {
                if let Value::String(s) = value {
                    if regex.is_match(s) {
                        Ok(())
                    } else {
                        Err("Invalid format".into())
                    }
                } else {
                    Ok(()) // Non-string values pass regex validation
                }
            }
            Self::Min(min) => {
                if let Value::Number(n) = value {
                    if *n < *min {
                        Err(format!("Must be at least {}", min))
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
            Self::Max(max) => {
                if let Value::Number(n) = value {
                    if *n > *max {
                        Err(format!("Must be at most {}", max))
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
            Self::Custom(f) => f(value),
            Self::CrossField(f) => f(value, form_data),
        }
    }
}

/// Common validators for convenience.
impl Validator {
    /// Email validator.
    pub fn email() -> Self {
        // Simple email regex - not exhaustive but catches most cases
        Self::Regex(
            Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
        )
    }

    /// URL validator.
    pub fn url() -> Self {
        Self::Regex(
            Regex::new(r"^https?://[^\s]+$").unwrap()
        )
    }

    /// Phone number validator (simple).
    pub fn phone() -> Self {
        Self::Regex(
            Regex::new(r"^[+]?[\d\s\-()]+$").unwrap()
        )
    }

    /// Alphanumeric validator.
    pub fn alphanumeric() -> Self {
        Self::Regex(
            Regex::new(r"^[a-zA-Z0-9]+$").unwrap()
        )
    }

    /// Integer validator.
    pub fn integer() -> Self {
        Self::custom(|v| {
            if let Value::Number(n) = v {
                if n.fract() == 0.0 {
                    Ok(())
                } else {
                    Err("Must be a whole number".into())
                }
            } else if let Value::String(s) = v {
                if s.parse::<i64>().is_ok() {
                    Ok(())
                } else {
                    Err("Must be a whole number".into())
                }
            } else {
                Ok(())
            }
        })
    }

    /// Positive number validator.
    pub fn positive() -> Self {
        Self::Min(0.0)
    }

    /// Password confirmation validator (checks against another field).
    pub fn matches_field(field_name: &'static str) -> Self {
        Self::cross_field(move |value, form_data| {
            let other = form_data.get(field_name);
            if Some(value) == other {
                Ok(())
            } else {
                Err(format!("Must match {}", field_name))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn empty_form_data() -> FormData {
        HashMap::new()
    }

    #[test]
    fn test_required() {
        let v = Validator::Required;
        let data = empty_form_data();

        assert!(v.validate(&Value::None, &data).is_err());
        assert!(v.validate(&Value::String("".into()), &data).is_err());
        assert!(v.validate(&Value::String("hello".into()), &data).is_ok());
    }

    #[test]
    fn test_min_length() {
        let v = Validator::MinLength(3);
        let data = empty_form_data();

        assert!(v.validate(&Value::String("ab".into()), &data).is_err());
        assert!(v.validate(&Value::String("abc".into()), &data).is_ok());
        assert!(v.validate(&Value::String("abcd".into()), &data).is_ok());
    }

    #[test]
    fn test_max_length() {
        let v = Validator::MaxLength(5);
        let data = empty_form_data();

        assert!(v.validate(&Value::String("abc".into()), &data).is_ok());
        assert!(v.validate(&Value::String("abcde".into()), &data).is_ok());
        assert!(v.validate(&Value::String("abcdef".into()), &data).is_err());
    }

    #[test]
    fn test_min_max_number() {
        let data = empty_form_data();

        let min = Validator::Min(10.0);
        assert!(min.validate(&Value::Number(5.0), &data).is_err());
        assert!(min.validate(&Value::Number(10.0), &data).is_ok());
        assert!(min.validate(&Value::Number(15.0), &data).is_ok());

        let max = Validator::Max(10.0);
        assert!(max.validate(&Value::Number(5.0), &data).is_ok());
        assert!(max.validate(&Value::Number(10.0), &data).is_ok());
        assert!(max.validate(&Value::Number(15.0), &data).is_err());
    }

    #[test]
    fn test_email() {
        let v = Validator::email();
        let data = empty_form_data();

        assert!(v.validate(&Value::String("test@example.com".into()), &data).is_ok());
        assert!(v.validate(&Value::String("invalid".into()), &data).is_err());
        assert!(v.validate(&Value::String("@example.com".into()), &data).is_err());
    }

    #[test]
    fn test_cross_field() {
        let v = Validator::matches_field("password");

        let mut data: FormData = HashMap::new();
        data.insert("password".into(), Value::String("secret123".into()));

        assert!(v.validate(&Value::String("secret123".into()), &data).is_ok());
        assert!(v.validate(&Value::String("different".into()), &data).is_err());
    }

    #[test]
    fn test_custom() {
        let v = Validator::custom(|v| {
            if let Value::String(s) = v {
                if s.starts_with("hello") {
                    Ok(())
                } else {
                    Err("Must start with 'hello'".into())
                }
            } else {
                Ok(())
            }
        });

        let data = empty_form_data();
        assert!(v.validate(&Value::String("hello world".into()), &data).is_ok());
        assert!(v.validate(&Value::String("goodbye".into()), &data).is_err());
    }
}
