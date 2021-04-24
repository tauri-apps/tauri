//! Provides validation for text inputs
use std::fmt::{Debug, Display};

/// Trait for input validators.
///
/// A generic implementation for `Fn(&str) -> Result<(), E>` is provided
/// to facilitate development.
pub trait Validator<T> {
    type Err: Debug + Display;

    /// Invoked with the value to validate.
    ///
    /// If this produces `Ok(())` then the value is used and parsed, if
    /// an error is returned validation fails with that error.
    fn validate(&mut self, input: &T) -> Result<(), Self::Err>;
}

impl<T, F: FnMut(&T) -> Result<(), E>, E: Debug + Display> Validator<T> for F {
    type Err = E;

    fn validate(&mut self, input: &T) -> Result<(), Self::Err> {
        self(input)
    }
}
