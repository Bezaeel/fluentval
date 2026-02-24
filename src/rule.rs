use crate::error::ValidationError;
use crate::traits::{Numeric, OptionLike};

/// Rule function type that validates a value and returns an optional error message
pub type Rule<T> = Box<dyn Fn(&T) -> Option<String>>;

/// Builder for creating validation rules in a fluent style
pub struct RuleBuilder<T> {
    property_name: String,
    rules: Vec<Rule<T>>,
}

impl<T> RuleBuilder<T> {
    /// Create a new rule builder for a property
    pub fn for_property(property_name: impl Into<String>) -> Self {
        Self {
            property_name: property_name.into(),
            rules: Vec::new(),
        }
    }

    /// Add a custom rule
    pub fn rule(mut self, rule: impl Fn(&T) -> Option<String> + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    /// Validate that the value is not empty (for strings)
    /// 
    /// # Arguments
    /// * `message` - Optional custom error message. If not provided, uses default message.
    pub fn not_empty(self, message: Option<impl Into<String>>) -> Self
    where
        T: AsRef<str>,
    {
        let msg = message.map(|m| m.into()).unwrap_or_else(|| "must not be empty".to_string());
        self.rule(move |value| {
            if value.as_ref().trim().is_empty() {
                Some(msg.clone())
            } else {
                None
            }
        })
    }

    /// Validate that the value is not null/empty (for Option types)
    /// 
    /// # Arguments
    /// * `message` - Optional custom error message. If not provided, uses default message.
    pub fn not_null(self, message: Option<impl Into<String>>) -> Self
    where
        T: OptionLike,
    {
        let msg = message.map(|m| m.into()).unwrap_or_else(|| "must not be null".to_string());
        self.rule(move |value| {
            if value.is_none() {
                Some(msg.clone())
            } else {
                None
            }
        })
    }

    /// Validate minimum length
    /// 
    /// # Arguments
    /// * `min` - Minimum length required
    /// * `message` - Optional custom error message. If not provided, uses default message with the min value.
    pub fn min_length(self, min: usize, message: Option<impl Into<String> + Clone + 'static>) -> Self
    where
        T: AsRef<str>,
    {
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            let len = value.as_ref().len();
            if len < min {
                Some(msg.clone().unwrap_or_else(|| format!("must be at least {} characters long", min)))
            } else {
                None
            }
        })
    }

    /// Validate maximum length
    /// 
    /// # Arguments
    /// * `max` - Maximum length allowed
    /// * `message` - Optional custom error message. If not provided, uses default message with the max value.
    pub fn max_length(self, max: usize, message: Option<impl Into<String> + Clone + 'static>) -> Self
    where
        T: AsRef<str>,
    {
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            let len = value.as_ref().len();
            if len > max {
                Some(msg.clone().unwrap_or_else(|| format!("must be at most {} characters long", max)))
            } else {
                None
            }
        })
    }

    /// Validate length range
    /// 
    /// # Arguments
    /// * `min` - Minimum length required
    /// * `max` - Maximum length allowed
    /// * `min_message` - Optional custom error message for minimum length violation
    /// * `max_message` - Optional custom error message for maximum length violation
    pub fn length(self, min: usize, max: usize, min_message: Option<impl Into<String> + Clone + 'static>, max_message: Option<impl Into<String> + Clone + 'static>) -> Self
    where
        T: AsRef<str>,
    {
        self.min_length(min, min_message).max_length(max, max_message)
    }

    /// Validate email format
    /// 
    /// # Arguments
    /// * `message` - Optional custom error message. If not provided, uses default message.
    pub fn email(self, message: Option<impl Into<String>>) -> Self
    where
        T: AsRef<str>,
    {
        let msg = message.map(|m| m.into()).unwrap_or_else(|| "must be a valid email address".to_string());
        self.rule(move |value| {
            let email_regex = regex::Regex::new(
                r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
            )
            .unwrap();
            if !email_regex.is_match(value.as_ref()) {
                Some(msg.clone())
            } else {
                None
            }
        })
    }

    /// Validate that value is greater than a minimum
    /// 
    /// # Arguments
    /// * `min` - Minimum value (exclusive)
    /// * `message` - Optional custom error message. If not provided, uses default message with the min value.
    pub fn greater_than(self, min: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self
    where
        T: Numeric,
    {
        let min_val = min.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.to_f64() <= min_val {
                Some(msg.clone().unwrap_or_else(|| format!("must be greater than {}", min_val)))
            } else {
                None
            }
        })
    }

    /// Validate that value is greater than or equal to a minimum
    /// 
    /// # Arguments
    /// * `min` - Minimum value (inclusive)
    /// * `message` - Optional custom error message. If not provided, uses default message with the min value.
    pub fn greater_than_or_equal(self, min: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self
    where
        T: Numeric,
    {
        let min_val = min.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.to_f64() < min_val {
                Some(msg.clone().unwrap_or_else(|| format!("must be greater than or equal to {}", min_val)))
            } else {
                None
            }
        })
    }

    /// Validate that value is less than a maximum
    /// 
    /// # Arguments
    /// * `max` - Maximum value (exclusive)
    /// * `message` - Optional custom error message. If not provided, uses default message with the max value.
    pub fn less_than(self, max: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self
    where
        T: Numeric,
    {
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.to_f64() >= max_val {
                Some(msg.clone().unwrap_or_else(|| format!("must be less than {}", max_val)))
            } else {
                None
            }
        })
    }

    /// Validate that value is less than or equal to a maximum
    /// 
    /// # Arguments
    /// * `max` - Maximum value (inclusive)
    /// * `message` - Optional custom error message. If not provided, uses default message with the max value.
    pub fn less_than_or_equal(self, max: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self
    where
        T: Numeric,
    {
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.to_f64() > max_val {
                Some(msg.clone().unwrap_or_else(|| format!("must be less than or equal to {}", max_val)))
            } else {
                None
            }
        })
    }

    /// Validate that value is within a range (inclusive)
    /// 
    /// # Arguments
    /// * `min` - Minimum value (inclusive)
    /// * `max` - Maximum value (inclusive)
    /// * `message` - Optional custom error message. If not provided, uses default message with the min and max values.
    pub fn inclusive_between(self, min: impl Into<f64> + Copy + 'static, max: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self
    where
        T: Numeric,
    {
        let min_val = min.into();
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            let val = value.to_f64();
            if val < min_val || val > max_val {
                Some(msg.clone().unwrap_or_else(|| format!("must be between {} and {}", min_val, max_val)))
            } else {
                None
            }
        })
    }

    /// Validate with a custom predicate
    pub fn must(self, predicate: impl Fn(&T) -> bool + 'static, message: impl Into<String> + Clone + 'static) -> Self {
        let msg = message.into();
        self.rule(move |value| {
            if !predicate(value) {
                Some(msg.clone())
            } else {
                None
            }
        })
    }

    /// Build the rule and return a function that can be used in a validator
    pub fn build(self) -> impl Fn(&T) -> Vec<ValidationError> {
        let property_name = self.property_name.clone();
        let rules = self.rules;
        move |value: &T| {
            let mut errors = Vec::new();
            for rule in &rules {
                if let Some(message) = rule(value) {
                    errors.push(ValidationError::new(property_name.clone(), message));
                }
            }
            errors
        }
    }
}

