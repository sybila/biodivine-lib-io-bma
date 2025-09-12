//! Rust library for working with models in BMA format.
#![warn(clippy::pedantic)]
// On top of the pedantic configuration, we do turn off these checks.
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

/// Main `BmaModel` structure and its utilities.
pub(crate) mod model;
/// Internal module that implements serialization via dedicated types for XML and JSON.
pub(crate) mod serde;

/// Structures and utilities for parsing/evaluating update functions.
pub mod update_function;

pub use crate::model::bma_model::{BmaModel, BmaModelError};
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
