use serde::{Deserialize, Serialize};
use thiserror::Error;

/// This is an internal error type for the parsing process. The public API for this is
/// [`InvalidBmaExpression`]. The difference is that this error does
/// not contain the original input string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("Invalid expression: {message} at position `{position}`")]
pub(crate) struct ParserError {
    pub position: usize,
    pub message: String,
}

impl ParserError {
    pub fn at(position: usize, error_type: String) -> ParserError {
        ParserError {
            position,
            message: error_type,
        }
    }
}

/// An error raised when an update function expression is invalid and cannot be parsed correctly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Error)]
#[error("Invalid expression `{expression}`: {message} at position `{position}`")]
pub struct InvalidBmaExpression {
    pub expression: String,
    pub position: usize,
    pub message: String,
}

impl InvalidBmaExpression {
    pub(crate) fn from_parser_error(error: ParserError, expression: String) -> Self {
        InvalidBmaExpression {
            expression,
            position: error.position,
            message: error.message,
        }
    }
}
