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
                .or_default()
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

