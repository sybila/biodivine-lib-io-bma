use crate::update_fn::enums::{AggregateFn, ArithOp, Literal, UnaryFn};
use crate::update_fn::parser::parse_bma_fn_tokens;
use crate::update_fn::tokenizer::BmaFnToken;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::fmt;

/// Enum of possible node types in a BMA expression syntax tree.
///
/// In particular, a node type can be:
///     - A "terminal" node containing a literal (variable, constant).
///     - A "unary" node with a `UnaryFn` and a sub-expression.
///     - A binary "arithmetic" node, with a `BinaryOp` and two sub-expressions.
///     - An "aggregation" node with a `AggregateFn` op and a list of sub-expressions.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Terminal(Literal),
    Unary(UnaryFn, Box<BmaFnUpdate>),
    Arithmetic(ArithOp, Box<BmaFnUpdate>, Box<BmaFnUpdate>),
    Aggregation(AggregateFn, Vec<Box<BmaFnUpdate>>),
}

/// A single node in a syntax tree of a FOL formula.
///
/// Each node tracks its:
///     - `height`; A positive integer starting from 0 (for term nodes).
///     - `expression_tree`; A parse tree for the expression`.
///     - `function_str`; A canonical string representation of the expression.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BmaFnUpdate {
    pub function_str: String,
    pub height: u32,
    pub expression_tree: Expression,
}

impl BmaFnUpdate {
    /// "Parse" a new [BmaFnUpdate] from a list of [BmaFnToken] objects.
    pub fn from_tokens(tokens: &[BmaFnToken]) -> Result<BmaFnUpdate, String> {
        parse_bma_fn_tokens(tokens)
    }

    /// Create a "unary" [BmaFnUpdate] from the given arguments.
    ///
    /// See also [Expression::Unary].
    pub fn mk_unary(child: BmaFnUpdate, op: UnaryFn) -> BmaFnUpdate {
        let subform_str = format!("{op}({child})");
        BmaFnUpdate {
            function_str: subform_str,
            height: child.height + 1,
            expression_tree: Expression::Unary(op, Box::new(child)),
        }
    }

    /// Create a "binary" arithmetic [BmaFnUpdate] from the given arguments.
    ///
    /// See also [Expression::Binary].
    pub fn mk_arithmetic(left: BmaFnUpdate, right: BmaFnUpdate, op: ArithOp) -> BmaFnUpdate {
        BmaFnUpdate {
            function_str: format!("({left} {op} {right})"),
            height: cmp::max(left.height, right.height) + 1,
            expression_tree: Expression::Arithmetic(op, Box::new(left), Box::new(right)),
        }
    }

    /// Create a [BmaFnUpdate] representing a Boolean constant.
    ///
    /// See also [Expression::Terminal] and [Atomic::True] / [Atomic::False].
    pub fn mk_constant(constant_val: i32) -> BmaFnUpdate {
        Self::mk_literal(Literal::Int(constant_val))
    }

    /// Create a [BmaFnUpdate] representing a variable.
    ///
    /// See also [Expression::Terminal] and [Literal::Str].
    pub fn mk_variable(var_name: &str) -> BmaFnUpdate {
        Self::mk_literal(Literal::Str(var_name.to_string()))
    }

    /// A helper function which creates a new [BmaFnUpdate] for the given [Literal] value.
    fn mk_literal(literal: Literal) -> BmaFnUpdate {
        BmaFnUpdate {
            function_str: literal.to_string(),
            height: 0,
            expression_tree: Expression::Terminal(literal),
        }
    }

    /// Create a [BmaFnUpdate] representing an aggregation operator applied to given arguments.
    pub fn mk_aggregation(op: AggregateFn, inner_nodes: Vec<BmaFnUpdate>) -> BmaFnUpdate {
        let max_height = inner_nodes
            .iter()
            .map(|node| node.height)
            .max()
            .unwrap_or(0);
        let child_expressions: Vec<String> = inner_nodes
            .iter()
            .map(|child| child.function_str.clone())
            .collect();
        let args_str = child_expressions.join(", ");
        let function_str = format!("{}({})", op, args_str);

        let inner_boxed_nodes = inner_nodes.into_iter().map(Box::new).collect();

        BmaFnUpdate {
            function_str,
            height: max_height + 1,
            expression_tree: Expression::Aggregation(op, inner_boxed_nodes),
        }
    }
}

impl BmaFnUpdate {
    pub fn as_str(&self) -> &str {
        self.function_str.as_str()
    }
}

impl fmt::Display for BmaFnUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.function_str)
    }
}
