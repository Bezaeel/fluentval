use std::sync::{Arc, LazyLock, PoisonError, RwLock};

use regex::Regex;

use crate::error::ValidationError;
use crate::traits::{NotOption, Numeric, OptionLike};

/// A compiled, thread-safe rule: takes the object under validation and returns its errors.
pub(crate) type BoxedRule<T> = Box<dyn Fn(&T) -> Vec<ValidationError> + Send + Sync>;

/// Shared rule list owned by a `FluentValidator` and appended to by its `RuleBuilder`s.
pub(crate) type ParentRules<T> = Arc<RwLock<Vec<BoxedRule<T>>>>;

type Accessor<T, V> = Box<dyn Fn(&T) -> &V + Send + Sync>;
type ValueRule<V> = Box<dyn Fn(&V) -> Option<String> + Send + Sync>;
type CrossRule<T, V> = Box<dyn Fn(&T, &V) -> Option<String> + Send + Sync>;
type Condition<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;

/// Email pattern, compiled once on first use and shared across all rules and threads.
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("email regex is valid")
});

/// Builder for validation rules.
///
/// Carries both the struct type `T` (the object being validated) and the field type `V`
/// (the property under inspection). Build with [`for_property`](Self::for_property), passing
/// the property name and an accessor closure `|t: &T| &t.field`.
///
/// For a rule on a bare value (no surrounding struct), pass an identity accessor
/// `|v: &V| v` — `T` and `V` both infer to the value's type.
///
/// # Standalone
/// ```rust,ignore
/// // Value-only rule (identity accessor):
/// let rule = RuleBuilder::for_property("name", |s: &String| s)
///     .not_empty(None::<String>)
///     .min_length(2, None::<String>)
///     .build();
/// let errors = rule(&"".to_string());
///
/// // Struct-aware (cross-property):
/// let rule = RuleBuilder::for_property("alt_phone", |c: &Request| &c.alt_phone)
///     .not_empty(None::<String>)
///     .must(|c, v| v != &c.primary_phone, "alt phone cannot match primary")
///     .build();
/// let errors = rule(&Request { alt_phone: "x".into(), primary_phone: "x".into() });
/// ```
///
/// # Inside a `FluentValidator`
/// Rules registered via [`FluentValidator::rule_for`](crate::FluentValidator::rule_for) hold
/// a reference to the validator and self-register when the builder is dropped (at the `;`
/// ending the statement) — no `.build()` call required.
pub struct RuleBuilder<T: 'static, V: 'static> {
    property_name: String,
    accessor: Option<Accessor<T, V>>,
    value_rules: Vec<ValueRule<V>>,
    cross_rules: Vec<CrossRule<T, V>>,
    condition: Option<Condition<T>>,
    parent_rules: Option<ParentRules<T>>,
}

impl<T: 'static, V: 'static> RuleBuilder<T, V> {
    /// Create a builder for a property accessed via a closure.
    ///
    /// Use this when validating a field of a struct, especially when you need cross-property
    /// access via [`must`](Self::must).
    pub fn for_property<F>(property_name: impl Into<String>, accessor: F) -> Self
    where
        F: Fn(&T) -> &V + Send + Sync + 'static,
    {
        Self {
            property_name: property_name.into(),
            accessor: Some(Box::new(accessor)),
            value_rules: Vec::new(),
            cross_rules: Vec::new(),
            condition: None,
            parent_rules: None,
        }
    }

    pub(crate) fn with_parent(mut self, parent: ParentRules<T>) -> Self {
        self.parent_rules = Some(parent);
        self
    }

    /// Add a custom value-level rule.
    pub fn rule(mut self, rule: impl Fn(&V) -> Option<String> + Send + Sync + 'static) -> Self {
        self.value_rules.push(Box::new(rule));
        self
    }

    /// Gate the entire rule chain on a struct-level condition.
    ///
    /// When the predicate returns `false`, no rules on this builder fire — the property is
    /// treated as valid. Useful for conditional requirements like "billing address is
    /// required only when payment_method is CARD".
    ///
    /// Calling `.when(...)` multiple times ANDs the predicates together; order in the chain
    /// does not matter — the condition gates every rule registered on this builder.
    pub fn when(mut self, predicate: impl Fn(&T) -> bool + Send + Sync + 'static) -> Self {
        let predicate: Condition<T> = Box::new(predicate);
        self.condition = Some(match self.condition.take() {
            Some(existing) => Box::new(move |t| existing(t) && predicate(t)),
            None => predicate,
        });
        self
    }

    /// Add a cross-property rule. The predicate receives the full object and the field value.
    pub fn must(
        mut self,
        predicate: impl Fn(&T, &V) -> bool + Send + Sync + 'static,
        message: impl Into<String>,
    ) -> Self {
        let msg = message.into();
        self.cross_rules.push(Box::new(move |t, v| {
            if !predicate(t, v) {
                Some(msg.clone())
            } else {
                None
            }
        }));
        self
    }

