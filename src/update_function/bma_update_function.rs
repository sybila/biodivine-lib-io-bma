use crate::update_function::{AggregateFn, ArithOp, BmaExpressionNodeData, Literal, UnaryFn};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// A wrapper type that stores [`BmaExpressionNodeData`] using an atomic reference counter
/// such that it can be safely cloned without data duplication, or shared between threads
/// (e.g. when using Python/JavaScript bindings).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BmaUpdateFunction(Arc<BmaExpressionNodeData>);

/// Utility data access.
impl BmaUpdateFunction {
    /// Get a reference to the underlying [`BmaExpressionNodeData`].
    #[must_use]
    pub fn as_data(&self) -> &BmaExpressionNodeData {
        self.0.as_ref()
    }

    /// Build a string representation of this update function compatible with BMA format.
    #[must_use]
    pub fn as_bma_string(&self) -> String {
        self.to_string()
    }
}

/// Utility constructors
impl BmaUpdateFunction {
    /// Create a "unary" [`BmaUpdateFunction`] from the given arguments.
    ///
    /// See also [`BmaExpressionNodeData::Unary`].
    #[must_use]
    pub fn mk_unary(op: UnaryFn, child: &BmaUpdateFunction) -> BmaUpdateFunction {
        BmaExpressionNodeData::Unary(op, child.clone()).into()
    }

    /// Create a "binary" arithmetic [`BmaUpdateFunction`] from the given arguments.
    ///
    /// See also [`BmaExpressionNodeData::Arithmetic`].
    #[must_use]
    pub fn mk_arithmetic(
        op: ArithOp,
        left: &BmaUpdateFunction,
        right: &BmaUpdateFunction,
    ) -> BmaUpdateFunction {
        BmaExpressionNodeData::Arithmetic(op, left.clone(), right.clone()).into()
    }

    /// Create a [`BmaUpdateFunction`] representing a Boolean constant.
    ///
    /// See also [`BmaExpressionNodeData::Terminal`] and [`Literal::Const`].
    #[must_use]
    pub fn mk_constant(constant_val: i32) -> BmaUpdateFunction {
        BmaExpressionNodeData::Terminal(Literal::Const(constant_val)).into()
    }

    /// Create a [`BmaUpdateFunction`] representing a variable (using an ID).
    ///
    /// See also [`BmaExpressionNodeData::Terminal`] and [`Literal::Var`].
    #[must_use]
    pub fn mk_variable(var_id: u32) -> BmaUpdateFunction {
        BmaExpressionNodeData::Terminal(Literal::Var(var_id)).into()
    }

    /// Create a [`BmaUpdateFunction`] representing an aggregation operator
    /// applied to given arguments.
    #[must_use]
    pub fn mk_aggregation(op: AggregateFn, inner_nodes: &[BmaUpdateFunction]) -> BmaUpdateFunction {
        BmaExpressionNodeData::Aggregation(op, inner_nodes.to_vec()).into()
    }
}

impl AsRef<BmaExpressionNodeData> for BmaUpdateFunction {
    fn as_ref(&self) -> &BmaExpressionNodeData {
        self.as_data()
    }
}

impl From<BmaExpressionNodeData> for BmaUpdateFunction {
    fn from(value: BmaExpressionNodeData) -> Self {
        BmaUpdateFunction(Arc::new(value))
    }
}

impl Display for BmaUpdateFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.as_data() {
            BmaExpressionNodeData::Terminal(literal) => {
                write!(f, "{literal}")
            }
            BmaExpressionNodeData::Unary(op, arg) => {
                write!(f, "{op}({arg})")
            }
            BmaExpressionNodeData::Arithmetic(op, arg1, arg2) => {
                write!(f, "({arg1} {op} {arg2})")
            }
            BmaExpressionNodeData::Aggregation(op, args) => {
                write!(f, "{op}(")?;
                if let Some(first) = args.first() {
                    write!(f, "{first}")?;
                }
                for arg in args.iter().skip(1) {
                    write!(f, ", {arg}")?;
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}
