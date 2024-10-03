//! Rust library for working with models in BMA format.

/// Intermediate struct `JsonBmaModel` for (de)serializing JSON.
pub mod json_model;
/// Main `BmaModel` structure and its utilities.
pub mod model;
/// Few traits used throughout library (for simpler serialization and so on).
pub mod traits;
/// Structures and utilities for parsing/evaluating update functions.
pub mod update_fn;
/// Intermediate struct `XmlBmaModel` for (de)serializing XML.
pub mod xml_model;

mod enums;