    /// Build the rule chain into a function `Fn(&T) -> Vec<ValidationError>`.
    ///
    /// Consumes the builder; suppresses the auto-registration that would otherwise occur
    /// when a builder bound to a [`FluentValidator`](crate::FluentValidator) is dropped.
    pub fn build(mut self) -> impl Fn(&T) -> Vec<ValidationError> + Send + Sync {
        self.parent_rules = None;
        self.take_rule_fn()
    }

    /// Move the accumulated rules out of the builder and compile them into a single closure.
    ///
    /// Called exactly once per builder — from either [`build`](Self::build) or `Drop` — so the
    /// accessor is always present.
    fn take_rule_fn(&mut self) -> impl Fn(&T) -> Vec<ValidationError> + Send + Sync + 'static {
        let property_name = std::mem::take(&mut self.property_name);
        let accessor = self.accessor.take().expect("RuleBuilder accessor missing");
        let value_rules = std::mem::take(&mut self.value_rules);
        let cross_rules = std::mem::take(&mut self.cross_rules);
        let condition = self.condition.take();

        move |instance: &T| {
            if let Some(cond) = &condition {
                if !cond(instance) {
                    return Vec::new();
                }
            }
            let value = accessor(instance);
            let mut errors = Vec::new();
            for r in &value_rules {
                if let Some(message) = r(value) {
                    errors.push(ValidationError::new(property_name.clone(), message));
                }
            }
            for r in &cross_rules {
                if let Some(message) = r(instance, value) {
                    errors.push(ValidationError::new(property_name.clone(), message));
                }
            }
            errors
        }
    }
}

impl<T: 'static, V: 'static> Drop for RuleBuilder<T, V> {
    fn drop(&mut self) {
        let Some(parent) = self.parent_rules.take() else {
            return;
        };
        let rule_fn = self.take_rule_fn();
        // A poisoned lock only means another thread panicked mid-push; the Vec is still
        // structurally valid, so recover it rather than propagating the panic.
        parent
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Box::new(rule_fn));
    }
}

// ── String rules ─────────────────────────────────────────────────────────────

impl<T: 'static, V: AsRef<str> + NotOption + 'static> RuleBuilder<T, V> {
    pub fn not_empty(self, message: Option<impl Into<String>>) -> Self {
        let msg = message
            .map(|m| m.into())
            .unwrap_or_else(|| "must not be empty".to_string());
        self.rule(move |value| {
            if value.as_ref().trim().is_empty() {
                Some(msg.clone())
            } else {
                None
            }
        })
    }

    pub fn min_length(
        self,
        min: usize,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.as_ref().len() < min {
                Some(
                    msg.clone()
                        .unwrap_or_else(|| format!("must be at least {} characters long", min)),
                )
            } else {
                None
            }
        })
    }

    pub fn max_length(
        self,
        max: usize,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.as_ref().len() > max {
                Some(
                    msg.clone()
                        .unwrap_or_else(|| format!("must be at most {} characters long", max)),
                )
            } else {
                None
            }
        })
    }

    pub fn length(
        self,
        min: usize,
        max: usize,
        min_message: Option<impl Into<String> + Clone + 'static>,
        max_message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        self.min_length(min, min_message).max_length(max, max_message)
    }

    pub fn email(self, message: Option<impl Into<String>>) -> Self {
        let msg = message
            .map(|m| m.into())
            .unwrap_or_else(|| "must be a valid email address".to_string());
        self.rule(move |value| {
            if !EMAIL_REGEX.is_match(value.as_ref()) {
                Some(msg.clone())
            } else {
                None
            }
        })
    }
}

// ── Option rules ─────────────────────────────────────────────────────────────

impl<T: 'static, V: OptionLike + 'static> RuleBuilder<T, V> {
    pub fn not_null(self, message: Option<impl Into<String>>) -> Self {
        let msg = message
            .map(|m| m.into())
            .unwrap_or_else(|| "must not be null".to_string());
        self.rule(move |value| {
            if value.is_none() {
                Some(msg.clone())
            } else {
                None
            }
        })
    }
}

// ── Option-transparent string rules ──────────────────────────────────────────
//
// When the field is `Option<V>` with `V: AsRef<str>`, string rules apply only if the value
// is `Some(_)`. A `None` value passes — pair with `not_null` if presence is required.

impl<T: 'static, V: AsRef<str> + 'static> RuleBuilder<T, Option<V>> {
    pub fn not_empty(self, message: Option<impl Into<String>>) -> Self {
        let msg = message
            .map(|m| m.into())
            .unwrap_or_else(|| "must not be empty".to_string());
        self.rule(move |opt| match opt {
            Some(v) if v.as_ref().trim().is_empty() => Some(msg.clone()),
            _ => None,
        })
    }

