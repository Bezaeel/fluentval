use crate::error::ValidationResult;

/// Trait for defining validators
pub trait Validator<T> {
    fn validate(&self, instance: &T) -> ValidationResult;
}

/// Trait for types that can be treated as numeric values
pub trait Numeric {
    fn to_f64(&self) -> f64;
}

impl Numeric for i8 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i16 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i32 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for i64 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u8 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u16 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u32 { fn to_f64(&self) -> f64 { *self as f64 } }
impl Numeric for u64 { fn to_f64(&self) -> f64 { *self as f64 } }
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

