use std::fmt::Display;

use rayon::prelude::*;
use serde_json::Value;

#[derive(Debug)]
#[must_use = "OptionalTruncate is lazy, and only takes effect when used in formatting"]
pub struct OptionalTruncate<T> {
    data: T,
    length: Option<usize>,
    suffix: Option<&'static str>,
}

impl<T> OptionalTruncate<T> {
    /// Construct a new [`OptionalTruncate`] from the provided data
    pub fn new(data: T) -> Self {
        Self {
            data,
            length: None,
            suffix: None,
        }
    }

    // Generally length would not be passed as an option,
    // but given we are just forwarding what is passed to `Structured`,
    // it works better here
    pub fn with_length(mut self, length: Option<usize>) -> Self {
        self.length = length;

        self
    }

    pub fn with_suffix(mut self, suffix: Option<&'static str>) -> Self {
        self.suffix = suffix;

        self
    }
}

impl<T: Display> Display for OptionalTruncate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(length) = self.length {
            use quork::truncate::Truncate;

            let mut truncation = Truncate::new(&self.data, length);

            if let Some(ref suffix) = self.suffix {
                truncation = truncation.with_suffix(suffix);
            }

            truncation.fmt(f)
        } else {
            self.data.fmt(f)
        }
    }
}

#[must_use = "Structured is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct Structured<'a> {
    headers: &'a [&'a str],
    values: &'a [Value],
    max_length: Option<usize>,
}

impl<'a> Structured<'a> {
    /// Construct a new [`Structured`] formatter
    ///
    /// # Panics
    /// - If the length of headers is not equal to the length of values
    /// - If the values provided are not objects
    pub fn new(headers: &'a [&'a str], values: &'a [Value]) -> Self {
        assert_eq!(
            headers.len(),
            // TODO: Do not panic here
            values[0].as_object().unwrap().keys().len(),
            "The number of column headers must match quantity data for said columns"
        );
        Structured {
            headers,
            values,
            max_length: None,
        }
    }

    /// Add a max length to the [`Structured`] formatter
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);

        self
    }
}

impl<'a> Display for Structured<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = {
            let mut data = vec![];
            self.values
                .par_iter()
                // TODO: Do not panic here
                .map(|row| row.as_object().expect("object"))
                .collect_into_vec(&mut data);

            data
        };

        let contestants = {
            // TODO: Make this dynamic largest header
            let default_width = "Updated".len();

            let mut v = vec![default_width];
            v.extend(self.headers.iter().map(|s| s.len()));

            v
        };

        // TODO: Imeplement max length with truncation
        let access_lengths: Vec<usize> =
            data.iter().fold(vec![0; self.headers.len()], |base, row| {
                // TODO: Simultaneous iterators

                self.headers
                    .iter()
                    .enumerate()
                    .map(|(i, header)| {
                        let element = row
                            .get(*header)
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();

                        let mut contestants = contestants.clone();
                        contestants.push(base[i]);
                        contestants.push(
                            OptionalTruncate::new(element)
                                .with_length(self.max_length)
                                // TODO: Fix suffix
                                .with_suffix(Some("..."))
                                .to_string()
                                .len(),
                        );

                        *contestants.iter().max().unwrap()
                    })
                    .collect()
            });

        for (i, header) in self.headers.iter().enumerate() {
            let header_size = access_lengths[i];

            let truncated = OptionalTruncate::new(header).with_length(self.max_length);
            write!(f, "{truncated:header_size$} | ")?;
        }

        // Enter new row
        writeln!(f)?;

        for row in &data {
            for (i, header) in self.headers.iter().enumerate() {
                let value_size = access_lengths[i];
                let element = row
                    .get(*header)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                let truncated = OptionalTruncate::new(element).with_length(self.max_length);

                write!(f, "{truncated:value_size$} | ")?;
            }

            // Enter new row
            writeln!(f)?;
        }

        Ok(())
    }
}
