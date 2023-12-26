use std::fmt::Display;

use crate::{output::wrappers::cap_str::CapitalizedStr, SimIter};

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
    // but given we are just forwarding what is passed to `VTable`,
    // it works better here
    pub fn with_length(self, length: Option<usize>) -> Self {
        Self { length, ..self }
    }

    pub fn with_suffix(self, suffix: &'static str) -> Self {
        Self {
            suffix: Some(suffix),
            ..self
        }
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

            truncation.to_string();

            truncation.fmt(f)
        } else {
            self.data.fmt(f)
        }
    }
}

#[must_use = "VTable is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct VTable<'a, H, V> {
    headers: &'a [H],
    values: &'a [V],
    max_length: Option<usize>,
}

impl<'a, H: Display, V: Display + Send + Sync> VTable<'a, H, V> {
    /// Construct a new [`VTable`] formatter
    ///
    /// # Panics
    /// - If the length of headers is not equal to the length of values
    /// - If the values provided are not objects
    pub fn new(headers: &'a [H], values: &'a [V]) -> Self {
        assert_eq!(
            headers.len(),
            // TODO: Do not panic here
            values.len(),
            "The number of column headers must match quantity data for said columns"
        );
        Self {
            headers,
            values,
            max_length: None,
        }
    }

    /// Add a max length to the [`VTable`] formatter
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);

        self
    }
}

impl<'a, H: Display, V: Display + Send + Sync> Display for VTable<'a, H, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let contestants = {
            // TODO: Make this dynamic largest header
            let default_width = "Updated".len();

            let mut v = vec![default_width];
            v.extend(self.headers.iter().map(|s| s.to_string().len()));

            v
        };

        let header_lengths: Vec<usize> =
            self.headers
                .iter()
                .fold(vec![0; self.headers.len()], |base, element| {
                    // TODO: Simultaneous iterators

                    self.headers
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let mut contestants = contestants.clone();
                            contestants.push(base[i]);
                            contestants.push(
                                OptionalTruncate::new(element)
                                    .with_length(self.max_length)
                                    // TODO: Fix suffix
                                    .with_suffix("...")
                                    .to_string()
                                    .len(),
                            );

                            *contestants.iter().max().unwrap()
                        })
                        .collect()
                });

        let iters = SimIter(self.headers.iter(), self.values.iter()).enumerate();

        for (i, (header, element)) in iters {
            let header_size = header_lengths[i];

            let truncated = OptionalTruncate::new(CapitalizedStr::new(header).to_string())
                .with_length(self.max_length);

            let element = element.to_string();

            writeln!(f, "{truncated:header_size$} : {element}")?;
        }

        Ok(())
    }
}
