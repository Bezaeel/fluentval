//! FluentVal — a fluent validation library for Rust.
//!
//! The API centers on two types:
//!
//! - [`RuleBuilder<T, V>`] — builds a validation rule chain for a field of type `V` on a
//!   struct of type `T`. Use standalone when you want a callable `Fn(&T) -> Vec<ValidationError>`.
//! - [`FluentValidator<T>`] — collects rules across many fields and exposes
//!   [`Validator::validate`].
//!
//! Each call to [`FluentValidator::rule_for`] (or the [`rule_for!`] macro) returns a
//! `RuleBuilder` that auto-registers into the validator when dropped — no build step.
//!
//! # Example
//! ```rust,ignore
//! use fluentval::{FluentValidator, rule_for, Validator};
//!
//! struct User { name: String, email: String, age: i32 }
//!
//! let v = FluentValidator::<User>::new();
//! rule_for!(v, |u| &u.name).not_empty(None::<String>).min_length(2, None::<String>);
//! rule_for!(v, |u| &u.email).not_empty(None::<String>).email(None::<String>);
//! rule_for!(v, |u| &u.age).greater_than_or_equal(18, Some("Must be 18 or older"));
//!
//! let user = User { name: "".into(), email: "invalid".into(), age: 15 };
//! let result = v.validate(&user);
//! for error in result.errors() {
//!     println!("{}: {}", error.property, error.message);
//! }
//! ```
//!
//! Or, without the macro:
//! ```rust,ignore
//! v.rule_for("name", |u| &u.name).not_empty(None::<String>);
//! ```

mod error;
mod fluent_validator;
mod rule;
mod traits;

pub use error::{ValidationError, ValidationResult};
pub use fluent_validator::FluentValidator;
pub use rule::RuleBuilder;
pub use traits::{NotOption, Numeric, OptionLike, Validator};

/// Create a [`RuleBuilder`] for a field on a [`FluentValidator`].
///
/// Takes a closure literal `|binding| &binding.field`:
///
/// - `rule_for!(v, |c| &c.field)` expands to `v.rule_for("field", |c| &c.field)`.
/// - `rule_for!(v, |c| &c.field.subfield)` expands to
///   `v.rule_for("field.subfield", |c| &c.field.subfield)`.
///
/// The identifier(s) after the last `.` become the property name (via `stringify!`); the
/// binding on the left of `|...|` is whatever you choose.
#[macro_export]
macro_rules! rule_for {
    ($v:expr, | $c:ident | & $base:ident . $field:ident) => {
        $v.rule_for(stringify!($field), |$c: &_| &$base.$field)
    };
    ($v:expr, | $c:ident | & $base:ident . $field:ident . $subfield:ident) => {
        $v.rule_for(
            concat!(stringify!($field), ".", stringify!($subfield)),
            |$c: &_| &$base.$field.$subfield,
        )
    };
}
