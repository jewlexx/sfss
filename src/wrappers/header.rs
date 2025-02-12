//! A nicer way to display headers

use std::fmt::Display;

use itertools::Itertools;

use crate::handlers::upper_first_char;

#[derive(Debug, Clone)]
#[must_use = "Lazy. Does nothing until consumed"]
/// A nicer way to display headers
pub struct Header<T>(T);

impl<T: Display> Header<T> {
    /// Create a new [`Header`] from the provided value
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Display> Display for Header<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .0
            .to_string()
            .split('_')
            .map(|word| upper_first_char(word).to_string())
            .join(" ");

        write!(f, "{string}")
    }
}
