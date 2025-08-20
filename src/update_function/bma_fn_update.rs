use crate::update_function::expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
use crate::update_function::parser::parse_bma_fn_tokens;
use crate::update_function::tokenizer::BmaFnToken;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp;
use std::fmt;

use super::parser::parse_bma_formula;

/// Enum of possible node types in a BMA expression syntax tree.
///
/// In particular, a node type can be:
///     - A "terminal" node containing a literal (variable, constant).
///     - A "unary" node with a `UnaryFn` and a sub-expression.
///     - A binary "arithmetic" node, with a `BinaryOp` and two sub-expressions.
///     - An "aggregation" node with a `AggregateFn` op and a list of sub-expressions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BmaUpdateFunctionNode {
    Terminal(Literal),
    Unary(UnaryFn, Box<BmaUpdateFunction>),
    Arithmetic(ArithOp, Box<BmaUpdateFunction>, Box<BmaUpdateFunction>),
    Aggregation(AggregateFn, Vec<Box<BmaUpdateFunction>>),
}

/// A single node in a syntax tree of a BMA update function's expression.
///
/// Each node tracks its:
///     - `height`; A positive integer starting from 0 (for term nodes).
///     - `expression_tree`; A parse tree for the expression`.
///     - `function_str`; A canonical string representation of the expression.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BmaUpdateFunction {
    pub function_str: String,
    pub height: u32,
    pub expression_tree: BmaUpdateFunctionNode,
}

impl Serialize for BmaUpdateFunction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for BmaUpdateFunction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match BmaUpdateFunction::parse_from_str(&value, &[]) {
            Ok(tree) => Ok(tree),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

impl BmaUpdateFunction {
    /// "Parse" new [BmaUpdateFunction] tree from a list of [BmaFnToken] objects.
    pub fn from_tokens(tokens: &[BmaFnToken]) -> Result<BmaUpdateFunction, String> {
        parse_bma_fn_tokens(tokens)
    }

    /// Parse new [BmaUpdateFunction] tree directly from a string representation.
    ///
    /// Arg `variables` is a map of variable IDs to their names. It is needed because there are
    /// some weird format differences between different variants, and a variable can be referenced
    /// by either its ID or its name. We convert everything to IDs for easier processing.
    pub fn parse_from_str(
        function_str: &str,
        variables: &[(u32, String)],
    ) -> Result<BmaUpdateFunction, String> {
        parse_bma_formula(function_str, variables)
    }

    /// Create a "unary" [BmaUpdateFunction] from the given arguments.
    ///
    /// See also [BmaUpdateFunctionNode::Unary].
    pub fn mk_unary(child: BmaUpdateFunction, op: UnaryFn) -> BmaUpdateFunction {
        let subform_str = format!("{op}({child})");
        BmaUpdateFunction {
            function_str: subform_str,
            height: child.height + 1,
            expression_tree: BmaUpdateFunctionNode::Unary(op, Box::new(child)),
        }
    }

    /// Create a "binary" arithmetic [BmaUpdateFunction] from the given arguments.
    ///
    /// See also [BmaUpdateFunctionNode::Arithmetic].
    pub fn mk_arithmetic(
        left: BmaUpdateFunction,
        right: BmaUpdateFunction,
        op: ArithOp,
    ) -> BmaUpdateFunction {
        BmaUpdateFunction {
            function_str: format!("({left} {op} {right})"),
            height: cmp::max(left.height, right.height) + 1,
            expression_tree: BmaUpdateFunctionNode::Arithmetic(op, Box::new(left), Box::new(right)),
        }
    }

    /// Create a [BmaUpdateFunction] representing a Boolean constant.
    ///
    /// See also [BmaUpdateFunctionNode::Terminal] and [Literal::Const].
    pub fn mk_constant(constant_val: i32) -> BmaUpdateFunction {
        Self::mk_literal(Literal::Const(constant_val))
    }

    /// Create a [BmaUpdateFunction] representing a variable.
    ///
    /// See also [BmaUpdateFunctionNode::Terminal] and [Literal::Var].
    pub fn mk_variable(var_id: u32) -> BmaUpdateFunction {
        Self::mk_literal(Literal::Var(var_id))
    }

    /// A helper function which creates a new [BmaUpdateFunction] for the given [Literal] value.
    fn mk_literal(literal: Literal) -> BmaUpdateFunction {
        BmaUpdateFunction {
            function_str: literal.to_string(),
            height: 0,
            expression_tree: BmaUpdateFunctionNode::Terminal(literal),
        }
    }

    /// Create a [BmaUpdateFunction] representing an aggregation operator applied to given arguments.
    pub fn mk_aggregation(
        op: AggregateFn,
        inner_nodes: Vec<BmaUpdateFunction>,
    ) -> BmaUpdateFunction {
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

        BmaUpdateFunction {
            function_str,
            height: max_height + 1,
            expression_tree: BmaUpdateFunctionNode::Aggregation(op, inner_boxed_nodes),
        }
    }
}

impl BmaUpdateFunction {
    pub fn as_str(&self) -> &str {
        self.function_str.as_str()
    }
}

impl fmt::Display for BmaUpdateFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.function_str)
    }
}
