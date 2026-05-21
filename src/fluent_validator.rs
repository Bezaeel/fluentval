use std::cell::RefCell;
use std::rc::Rc;

use crate::error::{ValidationError, ValidationResult};
use crate::rule::RuleBuilder;
use crate::traits::Validator;

/// Entry point for the fluent validation API.
///
/// Construct a validator, register property rules via [`rule_for`](Self::rule_for) (or the
/// [`rule_for!`](crate::rule_for) macro), then call [`validate`](Validator::validate) on the
/// validator itself — there is no separate build step.
///
/// Each `rule_for(...)` call returns a [`RuleBuilder<T, V>`] that auto-registers into the
/// validator when it is dropped (at the `;` ending the statement).
///
/// # Example
/// ```rust,ignore
/// use fluentval::{FluentValidator, rule_for, Validator};
///
/// struct User { name: String, email: String }
///
/// let v = FluentValidator::<User>::new();
/// // Explicit form:
/// v.rule_for("name", |u| &u.name).not_empty(None::<String>).min_length(2, None::<String>);
/// // Macro sugar:
/// rule_for!(v, |u| &u.email).not_empty(None::<String>).email(None::<String>);
///
/// let user = User { name: "Ada".into(), email: "ada@example.com".into() };
/// let result = v.validate(&user);
/// ```
pub struct FluentValidator<T: 'static> {
    rules: Rc<RefCell<Vec<Box<dyn Fn(&T) -> Vec<ValidationError>>>>>,
}

impl<T: 'static> FluentValidator<T> {
    /// Create a new, empty `FluentValidator`.
    pub fn new() -> Self {
        Self {
            rules: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Return a [`RuleBuilder<T, V>`] bound to this validator.
    ///
    /// The returned builder registers itself into the validator when dropped. Prefer the
    /// [`rule_for!`](crate::rule_for) macro for the same effect with less typing.
    pub fn rule_for<V: 'static, F>(
        &self,
        property_name: impl Into<String>,
        accessor: F,
    ) -> RuleBuilder<T, V>
    where
        F: Fn(&T) -> &V + 'static,
    {
        RuleBuilder::for_property(property_name, accessor).with_parent(Rc::clone(&self.rules))
    }
}

impl<T: 'static> Default for FluentValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> Validator<T> for FluentValidator<T> {
    fn validate(&self, instance: &T) -> ValidationResult {
        let mut result = ValidationResult::new();
        for rule in self.rules.borrow().iter() {
            result.add_errors(rule(instance));
        }
        result
    }
}
