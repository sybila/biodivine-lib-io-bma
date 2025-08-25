use crate::update_function::expression_enums::{ArithOp, Literal};
use crate::update_function::expression_token::{BmaToken, BmaTokenData, try_tokenize_bma_formula};
use crate::update_function::{BmaUpdateFunction, ParserError};
use BmaTokenData::{Aggregate, Atomic, Binary, TokenList, Unary};

// TODO: This should probably be a method
/// Parse an BMA update function formula string representation into an actual expression tree.
/// Basically a wrapper for tokenize+parse (used often for testing/debug purposes).
///
/// Arg `variables` is a map of variable IDs to their names. It is needed because there are
/// some weird format differences between different variants, and a variable can be referenced
/// by either its ID or its name. We convert everything to IDs for easier processing.
pub fn parse_bma_formula(
    formula: &str,
    variable_id_hint: &[(u32, String)],
) -> Result<BmaUpdateFunction, ParserError> {
    let tokens = try_tokenize_bma_formula(formula, variable_id_hint)?;
    let tree = parse_bma_fn_tokens(&tokens)?;
    Ok(tree)
}

/// A utility function that allows us to properly handle empty token list errors.
fn before_or_empty<F: Fn(&[BmaToken]) -> Result<BmaUpdateFunction, ParserError>>(
    op: F,
    split_at: usize,
    tokens: &[BmaToken],
) -> Result<BmaUpdateFunction, ParserError> {
    let slice = &tokens[..split_at];
    if slice.is_empty() {
        let message = format!(
            "Found nothing at the left-hand-side of operator `{}`",
            tokens[split_at]
        );
        Err(ParserError::at(tokens[split_at].position, message))
    } else {
        op(slice)
    }
}

fn after_or_empty<F: Fn(&[BmaToken]) -> Result<BmaUpdateFunction, ParserError>>(
    op: F,
    split_at: usize,
    tokens: &[BmaToken],
) -> Result<BmaUpdateFunction, ParserError> {
    let slice = &tokens[(split_at + 1)..];
    if slice.is_empty() {
        let message = format!(
            "Found nothing at the right-hand-side of operator `{}`",
            tokens[split_at]
        );
        Err(ParserError::at(tokens[split_at].position, message))
    } else {
        op(slice)
    }
}

/// Parse `tokens` of BMA update fn formula into an abstract syntax tree using recursive steps.
pub fn parse_bma_fn_tokens(tokens: &[BmaToken]) -> Result<BmaUpdateFunction, ParserError> {
    if tokens.is_empty() {
        Err(ParserError::at(0, "Expression is empty".to_string()))
    } else {
        parse_1_add_sub(tokens)
    }
}

/// Recursive parsing step 1: extract `-` and `+` operators.
fn parse_1_add_sub(tokens: &[BmaToken]) -> Result<BmaUpdateFunction, ParserError> {
    let split_at = tokens.iter().rposition(|t| {
        matches!(t.data, Binary(ArithOp::Plus)) || matches!(t.data, Binary(ArithOp::Minus))
    });
    if let Some(split_at) = split_at {
        let Binary(op) = &tokens[split_at].data else {
            unreachable!("Parser invariant: split token must be binary.")
        };
        Ok(BmaUpdateFunction::mk_arithmetic(
            *op,
            &before_or_empty(parse_1_add_sub, split_at, tokens)?,
            &after_or_empty(parse_2_div_mul, split_at, tokens)?,
        ))
    } else {
        parse_2_div_mul(tokens)
    }
}

/// Recursive parsing step 2: extract `/` and `*` operators.
fn parse_2_div_mul(tokens: &[BmaToken]) -> Result<BmaUpdateFunction, ParserError> {
    let split_at = tokens.iter().rposition(|t| {
        matches!(t.data, Binary(ArithOp::Div)) || matches!(t.data, Binary(ArithOp::Mult))
    });
    if let Some(split_at) = split_at {
        let Binary(op) = &tokens[split_at].data else {
            unreachable!("Parser invariant: split token must be binary.")
        };
        Ok(BmaUpdateFunction::mk_arithmetic(
            *op,
            &before_or_empty(parse_2_div_mul, split_at, tokens)?,
            &after_or_empty(parse_3_others, split_at, tokens)?,
        ))
    } else {
        parse_3_others(tokens)
    }
}

