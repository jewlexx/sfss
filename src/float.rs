#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

pub mod usize {
    /// Convert a `usize` to a `f64` if possible
    ///
    /// Checks that the conversion is lossless by asserting that the value is equal to the result
    /// of the conversion
    #[must_use]
    pub fn convert_to_f64(value: usize) -> Option<f64> {
        let result = convert_to_f64_debug(value);
        (result as usize == value).then_some(result)
    }

    /// Convert a `usize` to a `f64` if possible
    ///
    /// This is almost equivalent to `as f64` and does not check that the conversion is lossless
    /// but will panic if the conversion is not lossless in debug mode
    #[must_use]
    pub fn convert_to_f64_debug(value: usize) -> f64 {
        let result = value as f64;
        debug_assert_eq!(result as usize, value);
        result
    }
}
