use serde::{Deserialize, Serialize};
use std::fmt;

/// An atomic expression that can be either an integer or a variable.
///
/// There are some weird format differences, and a variable can be referenced by
/// either its ID or its name. We convert everything to IDs for easier processing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Const(i32),
    Var(u32),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Const(value) => write!(f, "{value}"),
            Literal::Var(value) => write!(f, "var({value})"),
        }
    }
}

/// Arithmetic operations admissible in BMA function expressions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
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

impl TryFrom<char> for ArithOp {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(ArithOp::Plus),
            '-' => Ok(ArithOp::Minus),
            '*' => Ok(ArithOp::Mult),
            '/' => Ok(ArithOp::Div),
            _ => Err(()),
        }
    }
}

/// Unary functions admissible in BMA function expressions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum UnaryFn {
    Ceil,
    Floor,
    Abs,
    Neg, // Unary minus operator (negation)
}

impl fmt::Display for UnaryFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryFn::Ceil => write!(f, "ceil"),
            UnaryFn::Floor => write!(f, "floor"),
            UnaryFn::Abs => write!(f, "abs"),
            UnaryFn::Neg => write!(f, "-"),
        }
    }
}

impl TryFrom<&str> for UnaryFn {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ceil" => Ok(UnaryFn::Ceil),
            "floor" => Ok(UnaryFn::Floor),
            "abs" => Ok(UnaryFn::Abs),
            _ => Err(()),
        }
    }
}

/// Aggregate functions admissible in BMA function expressions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AggregateFn {
    Min,
    Max,
    Avg,
}

impl TryFrom<&str> for AggregateFn {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "min" => Ok(AggregateFn::Min),
            "max" => Ok(AggregateFn::Max),
            "avg" => Ok(AggregateFn::Avg),
            _ => Err(()),
        }
    }
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
