use std::fmt::Display;

use super::consts::{SUFFIX, WALL};

pub struct FixedLength<T>(T);

impl<T> FixedLength<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T: Display> Display for FixedLength<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.0.to_string();
        if let Some(length) = f.width() {
            let length = length - WALL.len();
            if data.len() > length {
                write!(
                    f,
                    "{}{SUFFIX}",
                    &data[0..length.checked_sub(3).unwrap_or_default()]
                )
            } else {
                write!(f, "{data:length$}")
            }
        } else {
            Display::fmt(&self.0, f)
        }
    }
}
