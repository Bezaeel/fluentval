//! FluentVal - A fluent validation library for Rust
//!
//! This library provides two APIs:
//!
//! **v0.x API** (`ValidatorBuilder` + `RuleBuilder`) — still supported.
//!
//! **v1.0 API** (`FluentValidator` + `rule_for!` macro) — simplified, eliminates property
//! name duplication and merges cross-property validation onto the property chain.
//!
//! # v1.0 Example
//! ```rust,ignore
//! use fluentval::{FluentValidator, rule_for, Validator, validate};
//!
//! struct User { name: String, email: String, age: i32 }
//!
//! let v = FluentValidator::<User>::new();
//! rule_for!(v, u.name).not_empty(None::<String>).min_length(2, None::<String>);
//! rule_for!(v, u.email).not_empty(None::<String>).email(None::<String>);
//! rule_for!(v, u.age).greater_than_or_equal(18, Some("Must be 18 or older"));
//! let validator = v.build();
//!
//! let user = User { name: "".into(), email: "invalid".into(), age: 15 };
//! let result = validator.validate(&user);
//! for error in result.errors() {
//!     println!("{}: {}", error.property, error.message);
//! }
//! ```

mod builder;
mod error;
mod fluent_validator;
mod rule;
mod traits;

// v0.x API
pub use builder::{validate, ValidatorBuilder};
pub use error::{ValidationError, ValidationResult};
pub use rule::{Rule, RuleBuilder};
pub use traits::{Numeric, OptionLike, Validator};

// v1.0 API
pub use fluent_validator::{FluentValidator, PropertyRuleBuilder};

/// Create a [`PropertyRuleBuilder`] for a field on a [`FluentValidator`].
///
/// `rule_for!(v, c.field)` expands to `v.rule_for("field", |c| &c.field)`.
/// `rule_for!(v, c.field.subfield)` expands to `v.rule_for("field.subfield", |c| &c.field.subfield)`.
///
/// The identifier before the `.` (`c` in the examples) is used as the closure parameter name.
#[macro_export]
macro_rules! rule_for {
    ($v:expr, $c:ident . $field:ident) => {
        $v.rule_for(stringify!($field), |$c| &$c.$field)
    };
    ($v:expr, $c:ident . $field:ident . $subfield:ident) => {
        $v.rule_for(
            concat!(stringify!($field), ".", stringify!($subfield)),
            |$c| &$c.$field.$subfield,
        )
    };
}
