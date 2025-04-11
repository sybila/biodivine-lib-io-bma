use serde::{Deserialize, Serialize};
use std::fmt;

/// An atomic expression that can be either an integer or a string (variable name).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i32),
    Str(String),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(value) => write!(f, "{}", value),
            Literal::Str(value) => write!(f, "{}", value),
        }
    }
}

/// Arithmetic operations admissible in BMA function expressions.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ArithOp {
    Plus,
    Minus,
    Mult,
    Div,
}

impl fmt::Display for ArithOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithOp::Plus => write!(f, "+"),
            ArithOp::Minus => write!(f, "-"),
            ArithOp::Mult => write!(f, "*"),
            ArithOp::Div => write!(f, "/"),
        }
    }
}

/// Unary functions admissible in BMA function expressions.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum UnaryFn {
    Ceil,
    Floor,
    Abs,
}

impl fmt::Display for UnaryFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryFn::Ceil => write!(f, "ceil"),
            UnaryFn::Floor => write!(f, "floor"),
            UnaryFn::Abs => write!(f, "abs"),
        }
    }
}

/// Aggregate functions admissible in BMA function expressions.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AggregateFn {
    Min,
    Max,
    Avg,
}

impl fmt::Display for AggregateFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateFn::Min => write!(f, "min"),
            AggregateFn::Max => write!(f, "max"),
            AggregateFn::Avg => write!(f, "avg"),
        }
    }
}
