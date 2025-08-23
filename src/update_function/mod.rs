use crate::utils::take_if_not_blank;

pub mod bma_fn_update;

mod bma_update_function;
mod expression_enums;
mod expression_node_data;

mod _impl_from_update_fn;
mod _impl_to_update_fn;
mod bma_update_function_error;
mod expression_token;
mod parser;

pub use bma_update_function::BmaUpdateFunction;
pub use expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
pub use expression_node_data::BmaExpressionNodeData;

pub use bma_update_function_error::InvalidBmaUpdateFunction;
pub(crate) use bma_update_function_error::ParserError;

/// A utility function to correctly parse [`BmaUpdateFunction`], including handling of blank values.
#[must_use]
pub fn read_fn_update(
    input: &str,
    variables: &[(u32, String)],
) -> Option<Result<BmaUpdateFunction, InvalidBmaUpdateFunction>> {
    let value = take_if_not_blank(input)?;
    Some(
        BmaUpdateFunction::parse_from_str(value.as_str(), variables)
            .map_err(|error| InvalidBmaUpdateFunction::from_parser_error(error, input.to_owned())),
    )
}
