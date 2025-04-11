//! Rust library for working with models in BMA format.

/// Main `BmaModel` structure and its utilities.
pub mod model;
/// Structures and utilities for parsing/evaluating update functions.
pub mod update_fn;

/// Enums for BMA variable and relationship types.
mod enums;
/// Intermediate struct `JsonBmaModel` for (de)serializing JSON.
mod json_model;
/// Intermediate struct `XmlBmaModel` for (de)serializing XML.
mod xml_model;
