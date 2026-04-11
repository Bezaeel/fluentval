# FluentVal <img src="https://github.com/Bezaeel/fluentval/actions/workflows/publish.yml/badge.svg" alt="actions status" />

A fluent validation library for Rust. FluentVal provides an intuitive, chainable API for validating data structures with comprehensive error reporting.

## Features

- 🎯 **Fluent API** - Chain validation rules in a readable, expressive way
- 📝 **Comprehensive Rules** - Built-in validators for strings, numbers, emails, and more
- 🔧 **Custom Rules** - Define your own validation logic
- 🔗 **Cross-Property Validation** - Validate a field against sibling fields inline
- 📊 **Rich Error Reporting** - Detailed validation errors grouped by property
- 🚀 **Type-Safe** - Leverages Rust's type system for compile-time safety

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluentval = "1.0.0"
```

## Quick Start

```rust
use fluentval::*;

#[derive(Debug)]
struct User {
    name: String,
    email: String,
    age: i32,
}

let v = FluentValidator::<User>::new();
rule_for!(v, u.name)
    .not_empty(None::<String>)
    .min_length(2, None::<String>)
    .max_length(50, None::<String>);
rule_for!(v, u.email)
    .not_empty(None::<String>)
    .email(None::<String>);
rule_for!(v, u.age)
    .greater_than_or_equal(18, None::<String>);
let validator = v.build();

let user = User {
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    age: 25,
};

let result = validator.validate(&user);
if result.is_valid() {
    println!("User is valid!");
} else {
    for error in result.errors() {
        println!("{}: {}", error.property, error.message);
    }
}
```

## Custom Error Messages

All rules accept an optional custom error message:

```rust
use fluentval::*;

#[derive(Debug)]
struct User {
    name: String,
    email: String,
    age: i32,
    password: String,
}

let v = FluentValidator::<User>::new();
rule_for!(v, u.name)
    .not_empty(Some("Name is required"))
    .min_length(2, Some("Name must be at least 2 characters long"))
    .max_length(50, Some("Name cannot exceed 50 characters"));
rule_for!(v, u.email)
    .not_empty(Some("Email address is required"))
    .email(Some("Please provide a valid email address"));
rule_for!(v, u.age)
    .greater_than_or_equal(18, Some("You must be at least 18 years old"))
    .less_than_or_equal(120, Some("Age must be realistic"));
rule_for!(v, u.password)
    .not_empty(Some("Password is required"))
    .min_length(8, Some("Password must be at least 8 characters"))
    .must(|_, p| p.chars().any(|c| c.is_ascii_uppercase()),
          "Password must contain at least one uppercase letter")
    .must(|_, p| p.chars().any(|c| c.is_ascii_digit()),
          "Password must contain at least one number");
let validator = v.build();

let invalid_user = User {
    name: "A".to_string(),
    email: "invalid-email".to_string(),
    age: 15,
    password: "weak".to_string(),
};

let result = validator.validate(&invalid_user);
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

## Cross-Property Validation

The `must` method on a property rule receives the full object (`&T`) and the property value (`&V`), enabling validation that depends on sibling fields:

```rust
use fluentval::*;

#[derive(Debug)]
struct Command {
    country_iso_code: String,
    phone_number: String,
    alt_phone_number: String,
    tax_number: String,
}

fn is_valid_phone_for_country(phone: &str, country_code: &str) -> bool {
    match country_code {
        "US" => phone.len() == 10 && phone.chars().all(|c| c.is_ascii_digit()),
        "UK" => phone.len() == 11 && phone.starts_with('0'),
        _ => phone.len() >= 8 && phone.len() <= 15,
    }
}

fn is_valid_tax_number(tax_number: &str, country_code: &str) -> bool {
    match country_code {
        "US" => tax_number.len() == 9 && tax_number.chars().all(|c| c.is_ascii_digit()),
        "UK" => tax_number.len() == 10 && tax_number.starts_with("GB"),
        _ => tax_number.len() >= 8 && tax_number.len() <= 15,
    }
}

let v = FluentValidator::<Command>::new();
rule_for!(v, c.phone_number)
    .not_empty(None::<String>)
    .must(|cmd, phone| is_valid_phone_for_country(phone, &cmd.country_iso_code),
          "Phone number is not valid for the specified country");
rule_for!(v, c.alt_phone_number)
    .not_empty(None::<String>)
    .must(|cmd, alt| alt != &cmd.phone_number,
          "Alternative phone number must be different from primary phone number");
rule_for!(v, c.tax_number)
    .not_empty(None::<String>)
    .must(|cmd, tax| is_valid_tax_number(tax, &cmd.country_iso_code),
          "Tax number is not valid for the specified country");
let validator = v.build();
```

