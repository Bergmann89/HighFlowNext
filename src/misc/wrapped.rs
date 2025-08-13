use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::ops::Deref;

use thiserror::Error;

use crate::misc::{Decode, Guard, GuardOutput, IoError, Reader};

/// Macro to define a new typed wrapper around a primitive value.
///
/// Example:
/// ```rust,ignore
/// define_wrapped! {
///     /// Brightness level (0..255).
///     pub type Brightness<u8, BrightnessTag>;
/// }
/// ```
#[macro_export]
macro_rules! define_wrapped {
    ($(#[$meta:meta])* $pub:vis type $name:ident<$base:ty, $tag:ident> ;) => {
        $(#[$meta])*
        $pub type $name = $crate::misc::Wrapped<$base, $tag>;

        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
        pub struct $tag;
    };
}

/// Macro to implement a trivial verifier that accepts all values.
///
/// Use this when no validation is needed for the type.
#[macro_export]
macro_rules! impl_verify_simple {
    ($value_type:ident<$base:ty, $tag:ident>) => {
        impl $crate::misc::ValueVerifier<$base> for $tag {
            type Error = core::convert::Infallible;

            fn verify(val: $base) -> Result<$base, Self::Error> {
                Ok(val)
            }
        }
    };
}

/// Macro to implement a ranged verifier for a wrapper type.
///
/// The type will only accept values within the inclusive range [`min`, `max`].
#[macro_export]
macro_rules! impl_ranged {
    ($value_type:ident<$base:ty, $tag:ident>, $min:expr, $max:expr) => {
        impl $crate::misc::Ranged<$base> for $tag {
            #[inline]
            fn min_inclusive() -> $base {
                $min
            }

            #[inline]
            fn max_inclusive() -> $base {
                $max
            }
        }
    };
}

/// A strongly typed wrapper around a primitive value with validation.
///
/// Wrappers are parameterized by a phantom `tag` type which implements
/// either [`Ranged`] or [`ValueVerifier`]. This allows creating distinct
/// types from the same base primitive while enforcing domain-specific rules.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Wrapped<T, X> {
    value: T,
    tag: PhantomData<X>,
}

impl<T, X> Wrapped<T, X>
where
    T: Ord,
    X: Ranged<T>,
{
    /// Returns the minimum allowed value.
    #[must_use]
    pub fn min_inclusive() -> T {
        X::min_inclusive()
    }

    /// Returns the maximum allowed value.
    #[must_use]
    pub fn max_inclusive() -> T {
        X::max_inclusive()
    }
}

impl<T, X> Wrapped<T, X>
where
    X: ValueVerifier<T>,
{
    /// Creates a new wrapper from a primitive value,
    /// validating it with the associated verifier.
    pub fn from_value(val: T) -> Result<Self, X::Error> {
        let val = X::verify(val)?;

        Ok(Self {
            value: val,
            tag: PhantomData,
        })
    }
}

impl<T, X> Deref for Wrapped<T, X> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Implements [`Decode`] for `Wrapped<u8, X>`.
impl<X> Decode for Wrapped<u8, X>
where
    X: ValueVerifier<u8>,
    IoError: From<X::Error>,
{
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let val = reader.read_u8()?;
        let ret = R::guard(|_| Self::from_value(val));

        Ok(R::Guard::transpose_result(ret)?)
    }
}

/// Implements [`Decode`] for `Wrapped<u16, X>`.
impl<X> Decode for Wrapped<u16, X>
where
    X: ValueVerifier<u16>,
    IoError: From<X::Error>,
{
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let val = reader.read_u16be()?;
        let ret = R::guard(|_| Self::from_value(val));

        Ok(R::Guard::transpose_result(ret)?)
    }
}

/// Implements [`Decode`] for `Wrapped<i16, X>`.
impl<X> Decode for Wrapped<i16, X>
where
    X: ValueVerifier<i16>,
    IoError: From<X::Error>,
{
    fn decode<R: Reader>(reader: &mut R) -> Result<GuardOutput<R, Self>, IoError> {
        let val = reader.read_i16be()?;
        let ret = R::guard(|_| Self::from_value(val));

        Ok(R::Guard::transpose_result(ret)?)
    }
}

/// Trait for types that define inclusive minimum and maximum bounds.
pub trait Ranged<T> {
    /// Return the minimum inclusive value for this range.
    fn min_inclusive() -> T;

    /// Return the maximum inclusive value for this range.
    fn max_inclusive() -> T;
}

/// Trait for types that verify whether a value is valid.
///
/// Custom verifiers can reject values outside ranges, apply additional
/// constraints, or simply accept all values.
pub trait ValueVerifier<T> {
    /// Error returned by the verifier.
    type Error;

    /// Verify if the given `value` is valid, returning either the valid value
    /// `Ok(value)` or a suitable error `Err(error)`.
    fn verify(value: T) -> Result<T, Self::Error>;
}

/// Default implementation of [`ValueVerifier`] for any [`Ranged`] type.
///
/// Rejects values outside the inclusive min/max bounds.
impl<T, X> ValueVerifier<T> for X
where
    X: Ranged<T>,
    T: Ord,
{
    type Error = RangeError<T>;

    fn verify(val: T) -> Result<T, Self::Error> {
        let min = X::min_inclusive();
        let max = X::max_inclusive();

        if val < min || val > max {
            Err(RangeError { min, max, val })
        } else {
            Ok(val)
        }
    }
}

/// Error returned when a value lies outside the allowed range.
#[derive(Debug, Error)]
#[error("Value out of range (min={min}, max={max}, val={val})!")]
pub struct RangeError<T> {
    /// Minimum inclusive value of the range.
    pub min: T,

    /// Maximum inclusive value of the range.
    pub max: T,

    /// Actual value (not in range).
    pub val: T,
}

impl<T> RangeError<T>
where
    T: Display,
{
    /// Converts this error into an owned `RangeError<String>`,
    /// useful for error reporting where `T` is not `Clone`.
    pub fn to_owned(&self) -> RangeError<String> {
        RangeError {
            min: self.min.to_string(),
            max: self.max.to_string(),
            val: self.val.to_string(),
        }
    }
}
