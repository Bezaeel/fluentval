# FluentVal <img src="https://github.com/Bezaeel/fluentval/actions/workflows/publish.yml/badge.svg" alt="actions status" />

A fluent validation library for Rust with a builder pattern API. FluentVal provides an intuitive, chainable API for validating data structures with comprehensive error reporting.

## Features

- 🎯 **Fluent Builder API** - Chain validation rules in a readable, expressive way
- 📝 **Comprehensive Rules** - Built-in validators for strings, numbers, emails, and more
- 🔧 **Custom Rules** - Define your own validation logic
- 📊 **Rich Error Reporting** - Detailed validation errors grouped by property
- 🚀 **Type-Safe** - Leverages Rust's type system for compile-time safety

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluentval = "0.1.0"
```

## Quick Start

### Basic String Validation

```rust
use fluentval::*;

let rule_fn = RuleBuilder::<String>::for_property("name")
    .not_empty(None::<String>)
    .min_length(3, None::<String>)
    .max_length(50, None::<String>)
    .build();

let errors = rule_fn(&"ab".to_string());
// Returns validation errors if validation fails
```

### Validating Complex Objects

```rust
use fluentval::*;

#[derive(Debug)]
struct User {
    name: String,
    email: String,
    age: i32,
}

let validator = ValidatorBuilder::<User>::new()
    .rule_for("name", |u| &u.name,
        RuleBuilder::for_property("name")
            .not_empty(None::<String>)
            .min_length(2, None::<String>))
    .rule_for("email", |u| &u.email,
        RuleBuilder::for_property("email")
            .not_empty(None::<String>)
            .email(None::<String>))
    .rule_for("age", |u| &u.age,
        RuleBuilder::for_property("age")
            .greater_than_or_equal(18, None::<String>))
    .build();

let user = User {
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    age: 25,
};

let result = validate(&user, &validator);
if result.is_valid() {
    println!("User is valid!");
} else {
    for error in result.errors() {
        println!("{}: {}", error.property, error.message);
    }
}
```

### Validating with Custom Error Messages

You can specify custom error messages for each rule to provide more meaningful feedback:

```rust
use fluentval::*;

#[derive(Debug)]
struct User {
    name: String,
    email: String,
    age: i32,
    password: String,
}

let validator = ValidatorBuilder::<User>::new()
    .rule_for("name", |u| &u.name,
        RuleBuilder::for_property("name")
            .not_empty(Some("Name is required"))
            .min_length(2, Some("Name must be at least 2 characters long"))
            .max_length(50, Some("Name cannot exceed 50 characters")))
    .rule_for("email", |u| &u.email,
        RuleBuilder::for_property("email")
            .not_empty(Some("Email address is required"))
            .email(Some("Please provide a valid email address")))
    .rule_for("age", |u| &u.age,
        RuleBuilder::for_property("age")
            .greater_than_or_equal(18, Some("You must be at least 18 years old"))
            .less_than_or_equal(120, Some("Age must be realistic")))
    .rule_for("password", |u| &u.password,
        RuleBuilder::for_property("password")
            .not_empty(Some("Password is required"))
            .min_length(8, Some("Password must be at least 8 characters"))
            .must(|p| p.chars().any(|c| c.is_ascii_uppercase()), 
                  "Password must contain at least one uppercase letter")
            .must(|p| p.chars().any(|c| c.is_ascii_digit()), 
                  "Password must contain at least one number"))
    .build();

let invalid_user = User {
    name: "A".to_string(),  // Too short
    email: "invalid-email".to_string(),  // Invalid format
    age: 15,  // Too young
    password: "weak".to_string(),  // Too short and missing requirements
};

let result = validate(&invalid_user, &validator);
if !result.is_valid() {
    for error in result.errors() {
        println!("{}: {}", error.property, error.message);
    }
    // Output:
    // name: Name must be at least 2 characters long
    // email: Please provide a valid email address
    // age: You must be at least 18 years old
    // password: Password must be at least 8 characters
    // password: Password must contain at least one uppercase letter
    // password: Password must contain at least one number
}
```

## Available Rules

### String Rules

- `not_empty()` - Validates that a string is not empty or whitespace
- `min_length(min)` - Validates minimum string length
- `max_length(max)` - Validates maximum string length
- `length(min, max)` - Validates string length range
- `email()` - Validates email format

### Numeric Rules

- `greater_than(min)` - Value must be greater than minimum
- `greater_than_or_equal(min)` - Value must be greater than or equal to minimum
- `less_than(max)` - Value must be less than maximum
- `less_than_or_equal(max)` - Value must be less than or equal to maximum
- `inclusive_between(min, max)` - Value must be within range (inclusive)

### Option Rules

- `not_null()` - Validates that an Option is Some

### Custom Rules

- `rule(predicate)` - Add a custom validation rule
- `must(predicate, message)` - Validate with a custom predicate

## Advanced Usage

### Cross-Property Validation

Validate a property based on other properties in the same struct. The `must()` method in `ValidatorBuilder` allows you to access both the entire object and the property value:

```rust
use fluentval::*;

