//! Structured output for the CLI

use std::fmt::Display;

use indexmap::IndexMap;
use itertools::Itertools;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::wrappers::header::Header;

use super::{consts::WALL, truncate::FixedLength};

pub mod vertical;

#[must_use = "Structured is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct Structured {
    objects: Vec<Map<String, Value>>,
    max_length: Option<usize>,
}

impl Structured {
    /// Construct a new [`Structured`] formatter
    ///
    /// # Panics
    /// - If the length of headers is not equal to the length of values
    /// - If the values provided are not objects
    pub fn new(values: &[impl Serialize]) -> Self {
        let objects = values
            .iter()
            .map(|v| {
                let value = serde_json::to_value(v).expect("valid value");

                if let Value::Object(object) = value {
                    object
                } else {
                    panic!("Expected object, got {value:?}");
                }
            })
            .collect::<Vec<_>>();

        Structured {
            objects,
            max_length: None,
        }
    }

    /// Add a max length to the [`Structured`] formatter
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);

        self
    }
}

struct Values<'a> {
    header_values: Vec<&'a Value>,
}

impl std::ops::DerefMut for Values<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.header_values
    }
}

impl<'a> std::ops::Deref for Values<'a> {
    type Target = Vec<&'a Value>;

    fn deref(&self) -> &Self::Target {
        &self.header_values
    }
}

impl Values<'_> {
    fn new() -> Self {
        Self {
            header_values: vec![],
        }
    }

    fn max_length(&self) -> usize {
        self.header_values
            .iter()
            .filter_map(|v| Some(v.as_str()?.len()))
            .max()
            .unwrap_or_default()
    }
}

impl Display for Structured {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header_values =
            self.objects
                .iter()
                .fold(IndexMap::<String, Values<'_>>::new(), |mut base, object| {
                    for (k, v) in object {
                        if let Some(values) = base.get_mut(k) {
                            values.header_values.push(v);
                        } else {
                            let mut values = Values::new();
                            values.push(v);
                            base.insert(k.to_string(), values);
                        }
                    }

                    base
                });

        let access_lengths = header_values
            .iter()
            .map(|(header, values)| header.len().max(values.max_length()))
            .collect_vec();

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let evened_access_lengths = {
            let term_columns: f64 = console::Term::stdout().size().1.into();
            let total = access_lengths.iter().sum::<usize>() as f64;
            let percents = access_lengths.iter().map(|s| ((*s) as f64) / total);
            let even_parts = percents.map(|p| (p * term_columns).floor() as usize);

            even_parts.collect::<Vec<_>>()
        };

        let access_lengths = evened_access_lengths;

        // Print Headers
        for (i, (header, _)) in header_values.iter().enumerate() {
            let header_size = access_lengths[i];

            let truncated = FixedLength::new(Header::new(header));
            write!(f, "{truncated:header_size$}{WALL}")?;
        }

        // Enter new row
        writeln!(f)?;

        // Finalise values
        let mut finalised_values = header_values;

        // Print Values
        for _ in 0..=self.objects.len() {
            for (i, (_, values)) in finalised_values.iter_mut().enumerate() {
                let value_size = access_lengths[i];

                let Some(current_value) = values.pop() else {
                    continue;
                };
                let Some(element) = (match current_value {
                    Value::Null => None,
                    Value::Bool(bool) => Some(bool.to_string()),
                    Value::Number(number) => Some(number.to_string()),
                    Value::String(string) => Some(string.to_string()),
                    Value::Array(array) => Some(
                        array
                            .iter()
                            .map(|v| {
                                v.as_str()
                                    .map(std::string::ToString::to_string)
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<String>>()
                            .join(", "),
                    ),
                    Value::Object(_) => panic!("Objects not supported within other objects"),
                }) else {
                    continue;
                };

                let with_suffix = FixedLength::new(element);

                #[cfg(feature = "v2")]
                write!(f, "{with_suffix:value_size$}{WALL}")?;
                #[cfg(not(feature = "v2"))]
                write!(f, "{with_suffix:value_size$}{WALL}")?;
            }

            // Enter new row
            writeln!(f)?;
        }

        Ok(())
    }
}
