use std::collections::HashMap;
use std::fmt::Display;

/// Represents a validation error with a property name and error message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub property: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(property: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            property: property.into(),
            message: message.into(),
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.property, self.message)
    }
}

/// Result of validation containing errors if validation failed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create a new empty validation result
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add a validation error
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add multiple validation errors
    pub fn add_errors(&mut self, errors: Vec<ValidationError>) {
        self.errors.extend(errors);
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get all validation errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Get errors grouped by property name
    pub fn errors_by_property(&self) -> HashMap<String, Vec<String>> {
        let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
        for error in &self.errors {
            grouped
                .entry(error.property.clone())
                .or_insert_with(Vec::new)
                .push(error.message.clone());
        }
        grouped
    }

    /// Get the first error message for a property, if any
    pub fn first_error_for(&self, property: &str) -> Option<&str> {
        self.errors
            .iter()
            .find(|e| e.property == property)
            .map(|e| e.message.as_str())
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for defining validators
pub trait Validator<T> {
    fn validate(&self, instance: &T) -> ValidationResult;
}

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

/// Trait for types that can be treated as numeric values
pub trait Numeric {
    fn to_f64(&self) -> f64;
}

impl Numeric for i8 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i16 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i32 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i64 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u8 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u16 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u32 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u64 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for f32 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for f64 { fn to_f64(&self) -> f64 { *self } }

/// Trait for types that can be treated as Option-like
pub trait OptionLike {
    fn is_none(&self) -> bool;
}

impl<T> OptionLike for Option<T> {
    fn is_none(&self) -> bool {
        Option::is_none(self)
    }
}

/// Helper struct to build validators in a fluent style
pub struct ValidatorBuilder<T> {
    rules: Vec<Box<dyn Fn(&T) -> Vec<ValidationError>>>,
}

impl<T> ValidatorBuilder<T> {
    /// Create a new validator builder
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule for a property
    pub fn rule_for<F, V>(mut self, _property_name: impl Into<String>, accessor: F, builder: RuleBuilder<V>) -> Self
    where
        F: Fn(&T) -> &V + 'static,
        V: 'static,
    {
        let rule_fn = builder.build();
        self.rules.push(Box::new(move |instance: &T| {
            let value = accessor(instance);
            rule_fn(value)
        }));
        self
    }

    /// Add a rule for a property that can access the entire object
    /// 
    /// This allows you to validate a property based on other properties in the object.
    /// The closure receives both the object and the property value.
    /// 
    /// # Arguments
    /// * `property_name` - Name of the property being validated
    /// * `accessor` - Function to access the property value from the object
    /// * `predicate` - Function that receives both the entire object and the property value, returns true if valid
    /// * `message` - Error message to use if validation fails
    /// 
    /// # Example
    /// ```rust,ignore
    /// // Validate property using both object and property value
    /// .must("taxNumber", |c| &c.tax_number,
    ///     |command, tax_number| tax_number.is_valid_tax_number(&command.country_iso_code),
    ///     "Tax number is not valid for the specified country")
    /// 
    /// // Validate property ignoring the object (use _ for object parameter)
    /// .must("country", |c| &c.country,
    ///     |_, country| Countries::allowed_countries().contains(country),
    ///     "Country is not in the allowed list")
    /// ```
    pub fn must<F, V, P>(mut self, property_name: impl Into<String>, accessor: F, predicate: P, message: impl Into<String>) -> Self
    where
        F: Fn(&T) -> &V + 'static,
        V: 'static,
        P: Fn(&T, &V) -> bool + 'static,
    {
        let property_name = property_name.into();
        let msg = message.into();
        self.rules.push(Box::new(move |instance: &T| {
            let value = accessor(instance);
            if !predicate(instance, value) {
                vec![ValidationError::new(property_name.clone(), msg.clone())]
            } else {
                Vec::new()
            }
        }));
        self
    }

    /// Build the validator
    pub fn build(self) -> impl Validator<T> {
        ValidatorImpl { rules: self.rules }
    }
}

impl<T> Default for ValidatorBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

struct ValidatorImpl<T> {
    rules: Vec<Box<dyn Fn(&T) -> Vec<ValidationError>>>,
}

impl<T> Validator<T> for ValidatorImpl<T> {
    fn validate(&self, instance: &T) -> ValidationResult {
        let mut result = ValidationResult::new();
        for rule in &self.rules {
            let errors = rule(instance);
            result.add_errors(errors);
        }
        result
    }
}

/// Helper function to validate an instance with a validator
pub fn validate<T>(instance: &T, validator: &dyn Validator<T>) -> ValidationResult {
    validator.validate(instance)
}


