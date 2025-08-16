use crate::update_fn::bma_fn_update::BmaFnUpdate;
use crate::utils::take_if_not_blank;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use thiserror::Error;

pub mod bma_fn_update;
pub mod expression_enums;

mod _impl_from_update_fn;
mod _impl_to_update_fn;
mod parser;
mod tokenizer;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Error)]
pub struct InvalidBmaFnUpdate {
    pub input_string: String,
}

impl Display for InvalidBmaFnUpdate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvalidBmaFnUpdate({})", self.input_string)
    }
}

/// A utility function to correctly parse [`BmaFnUpdate`], including handling of blank values.
pub fn read_fn_update(
    input: &str,
    variables: &HashMap<u32, String>,
) -> Option<Result<BmaFnUpdate, InvalidBmaFnUpdate>> {
    let value = take_if_not_blank(input)?;
    Some(
        BmaFnUpdate::parse_from_str(value.as_str(), variables).map_err(|e| InvalidBmaFnUpdate {
            input_string: e,
        }),
    )
}
