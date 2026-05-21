use crate::error::ValidationResult;

/// Trait for defining validators
pub trait Validator<T> {
    fn validate(&self, instance: &T) -> ValidationResult;
}

mod sealed {
    pub trait Sealed {}
}

/// Sealed marker for bare (non-`Option`) field types.
///
/// Used internally to disambiguate rule methods between bare values (`String`, `i32`, …) and
/// `Option`-wrapped values. Implemented for `String`, `&str`, `str`, and the integer/float
/// primitives. Custom field types can validate via [`RuleBuilder::rule`](crate::RuleBuilder::rule).
pub trait NotOption: sealed::Sealed {}

macro_rules! impl_not_option {
    ($($t:ty),* $(,)?) => {
        $(
            impl sealed::Sealed for $t {}
            impl NotOption for $t {}
        )*
    };
}

impl_not_option!(
    String, &str, str,
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
    f32, f64,
);

/// Trait for types that can be treated as numeric values
pub trait Numeric {
    fn to_f64(&self) -> f64;
}

impl Numeric for i8 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i16 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i32 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i64 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for isize { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u8 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u16 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u32 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u64 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for usize { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for f32 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for f64 { fn to_f64(&self) -> f64 { *self } }

/// Trait for types that can be treated as Option-like
pub trait OptionLike {
    fn is_none(&self) -> bool;
}

impl<T> OptionLike for Option<T> {
    fn is_none(&self) -> bool {
        Option::is_none(self)
    }
}

