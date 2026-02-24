//! FluentVal - A fluent validation library for Rust
//!
//! This library provides a builder pattern for creating validators in a readable,
//! chainable style, inspired by FluentValidation in .NET.
//!
//! # Example
//! ```rust,ignore
//! use fluentval::{ValidatorBuilder, RuleBuilder, Validator};
//!
//! struct User {
//!     name: String,
//!     email: String,
//!     age: i32,
//! }
//!
//! let validator = ValidatorBuilder::<User>::new()
//!     .rule_for("name", |u| &u.name,
//!         RuleBuilder::for_property("name")
//!             .not_empty(None)
//!             .min_length(2, None))
//!     .rule_for("email", |u| &u.email,
//!         RuleBuilder::for_property("email")
//!             .email(None))
//!     .rule_for("age", |u| &u.age,
//!         RuleBuilder::for_property("age")
//!             .greater_than_or_equal(18, Some("Must be 18 or older")))
//!     .build();
//!
//! let user = User { name: "".into(), email: "invalid".into(), age: 15 };
//! let result = validator.validate(&user);
//!
//! if !result.is_valid() {
//!     for error in result.errors() {
//!         println!("{}: {}", error.property, error.message);
//!     }
//! }
//! ```

mod builder;
mod error;
mod rule;
mod traits;

// Re-export all public types
pub use builder::{validate, ValidatorBuilder};
pub use error::{ValidationError, ValidationResult};
pub use rule::{Rule, RuleBuilder};
pub use traits::{Numeric, OptionLike, Validator};
