use serde::{Deserialize, Serialize};
use std::fmt;

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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ArithOp {
    Add,
    Minus,
    Times,
    Div,
}

impl fmt::Display for ArithOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithOp::Add => write!(f, "+"),
            ArithOp::Minus => write!(f, "-"),
            ArithOp::Times => write!(f, "*"),
            ArithOp::Div => write!(f, "/"),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Ceil,
    Floor,
    Abs,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Ceil => write!(f, "ceil"),
            UnaryOp::Floor => write!(f, "floor"),
            UnaryOp::Abs => write!(f, "abs"),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AggregateOp {
    Min,
    Max,
    Avg,
}

impl fmt::Display for AggregateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateOp::Min => write!(f, "min"),
            AggregateOp::Max => write!(f, "max"),
            AggregateOp::Avg => write!(f, "avg"),
        }
    }
}
