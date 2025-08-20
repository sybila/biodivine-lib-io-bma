use crate::utils::take_if_not_blank;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod bma_fn_update;

mod bma_update_function;
mod expression_enums;
mod expression_node_data;

pub use bma_update_function::BmaUpdateFunction;
pub use expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
pub use expression_node_data::BmaExpressionNodeData;

mod _impl_from_update_fn;
mod _impl_to_update_fn;
mod expression_token;
mod parser;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Error)]
#[error("Invalid update function `{input_string}`: {error}")]
pub struct InvalidBmaFnUpdate {
    pub error: String,
    pub input_string: String,
}

/// A utility function to correctly parse [`BmaUpdateFunction`], including handling of blank values.
#[must_use] pub fn read_fn_update(
    input: &str,
    variables: &[(u32, String)],
) -> Option<Result<BmaUpdateFunction, InvalidBmaFnUpdate>> {
    let value = take_if_not_blank(input)?;
    Some(
        BmaUpdateFunction::parse_from_str(value.as_str(), variables).map_err(|error| {
            InvalidBmaFnUpdate {
                error,
                input_string: value,
            }
        }),
    )
}
