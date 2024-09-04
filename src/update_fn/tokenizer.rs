use crate::update_fn::enums::{AggregateOp, ArithOp, Literal, UnaryOp};

/// Enum of all possible tokens occurring in a BMA function string.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BmaFnToken {
    Atomic(Literal),
    Unary(UnaryOp),
    Binary(ArithOp),
    Aggregate(AggregateOp, Vec<BmaFnToken>),
    TokenList(Vec<BmaFnToken>),
}

// todo
