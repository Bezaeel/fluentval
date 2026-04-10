use std::cell::RefCell;
use std::rc::Rc;

use crate::error::{ValidationError, ValidationResult};
use crate::rule::RuleBuilder;
use crate::traits::{Numeric, OptionLike, Validator};

/// Entry point for the v1.0 fluent validation API.
///
/// Create a validator by calling [`FluentValidator::new`], registering property rules via the
/// [`rule_for!`] macro (or [`FluentValidator::rule_for`] directly), then calling
/// [`FluentValidator::build`] to obtain a [`Validator<T>`].
///
/// Rules accumulate on [`PropertyRuleBuilder`] values and are registered into the parent
/// `FluentValidator` when each builder is dropped (at the `;` ending the statement).
///
/// # Example
/// ```rust,ignore
/// use fluentval::{FluentValidator, rule_for, Validator};
///
/// struct User { name: String, email: String }
///
/// let v = FluentValidator::<User>::new();
/// rule_for!(v, c.name).not_empty(None::<String>).min_length(2, None::<String>);
/// rule_for!(v, c.email).not_empty(None::<String>).email(None::<String>);
/// let validator = v.build();
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

    /// Return a [`PropertyRuleBuilder`] that collects rules for the given property.
    ///
    /// Prefer the [`rule_for!`] macro, which extracts the property name automatically.
    /// Call this directly only when you need a custom property label.
    pub fn rule_for<V: 'static, F>(
        &self,
        property_name: impl Into<String>,
        accessor: F,
    ) -> PropertyRuleBuilder<T, V>
    where
        F: Fn(&T) -> &V + 'static,
    {
        let name = property_name.into();
        PropertyRuleBuilder {
            property_name: name.clone(),
            accessor: Some(Box::new(accessor)),
            inner: Some(RuleBuilder::for_property(name)),
            cross_rules: Vec::new(),
            parent_rules: Rc::clone(&self.rules),
        }
    }

    /// Consume the `FluentValidator` and return a [`Validator<T>`].
    ///
    /// All [`PropertyRuleBuilder`] values must have been dropped before calling `build()`.
    ///
    /// # Panics
    /// Panics if any `PropertyRuleBuilder` is still alive.
    pub fn build(self) -> impl Validator<T> {
        let rules = Rc::try_unwrap(self.rules)
            .unwrap_or_else(|_| panic!("all rule_for() builders must be dropped before calling build()"))
            .into_inner();
        FluentValidatorImpl { rules }
    }
}

impl<T: 'static> Default for FluentValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}

struct FluentValidatorImpl<T> {
    rules: Vec<Box<dyn Fn(&T) -> Vec<ValidationError>>>,
}

impl<T> Validator<T> for FluentValidatorImpl<T> {
    fn validate(&self, instance: &T) -> ValidationResult {
        let mut result = ValidationResult::new();
        for rule in &self.rules {
            result.add_errors(rule(instance));
        }
        result
    }
}

/// Builder for property-level rules, created by [`FluentValidator::rule_for`] or the
/// [`rule_for!`] macro.
///
/// All value-level rules (`not_empty`, `min_length`, etc.) delegate to the inner
/// [`RuleBuilder<V>`] — no logic is duplicated here. The only genuinely new method is
/// [`must`](PropertyRuleBuilder::must), whose predicate receives the full object (`&T`).
///
/// When this builder is dropped the accumulated rules are registered into the parent
/// [`FluentValidator`].
pub struct PropertyRuleBuilder<T: 'static, V: 'static> {
    /// Kept separately so `cross_rules` can produce `ValidationError` with the right name.
    property_name: String,
    accessor: Option<Box<dyn Fn(&T) -> &V + 'static>>,
    /// Owns all value-level rules; delegates entirely to `RuleBuilder`.
    inner: Option<RuleBuilder<V>>,
    /// Cross-property predicates added via `must()`.
    cross_rules: Vec<Box<dyn Fn(&T, &V) -> Option<String>>>,
    parent_rules: Rc<RefCell<Vec<Box<dyn Fn(&T) -> Vec<ValidationError>>>>>,
}

