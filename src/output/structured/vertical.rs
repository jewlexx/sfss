//! Vertical sectioned output

use std::fmt::Display;

use itertools::Itertools;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::wrappers::header::Header;

#[must_use = "VTable is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct VTable {
    object: Map<String, Value>,
    format_headers: bool,
}

impl VTable {
    /// Construct a new [`VTable`] formatter
    ///
    /// # Panics
    /// - If the values provided are not objects
    pub fn new(value: impl Serialize) -> Self {
        let value = serde_json::to_value(value).expect("valid value");
        let object = value.as_object().expect("Value must be an object").clone();

        Self {
            object,
            format_headers: true,
        }
    }

    // This will eventually be used
    #[allow(dead_code)]
    pub fn snake_case_headers(&mut self) {
        self.format_headers = false;
    }
}

impl Display for VTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers = self.object.keys().collect_vec();
        let values = self.object.values().collect_vec();

        let contestants = {
            // TODO: Make this dynamic largest header
            let default_width = "Updated".len();

            let mut v = vec![default_width];
            v.extend(headers.iter().map(|s| s.len()));

            v
        };

        let header_lengths: Vec<usize> =
            headers
                .iter()
                .fold(vec![0; headers.len()], |base, element| {
                    // TODO: Simultaneous iterators

                    headers
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let mut contestants = contestants.clone();
                            contestants.push(base[i]);
                            contestants.push(element.len());

                            // Safe as we have just pushed at least two items into the vector in the lines above
                            *unsafe { contestants.iter().max().unwrap_unchecked() }
                        })
                        .collect()
                });

        let iters = headers.iter().zip(values.iter()).enumerate();

        for (i, (header, element)) in iters {
            let header_size = header_lengths[i];

            let header = if self.format_headers {
                Header::new(header).to_string()
            } else {
                (*header).clone()
            };

            let element = if let Some(element) = element.as_str() {
                element.to_owned()
            } else if let Some(array) = element.as_array() {
                array
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map(std::string::ToString::to_string)
                            .unwrap_or_default()
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            } else {
                element.to_string()
            };

            writeln!(f, "{header:header_size$} : {element}")?;
        }

        Ok(())
    }
}
