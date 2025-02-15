//! Structured output for the CLI

use std::fmt::Display;

use hashbrown::HashMap;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{float::usize::convert_to_f64, wrappers::header::Header};

use super::{consts::WALL, truncate::FixedLength};

pub mod vertical;

#[must_use = "Structured is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct Structured {
    objects: Vec<Map<String, Value>>,
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

        Structured { objects }
    }

    #[inline]
    // TODO: Remove this redundant method
    /// Add a max length to the [`Structured`] formatter
    pub fn with_max_length(self, _max: usize) -> Self {
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
            .map(|(header, values)| (header, header.len().max(values.max_length())))
            .collect_vec();

        let term_columns = console::Term::stdout().size().1;

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        // Number of columns each header has access to in the terminal
        // The index of each column is the index of the header in the indexmap iterator
        let column_lengths = {
            let term_columns: f64 = term_columns.into();

            let total = convert_to_f64(access_lengths.iter().map(|(_, len)| len).sum::<usize>())
                .expect("total length within reasonable range. please report this bug");

            access_lengths
                .iter()
                .fold(HashMap::new(), |mut acc, (header, len)| {
                    let percent = ((*len) as f64) / total;
                    let columns = (percent * term_columns).floor() as usize;

                    acc.entry((*header).to_string()).or_insert(columns);
                    acc
                })
        };

        // Finalise values
        let mut finalised_values = header_values;

        // Print Headers
        for (header, _) in &finalised_values {
            let header_size = column_lengths.get(header).copied().unwrap_or_default();

            let truncated = console::style(FixedLength::new(Header::new(header))).green();
            write!(f, "{truncated:header_size$}{WALL}")?;
        }

        // Enter new row
        writeln!(f)?;

        // Print Values
        for _ in 0..self.objects.len() {
            for (header, values) in &mut finalised_values {
                let value_size = column_lengths.get(header).copied().unwrap_or_default();

                let Some(current_value) = values.pop() else {
                    panic!("ran out of values early. this is a bug.");
                };
                let element = match current_value {
                    Value::Null => String::new(),
                    Value::Bool(bool) => bool.to_string(),
                    Value::Number(number) => number.to_string(),
                    Value::String(string) => string.to_string(),
                    Value::Array(array) => array
                        .iter()
                        .map(|v| v.as_str().unwrap_or("<object>"))
                        .join(", "),

                    Value::Object(_) => panic!("Objects not supported within other objects"),
                };

                let with_suffix = FixedLength::new(element);

                write!(f, "{with_suffix:value_size$}{WALL}")?;
            }

            // Enter new row
            writeln!(f)?;
        }

        Ok(())
    }
}
