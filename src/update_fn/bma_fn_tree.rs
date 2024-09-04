use crate::update_fn::enums::{AggregateOp, ArithOp, Literal, UnaryOp};
use crate::update_fn::tokenizer::BmaFnToken;
use std::cmp;
use std::fmt;

/// Enum of possible node types in a BMA expression syntax tree.
///
/// In particular, a node type can be:
///     - A "terminal" node containing a literal (variable, constant).
///     - A "unary" node with a `UnaryOp` and a sub-expression.
///     - A binary "arithmetic" node, with a `BinaryOp` and two sub-expressions.
///     - An "aggregation" node with a `AggregateOp` op and a list of sub-expressions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Expression {
    Terminal(Literal),
    Unary(UnaryOp, Box<BmaFnNode>),
    Arithmetic(ArithOp, Box<BmaFnNode>, Box<BmaFnNode>),
    Aggregation(AggregateOp, Vec<Box<BmaFnNode>>),
}

/// A single node in a syntax tree of a FOL formula.
///
/// Each node tracks its:
///     - `height`; A positive integer starting from 0 (for term nodes).
///     - `expression_tree`; A parse tree for the expression`.
///     - `function_str`; A canonical string representation of the expression.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BmaFnNode {
    pub function_str: String,
    pub height: u32,
    pub expression_tree: Expression,
}

impl BmaFnNode {
    /// "Parse" a new [BmaFnNode] from a list of [BmaFnToken] objects.
    pub fn from_tokens(_tokens: &[BmaFnToken]) -> Result<BmaFnNode, String> {
        todo!()
    }

    /// Create a "unary" [BmaFnNode] from the given arguments.
    ///
    /// See also [Expression::Unary].
    pub fn mk_unary(child: BmaFnNode, op: UnaryOp) -> BmaFnNode {
        let subform_str = format!("{op}({child})");
        BmaFnNode {
            function_str: subform_str,
            height: child.height + 1,
            expression_tree: Expression::Unary(op, Box::new(child)),
        }
    }

    /// Create a "binary" arithmetic [BmaFnNode] from the given arguments.
    ///
    /// See also [Expression::Binary].
    pub fn mk_arithmetic(left: BmaFnNode, right: BmaFnNode, op: ArithOp) -> BmaFnNode {
        BmaFnNode {
            function_str: format!("({left} {op} {right})"),
            height: cmp::max(left.height, right.height) + 1,
            expression_tree: Expression::Arithmetic(op, Box::new(left), Box::new(right)),
        }
    }

    /// Create a [BmaFnNode] representing a Boolean constant.
    ///
    /// See also [Expression::Terminal] and [Atomic::True] / [Atomic::False].
    pub fn mk_constant(constant_val: i32) -> BmaFnNode {
        Self::mk_literal(Literal::Int(constant_val))
    }

    /// Create a [BmaFnNode] representing a variable.
    ///
    /// See also [Expression::Terminal] and [Literal::Str].
    pub fn mk_variable(var_name: &str) -> BmaFnNode {
        Self::mk_literal(Literal::Str(var_name.to_string()))
    }

    /// A helper function which creates a new [BmaFnNode] for the given [Literal] value.
    fn mk_literal(literal: Literal) -> BmaFnNode {
        BmaFnNode {
            function_str: literal.to_string(),
            height: 0,
            expression_tree: Expression::Terminal(literal),
        }
    }

    /// Create a [BmaFnNode] representing an aggregation operator applied to given arguments.
    pub fn mk_aggregation(op: AggregateOp, inner_nodes: Vec<BmaFnNode>) -> BmaFnNode {
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

        BmaFnNode {
            function_str,
            height: max_height + 1,
            expression_tree: Expression::Aggregation(op, inner_boxed_nodes),
        }
    }
}

impl BmaFnNode {
    pub fn as_str(&self) -> &str {
        self.function_str.as_str()
    }
}

impl fmt::Display for BmaFnNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.function_str)
    }
}
