use crate::error::{ValidationError, ValidationResult};
use crate::rule::RuleBuilder;
use crate::traits::Validator;

type RuleFn<T> = Box<dyn Fn(&T) -> Vec<ValidationError>>;

/// Helper struct to build validators in a fluent style
pub struct ValidatorBuilder<T> {
    rules: Vec<RuleFn<T>>,
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
    rules: Vec<RuleFn<T>>,
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

