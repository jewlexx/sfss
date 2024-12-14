//! Output types for the sprinkles library.
//!
//! This module contains the output types for the sprinkles library. These types are used to represent the output of the sprinkles library, and can be used to generate output in various formats.
//!
//! NOTE: These types are not meant to be used directly by the user. They are used internally by the sprinkles library and sfsu to generate output.
//!
pub mod colours;
pub mod consts;
pub mod sectioned;
pub mod structured;
pub mod truncate;

/// Opinionated whitespace for formatting
pub const WHITESPACE: &str = "  ";