    pub fn min_length(
        self,
        min: usize,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let msg = message.map(|m| m.into());
        self.rule(move |opt| match opt {
            Some(v) if v.as_ref().len() < min => Some(
                msg.clone()
                    .unwrap_or_else(|| format!("must be at least {} characters long", min)),
            ),
            _ => None,
        })
    }

    pub fn max_length(
        self,
        max: usize,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let msg = message.map(|m| m.into());
        self.rule(move |opt| match opt {
            Some(v) if v.as_ref().len() > max => Some(
                msg.clone()
                    .unwrap_or_else(|| format!("must be at most {} characters long", max)),
            ),
            _ => None,
        })
    }

    pub fn length(
        self,
        min: usize,
        max: usize,
        min_message: Option<impl Into<String> + Clone + 'static>,
        max_message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        self.min_length(min, min_message).max_length(max, max_message)
    }

    pub fn email(self, message: Option<impl Into<String>>) -> Self {
        let msg = message
            .map(|m| m.into())
            .unwrap_or_else(|| "must be a valid email address".to_string());
        self.rule(move |opt| match opt {
            Some(v) if !EMAIL_REGEX.is_match(v.as_ref()) => Some(msg.clone()),
            _ => None,
        })
    }
}

// ── Option-transparent numeric rules ─────────────────────────────────────────

impl<T: 'static, V: Numeric + 'static> RuleBuilder<T, Option<V>> {
    pub fn greater_than(
        self,
        min: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let min_val = min.into();
        let msg = message.map(|m| m.into());
        self.rule(move |opt| match opt {
            Some(v) if v.to_f64() <= min_val => Some(
                msg.clone()
                    .unwrap_or_else(|| format!("must be greater than {}", min_val)),
            ),
            _ => None,
        })
    }

    pub fn greater_than_or_equal(
        self,
        min: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let min_val = min.into();
        let msg = message.map(|m| m.into());
        self.rule(move |opt| match opt {
            Some(v) if v.to_f64() < min_val => Some(msg.clone().unwrap_or_else(|| {
                format!("must be greater than or equal to {}", min_val)
            })),
            _ => None,
        })
    }

    pub fn less_than(
        self,
        max: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |opt| match opt {
            Some(v) if v.to_f64() >= max_val => Some(
                msg.clone()
                    .unwrap_or_else(|| format!("must be less than {}", max_val)),
            ),
            _ => None,
        })
    }

    pub fn less_than_or_equal(
        self,
        max: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |opt| match opt {
            Some(v) if v.to_f64() > max_val => Some(msg.clone().unwrap_or_else(|| {
                format!("must be less than or equal to {}", max_val)
            })),
            _ => None,
        })
    }

    pub fn inclusive_between(
        self,
        min: impl Into<f64> + Copy + 'static,
        max: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let min_val = min.into();
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |opt| match opt {
            Some(v) => {
                let val = v.to_f64();
                if val < min_val || val > max_val {
                    Some(
                        msg.clone()
                            .unwrap_or_else(|| format!("must be between {} and {}", min_val, max_val)),
                    )
                } else {
                    None
                }
            }
            None => None,
        })
    }
}

// ── Numeric rules ────────────────────────────────────────────────────────────

impl<T: 'static, V: Numeric + NotOption + 'static> RuleBuilder<T, V> {
    pub fn greater_than(
        self,
        min: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let min_val = min.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.to_f64() <= min_val {
                Some(
                    msg.clone()
                        .unwrap_or_else(|| format!("must be greater than {}", min_val)),
                )
            } else {
                None
            }
        })
    }

    pub fn greater_than_or_equal(
        self,
        min: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let min_val = min.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.to_f64() < min_val {
                Some(msg.clone().unwrap_or_else(|| {
                    format!("must be greater than or equal to {}", min_val)
                }))
            } else {
                None
            }
        })
    }

    pub fn less_than(
        self,
        max: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.to_f64() >= max_val {
                Some(
                    msg.clone()
                        .unwrap_or_else(|| format!("must be less than {}", max_val)),
                )
            } else {
                None
            }
        })
    }

    pub fn less_than_or_equal(
        self,
        max: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            if value.to_f64() > max_val {
                Some(msg.clone().unwrap_or_else(|| {
                    format!("must be less than or equal to {}", max_val)
                }))
            } else {
                None
            }
        })
    }

    pub fn inclusive_between(
        self,
        min: impl Into<f64> + Copy + 'static,
        max: impl Into<f64> + Copy + 'static,
        message: Option<impl Into<String> + Clone + 'static>,
    ) -> Self {
        let min_val = min.into();
        let max_val = max.into();
        let msg = message.map(|m| m.into());
        self.rule(move |value| {
            let val = value.to_f64();
            if val < min_val || val > max_val {
                Some(
                    msg.clone()
                        .unwrap_or_else(|| format!("must be between {} and {}", min_val, max_val)),
                )
            } else {
                None
            }
        })
    }
}