/// Recursive parsing step 5: extract literals and recursively solve sub-formulae in parentheses
/// and in functions.
fn parse_3_others(tokens: &[BmaToken]) -> Result<BmaUpdateFunction, ParserError> {
    match tokens.len() {
        0 => unreachable!("Parser invariant: Empty tokens are resolved."),
        // This should be named (var/function) or a parenthesis group, anything
        // else does not make sense.
        1 => match &tokens[0].data {
            Binary(_) => unreachable!("Parser invariant: Binary operators are resolved."),
            Atomic(Literal::Var(id)) => Ok(BmaUpdateFunction::mk_variable(*id)),
            Atomic(Literal::Const(num)) => Ok(BmaUpdateFunction::mk_constant(*num)),
            Aggregate(op, arguments) => {
                let mut arg_expressions = Vec::new();
                for inner in arguments {
                    let TokenList(inner_tokens) = &inner.data else {
                        unreachable!("Tokenizer invariant: Function arguments are token lists.")
                    };

                    arg_expressions.push(parse_bma_fn_tokens(inner_tokens)?);
                }
                Ok(BmaUpdateFunction::mk_aggregation(*op, &arg_expressions))
            }
            Unary(op, argument) => {
                let TokenList(inner_tokens) = &argument.data else {
                    unreachable!("Tokenizer invariant: Function arguments are token lists.")
                };
                let arg_expression = parse_bma_fn_tokens(inner_tokens)?;
                Ok(BmaUpdateFunction::mk_unary(*op, &arg_expression))
            }
            // recursively solve sub-formulae in parentheses
            TokenList(inner_tokens) => parse_bma_fn_tokens(inner_tokens),
        },
        _ => {
            let token_str = tokens.iter().map(ToString::to_string).collect::<Vec<_>>();
            let token_str = token_str.join(" ");
            Err(ParserError::at(
                tokens[1].position,
                format!(
                    "Unexpected: `{token_str}`. Expecting atomic proposition, function call, or parenthesis group"
                ),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update_function::{AggregateFn, ArithOp, BmaUpdateFunction, UnaryFn};

    #[test]
    fn test_parse_simple_addition() {
        let input = "3 + 5";
        let result = parse_bma_formula(input, &[]);
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Plus,
            &BmaUpdateFunction::mk_constant(3),
            &BmaUpdateFunction::mk_constant(5),
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_simple_subtraction() {
        let input = "10 - 7";
        let result = parse_bma_formula(input, &[]);
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Minus,
            &BmaUpdateFunction::mk_constant(10),
            &BmaUpdateFunction::mk_constant(7),
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_multiplication_and_division() {
        let input = "8 * 4 / 2";
        let result = parse_bma_formula(input, &[]);
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Div,
            &BmaUpdateFunction::mk_arithmetic(
                ArithOp::Mult,
                &BmaUpdateFunction::mk_constant(8),
                &BmaUpdateFunction::mk_constant(4),
            ),
            &BmaUpdateFunction::mk_constant(2),
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_nested_arithmetic() {
        let input = "3 + (5 * 2)";
        let result = parse_bma_formula(input, &[]);
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Plus,
            &BmaUpdateFunction::mk_constant(3),
            &BmaUpdateFunction::mk_arithmetic(
                ArithOp::Mult,
                &BmaUpdateFunction::mk_constant(5),
                &BmaUpdateFunction::mk_constant(2),
            ),
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_abs_function() {
        let input = "abs(5)";
        let result = parse_bma_formula(input, &[]);
        let expected =
            BmaUpdateFunction::mk_unary(UnaryFn::Abs, &BmaUpdateFunction::mk_constant(5));
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_aggregate_min() {
        let input = "min(3, 5, 5 + var(a))";
        let vars = vec![(42, "a".to_string())];
        let result = parse_bma_formula(input, &vars);
        let expected = BmaUpdateFunction::mk_aggregation(
            AggregateFn::Min,
            &[
                BmaUpdateFunction::mk_constant(3),
                BmaUpdateFunction::mk_constant(5),
                BmaUpdateFunction::mk_arithmetic(
                    ArithOp::Plus,
                    &BmaUpdateFunction::mk_constant(5),
                    &BmaUpdateFunction::mk_variable(42),
                ),
            ],
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_unmatched_parentheses() {
        let input = "3 + (5 * 2";
        let result = parse_bma_formula(input, &[]).unwrap_err();
        assert_eq!(result.message.as_str(), "Input ended while expecting `)`");
    }

    #[test]
    fn test_parse_invalid_token() {
        let input = "5 + @";
        let result = parse_bma_formula(input, &[]).unwrap_err();
        assert_eq!(result.message.as_str(), "Unexpected `@`");
    }

    #[test]
    fn test_parse_function_with_multiple_arguments() {
        let input = "max(3, 5, 10)";
        let result = parse_bma_formula(input, &[]);
        let expected = BmaUpdateFunction::mk_aggregation(
            AggregateFn::Max,
            &[
                BmaUpdateFunction::mk_constant(3),
                BmaUpdateFunction::mk_constant(5),
                BmaUpdateFunction::mk_constant(10),
            ],
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_empty_formula() {
        let input = "";
        let result = parse_bma_formula(input, &[]).unwrap_err();
        assert_eq!(result.message.as_str(), "Expression is empty");
    }

    #[test]
    fn test_parse_left_associative_division() {
        let input = "8 / 4 / 2";
        let result = parse_bma_formula(input, &[]).unwrap();
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Div,
            &BmaUpdateFunction::mk_arithmetic(
                ArithOp::Div,
                &BmaUpdateFunction::mk_constant(8),
                &BmaUpdateFunction::mk_constant(4),
            ),
            &BmaUpdateFunction::mk_constant(2),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_add_sub_left_associative() {
        let input = "1 - 2 + 3";
        let result = parse_bma_formula(input, &[]).unwrap();
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Plus,
            &BmaUpdateFunction::mk_arithmetic(
                ArithOp::Minus,
                &BmaUpdateFunction::mk_constant(1),
                &BmaUpdateFunction::mk_constant(2),
            ),
            &BmaUpdateFunction::mk_constant(3),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_sub_chain_left_associative() {
        let input = "1 - 2 - 3";
        let result = parse_bma_formula(input, &[]).unwrap();
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Minus,
            &BmaUpdateFunction::mk_arithmetic(
                ArithOp::Minus,
                &BmaUpdateFunction::mk_constant(1),
                &BmaUpdateFunction::mk_constant(2),
            ),
            &BmaUpdateFunction::mk_constant(3),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_priority() {
        let input = "1 - 2 * 3";
        let result = parse_bma_formula(input, &[]).unwrap();
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Minus,
            &BmaUpdateFunction::mk_constant(1),
            &BmaUpdateFunction::mk_arithmetic(
                ArithOp::Mult,
                &BmaUpdateFunction::mk_constant(2),
                &BmaUpdateFunction::mk_constant(3),
            ),
        );
        assert_eq!(result, expected);

        let input = "1 + 2 / 3";
        let result = parse_bma_formula(input, &[]).unwrap();
        let expected = BmaUpdateFunction::mk_arithmetic(
            ArithOp::Plus,
            &BmaUpdateFunction::mk_constant(1),
            &BmaUpdateFunction::mk_arithmetic(
                ArithOp::Div,
                &BmaUpdateFunction::mk_constant(2),
                &BmaUpdateFunction::mk_constant(3),
            ),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_unexpected_tokens() {
        let input = "1 + 1 2 3";
        let result = parse_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.message,
            "Unexpected: `1 2 3`. Expecting atomic proposition, function call, or parenthesis group"
        );
    }

    #[test]
    fn test_empty_sub_expression() {
        let input = "1 - + 1";
        let result = parse_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.message,
            "Found nothing at the right-hand-side of operator `-`"
        );

        let input = "+ 1 + 1";
        let result = parse_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.message,
            "Found nothing at the left-hand-side of operator `+`"
        );
    }
}
