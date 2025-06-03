//! Rust library for working with models in BMA format.

/// Main `BmaModel` structure and its utilities.
pub(crate) mod model;
/// Structures and utilities for parsing/evaluating update functions.
pub mod update_fn;

/// Intermediate struct `JsonBmaModel` for (de)serializing JSON.
mod json_model;
/// Intermediate struct `XmlBmaModel` for (de)serializing XML.
mod xml_model;

pub use crate::model::bma_model::BmaModel;
pub use crate::model::bma_network::{BmaNetwork, BmaNetworkError};
pub use crate::model::bma_relationship::{BmaRelationship, BmaRelationshipError, RelationshipType};
pub use crate::model::bma_variable::{BmaVariable, BmaVariableError};
pub use crate::model::layout::bma_layout::{BmaLayout, BmaLayoutError};
pub use crate::model::layout::bma_layout_container::{BmaLayoutContainer, BmaLayoutContainerError};
pub use crate::model::layout::bma_layout_variable::{
    BmaLayoutVariable, BmaLayoutVariableError, VariableType,
};

mod validation;
pub use validation::{
    ContextualValidation, ErrorReporter, ReporterWrapper, Validation, VecReporter,
};

pub(crate) mod utils;
