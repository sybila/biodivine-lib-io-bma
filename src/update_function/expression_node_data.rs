use crate::update_function::{AggregateFn, ArithOp, BmaUpdateFunction, Literal, UnaryFn};

/// Enum of possible node types in a BMA expression syntax tree.
///
/// In particular, a node type can be:
///     - A "terminal" node containing a literal (variable, constant).
///     - A "unary" node with a [`UnaryFn`] and a sub-expression.
///     - A binary "arithmetic" node, with a [`ArithOp`] and two sub-expressions.
///     - An "aggregation" node with an [`AggregateFn`] op and a list of sub-expressions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BmaExpressionNodeData {
    Terminal(Literal),
    Unary(UnaryFn, BmaUpdateFunction),
    Arithmetic(ArithOp, BmaUpdateFunction, BmaUpdateFunction),
    Aggregation(AggregateFn, Vec<BmaUpdateFunction>),
}