## Available Rules

### String Rules

- `not_empty(msg)` - Validates that a string is not empty or whitespace
- `min_length(min, msg)` - Validates minimum string length
- `max_length(max, msg)` - Validates maximum string length
- `length(min, max, min_msg, max_msg)` - Validates string length range
- `email(msg)` - Validates email format

### Numeric Rules

- `greater_than(min, msg)` - Value must be greater than minimum
- `greater_than_or_equal(min, msg)` - Value must be greater than or equal to minimum
- `less_than(max, msg)` - Value must be less than maximum
- `less_than_or_equal(max, msg)` - Value must be less than or equal to maximum
- `inclusive_between(min, max, msg)` - Value must be within range (inclusive)

### Option Rules

- `not_null(msg)` - Validates that an `Option` is `Some`

### Custom Rules

- `rule(predicate)` - Add a custom rule with predicate `Fn(&V) -> Option<String>`
- `must(predicate, message)` - Predicate is `Fn(&T, &V) -> bool`; receives the full object for cross-property access

## Advanced Usage

### Adding Rules Across Multiple Statements

The same property can have rules added across multiple `rule_for!` calls — they all accumulate:

```rust
use fluentval::*;

let v = FluentValidator::<User>::new();
rule_for!(v, u.name)
    .not_empty(None::<String>)
    .min_length(2, None::<String>);

// Rules added in a second statement are merged with the first
rule_for!(v, u.name)
    .max_length(50, None::<String>);

let validator = v.build();
```

### Custom Property Labels

If you need a display label that differs from the field name, call `rule_for` directly instead of using the macro:

```rust
let v = FluentValidator::<User>::new();
v.rule_for("taxId", |u| &u.tax_id)
    .not_empty(None::<String>);
let validator = v.build();
```

### Nested Field Paths

```rust
use fluentval::*;

#[derive(Debug)]
struct Address { city: String }
#[derive(Debug)]
struct Person { address: Address }

let v = FluentValidator::<Person>::new();
rule_for!(v, p.address.city).not_empty(None::<String>);
// Error property will be "address.city"
let validator = v.build();
```

### Working with Validation Results

```rust
let result = validator.validate(&user);
// or via the free function:
let result = validate(&user, &validator);

if result.is_valid() { /* ... */ }

for error in result.errors() {
    println!("{}: {}", error.property, error.message);
}

// Errors grouped by property
let errors_by_prop = result.errors_by_property();

// First error for a specific property
if let Some(message) = result.first_error_for("email") {
    println!("Email error: {}", message);
}
```

## Migration from v0.x

| v0.x | v1.0 |
|---|---|
| `ValidatorBuilder::<T>::new()` | `FluentValidator::<T>::new()` |
| `.rule_for("name", \|c\| &c.name, RuleBuilder::for_property("name").not_empty(...))` | `rule_for!(v, c.name).not_empty(...)` |
| `.must("prop", \|c\| &c.prop, \|obj, val\| ..., "msg")` on `ValidatorBuilder` | `.must(\|obj, val\| ..., "msg")` on the `rule_for!` chain |

The v0.x `ValidatorBuilder` and `RuleBuilder` types remain available for backward compatibility.

## License

MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
