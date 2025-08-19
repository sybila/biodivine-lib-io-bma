use crate::update_fn::bma_fn_update::BmaFnUpdate;
use crate::utils::take_if_not_blank;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod bma_fn_update;
pub mod expression_enums;

mod _impl_from_update_fn;
mod _impl_to_update_fn;
mod parser;
mod tokenizer;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Error)]
#[error("Invalid update function `{input_string}`: {error}")]
pub struct InvalidBmaFnUpdate {
    pub error: String,
    pub input_string: String,
}

/// A utility function to correctly parse [`BmaFnUpdate`], including handling of blank values.
pub fn read_fn_update(
    input: &str,
    variables: &[(u32, String)],
) -> Option<Result<BmaFnUpdate, InvalidBmaFnUpdate>> {
    let value = take_if_not_blank(input)?;
    Some(
        BmaFnUpdate::parse_from_str(value.as_str(), variables).map_err(|error| {
            InvalidBmaFnUpdate {
                error,
                input_string: value,
            }
        }),
    )
}