#[derive(Debug)]
struct Command {
    country_iso_code: String,
    phone_number: String,
    alt_phone_number: String,
    tax_number: String,
}

// Helper function to validate phone number based on country
fn is_valid_phone_for_country(phone: &str, country_code: &str) -> bool {
    match country_code {
        "US" => phone.len() == 10 && phone.chars().all(|c| c.is_ascii_digit()),
        "UK" => phone.len() == 11 && phone.starts_with('0'),
        _ => phone.len() >= 8 && phone.len() <= 15,
    }
}

// Helper function to validate tax number based on country
fn is_valid_tax_number(tax_number: &str, country_code: &str) -> bool {
    match country_code {
        "US" => tax_number.len() == 9 && tax_number.chars().all(|c| c.is_ascii_digit()),
        "UK" => tax_number.len() == 10 && tax_number.starts_with("GB"),
        _ => tax_number.len() >= 8 && tax_number.len() <= 15,
    }
}

let validator = ValidatorBuilder::<Command>::new()
    // Validate phone number based on country
    .must("phoneNumber", |c| &c.phone_number,
        |command, phone| is_valid_phone_for_country(phone, &command.country_iso_code),
        "Phone number is not valid for the specified country")
    // Validate that alt phone is different from primary phone
    .must("altPhoneNumber", |c| &c.alt_phone_number,
        |command, alt_phone| alt_phone != &command.phone_number,
        "Alternative phone number must be different from primary phone number")
    // Validate tax number based on country
    .must("taxNumber", |c| &c.tax_number,
        |command, tax_number| is_valid_tax_number(tax_number, &command.country_iso_code),
        "Tax number is not valid for the specified country")
    .build();

// Example: Invalid phone number for US
let invalid_command = Command {
    country_iso_code: "US".to_string(),
    phone_number: "123".to_string(),  // Too short for US
    alt_phone_number: "9876543210".to_string(),
    tax_number: "123456789".to_string(),
};

let result = validate(&invalid_command, &validator);
if !result.is_valid() {
    for error in result.errors() {
        println!("{}: {}", error.property, error.message);
    }
    // Output: phoneNumber: Phone number is not valid for the specified country
}

// Example: Alt phone same as primary
let invalid_command2 = Command {
    country_iso_code: "US".to_string(),
    phone_number: "1234567890".to_string(),
    alt_phone_number: "1234567890".to_string(),  // Same as primary
    tax_number: "123456789".to_string(),
};

let result = validate(&invalid_command2, &validator);
if !result.is_valid() {
    for error in result.errors() {
        println!("{}: {}", error.property, error.message);
    }
    // Output: altPhoneNumber: Alternative phone number must be different from primary phone number
}
```

You can also validate a property without using the object context (ignore the object parameter with `_`):

```rust
#[derive(Debug)]
struct Registration {
    country: String,
    email: String,
}

// Simulate allowed countries
fn is_allowed_country(country: &str) -> bool {
    vec!["US", "UK", "CA", "AU"].contains(&country)
}

let validator = ValidatorBuilder::<Registration>::new()
    // Validate country without needing the object context
    .must("country", |r| &r.country,
        |_, country| is_allowed_country(country),
        "Country is not in the allowed list")
    .build();
```

### Custom Error Messages

All rules accept optional custom error messages:

```rust
RuleBuilder::<String>::for_property("email")
    .email(Some("Please provide a valid email address"))
    .min_length(5, Some("Email must be at least 5 characters"))
```

### Working with Validation Results

```rust
let result = validate(&user, &validator);

// Check if valid
if result.is_valid() {
    // Handle valid case
}

// Get all errors
for error in result.errors() {
    println!("{}: {}", error.property, error.message);
}

// Get errors grouped by property
let errors_by_prop = result.errors_by_property();

// Get first error for a specific property
if let Some(message) = result.first_error_for("email") {
    println!("Email error: {}", message);
}
```

## License

MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