impl<T: 'static, V: 'static> Drop for PropertyRuleBuilder<T, V> {
    fn drop(&mut self) {
        let property_name = std::mem::take(&mut self.property_name);
        let accessor = self.accessor.take().expect("accessor was already consumed");
        let value_rule_fn = self.inner.take().expect("inner was already consumed").build();
        let cross_rules = std::mem::take(&mut self.cross_rules);

        let rule_fn = move |instance: &T| {
            let value = accessor(instance);
            let mut errors = value_rule_fn(value);
            for r in &cross_rules {
                if let Some(msg) = r(instance, value) {
                    errors.push(ValidationError::new(property_name.clone(), msg));
                }
            }
            errors
        };

        self.parent_rules.borrow_mut().push(Box::new(rule_fn));
    }
}

// ── Methods unique to PropertyRuleBuilder ────────────────────────────────────

impl<T: 'static, V: 'static> PropertyRuleBuilder<T, V> {
    /// Add a cross-property validation rule.
    ///
    /// The predicate receives the full object (`&T`) and the property value (`&V`), enabling
    /// validation that depends on sibling fields.
    ///
    /// ```rust,ignore
    /// rule_for!(v, c.alt_phone)
    ///     .must(|cmd, _| cmd.alt_phone != cmd.phone,
    ///           "Alternative phone cannot match primary phone");
    /// ```
    pub fn must(
        mut self,
        predicate: impl Fn(&T, &V) -> bool + 'static,
        message: impl Into<String> + 'static,
    ) -> Self {
        let msg = message.into();
        self.cross_rules.push(Box::new(move |t: &T, v: &V| {
            if !predicate(t, v) { Some(msg.clone()) } else { None }
        }));
        self
    }
}

// ── Delegation to RuleBuilder — no logic duplicated ──────────────────────────
//
// Every method below simply forwards to the inner `RuleBuilder<V>`, which owns all
// validation logic. Adding a new rule to `RuleBuilder` automatically appears here.

impl<T: 'static, V: 'static> PropertyRuleBuilder<T, V> {
    /// Add a custom rule that validates the property value only.
    pub fn rule(mut self, r: impl Fn(&V) -> Option<String> + 'static) -> Self {
        self.inner = Some(self.inner.take().unwrap().rule(r));
        self
    }
}

impl<T: 'static, V: AsRef<str> + 'static> PropertyRuleBuilder<T, V> {
    pub fn not_empty(mut self, message: Option<impl Into<String>>) -> Self {
        self.inner = Some(self.inner.take().unwrap().not_empty(message));
        self
    }

    pub fn min_length(mut self, min: usize, message: Option<impl Into<String> + Clone + 'static>) -> Self {
        self.inner = Some(self.inner.take().unwrap().min_length(min, message));
        self
    }

    pub fn max_length(mut self, max: usize, message: Option<impl Into<String> + Clone + 'static>) -> Self {
        self.inner = Some(self.inner.take().unwrap().max_length(max, message));
        self
    }

    pub fn length(
        mut self,
        min: usize,
        max: usize,
        min_message: Option<impl Into<String> + Clone + 'static>,
        max_message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        self.inner = Some(self.inner.take().unwrap().length(min, max, min_message, max_message));
        self
    }

    pub fn email(mut self, message: Option<impl Into<String>>) -> Self {
        self.inner = Some(self.inner.take().unwrap().email(message));
        self
    }
}

impl<T: 'static, V: OptionLike + 'static> PropertyRuleBuilder<T, V> {
    pub fn not_null(mut self, message: Option<impl Into<String>>) -> Self {
        self.inner = Some(self.inner.take().unwrap().not_null(message));
        self
    }
}

impl<T: 'static, V: Numeric + 'static> PropertyRuleBuilder<T, V> {
    pub fn greater_than(mut self, min: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self {
        self.inner = Some(self.inner.take().unwrap().greater_than(min, message));
        self
    }

    pub fn greater_than_or_equal(mut self, min: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self {
        self.inner = Some(self.inner.take().unwrap().greater_than_or_equal(min, message));
        self
    }

    pub fn less_than(mut self, max: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self {
        self.inner = Some(self.inner.take().unwrap().less_than(max, message));
        self
    }

    pub fn less_than_or_equal(mut self, max: impl Into<f64> + Copy + 'static, message: Option<impl Into<String> + Clone + 'static>) -> Self {
        self.inner = Some(self.inner.take().unwrap().less_than_or_equal(max, message));
        self
    }

    pub fn inclusive_between(
        mut self,
        min: impl Into<f64> + Copy + 'static,
        max: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        self.inner = Some(self.inner.take().unwrap().inclusive_between(min, max, message));
        self
    }
}
