use crate::update_function::BmaUpdateFunction;
use crate::update_function::expression_enums::*;
use crate::update_function::expression_token::{BmaExpressionToken, try_tokenize_bma_formula};

/// Parse an BMA update function formula string representation into an actual expression tree.
/// Basically a wrapper for tokenize+parse (used often for testing/debug purposes).
///
/// Arg `variables` is a map of variable IDs to their names. It is needed because there are
/// some weird format differences between different variants, and a variable can be referenced
/// by either its ID or its name. We convert everything to IDs for easier processing.
pub fn parse_bma_formula(
    formula: &str,
    variables: &[(u32, String)],
) -> Result<BmaUpdateFunction, String> {
    let tokens = try_tokenize_bma_formula(formula, variables).map_err(|e| e.to_string())?;
    let tree = parse_bma_fn_tokens(&tokens)?;
    Ok(tree)
}

/// Utility method to find the first occurrence of a specific token in the token tree.
fn index_of_first(tokens: &[BmaExpressionToken], token: BmaExpressionToken) -> Option<usize> {
    tokens.iter().position(|t| *t == token)
}

/// Parse `tokens` of BMA update fn formula into an abstract syntax tree using recursive steps.
pub fn parse_bma_fn_tokens(tokens: &[BmaExpressionToken]) -> Result<BmaUpdateFunction, String> {
    parse_1_div(tokens)
}

/// Recursive parsing step 1: extract `/` operators.
fn parse_1_div(tokens: &[BmaExpressionToken]) -> Result<BmaUpdateFunction, String> {
    let div_token = index_of_first(tokens, BmaExpressionToken::Binary(ArithOp::Div));
    Ok(if let Some(i) = div_token {
        BmaUpdateFunction::mk_arithmetic(
            ArithOp::Div,
            &parse_2_mul(&tokens[..i])?,
            &parse_1_div(&tokens[(i + 1)..])?,
        )
    } else {
        parse_2_mul(tokens)?
    })
}

/// Recursive parsing step 2: extract `*` operators.
fn parse_2_mul(tokens: &[BmaExpressionToken]) -> Result<BmaUpdateFunction, String> {
    let mul_token = index_of_first(tokens, BmaExpressionToken::Binary(ArithOp::Mult));
    Ok(if let Some(i) = mul_token {
        BmaUpdateFunction::mk_arithmetic(
            ArithOp::Mult,
            &parse_3_minus(&tokens[..i])?,
            &parse_2_mul(&tokens[(i + 1)..])?,
        )
    } else {
        parse_3_minus(tokens)?
    })
}

/// Recursive parsing step 3: extract `-` operators.
fn parse_3_minus(tokens: &[BmaExpressionToken]) -> Result<BmaUpdateFunction, String> {
    let minus_token = index_of_first(tokens, BmaExpressionToken::Binary(ArithOp::Minus));
    Ok(if let Some(i) = minus_token {
        BmaUpdateFunction::mk_arithmetic(
            ArithOp::Minus,
            &parse_4_plus(&tokens[..i])?,
            &parse_3_minus(&tokens[(i + 1)..])?,
        )
    } else {
        parse_4_plus(tokens)?
    })
}

/// Recursive parsing step 4: extract `+` operators.
fn parse_4_plus(tokens: &[BmaExpressionToken]) -> Result<BmaUpdateFunction, String> {
    let minus_token = index_of_first(tokens, BmaExpressionToken::Binary(ArithOp::Plus));
    Ok(if let Some(i) = minus_token {
        BmaUpdateFunction::mk_arithmetic(
            ArithOp::Plus,
            &parse_5_others(&tokens[..i])?,
            &parse_4_plus(&tokens[(i + 1)..])?,
        )
    } else {
        parse_5_others(tokens)?
    })
}

/// Recursive parsing step 5: extract literals and recursively solve sub-formulae in parentheses
/// and in functions.
fn parse_5_others(tokens: &[BmaExpressionToken]) -> Result<BmaUpdateFunction, String> {
    if tokens.is_empty() {
        Err("Expected formula, found nothing.".to_string())
    } else {
        if tokens.len() == 1 {
            // This should be named (var/function) or a parenthesis group, anything
            // else does not make sense.
            match &tokens[0] {
                BmaExpressionToken::Atomic(Literal::Var(var_id)) => {
                    return Ok(BmaUpdateFunction::mk_variable(*var_id));
                }
                BmaExpressionToken::Atomic(Literal::Const(num)) => {
                    return Ok(BmaUpdateFunction::mk_constant(*num));
                }
                BmaExpressionToken::Aggregate(operator, arguments) => {
                    let mut arg_expression_nodes = Vec::new();
                    for inner in arguments {
                        // it must be a token list
                        if let BmaExpressionToken::TokenList(inner_token_list) = inner {
                            arg_expression_nodes.push(parse_bma_fn_tokens(inner_token_list)?);
                        } else {
                            let message =
                                "Function must be applied on `BmaFnToken::TokenList` args.";
                            return Err(message.to_string());
                        }
                    }
                    return Ok(BmaUpdateFunction::mk_aggregation(
                        *operator,
                        &arg_expression_nodes,
                    ));
                }
                BmaExpressionToken::Unary(operator, argument) => {
                    return if let BmaExpressionToken::TokenList(inner_token_list) =
                        *argument.clone()
                    {
                        Ok(BmaUpdateFunction::mk_unary(
                            *operator,
                            &parse_bma_fn_tokens(&inner_token_list)?,
                        ))
                    } else {
                        return Err(
                            "Function must be applied on `BmaFnToken::TokenList` args.".to_string()
                        );
                    };
                }
                // recursively solve sub-formulae in parentheses
                BmaExpressionToken::TokenList(inner) => {
                    return parse_bma_fn_tokens(inner);
                }
                _ => {} // otherwise, fall through to the error at the end.
            }
        }
        Err(format!("Unexpected: {tokens:?}. Expecting formula."))
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
        let result = parse_bma_formula(input, &[]);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(
                "Cannot tokenize expression: Input ended while expecting `)` at position `10`"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_parse_invalid_token() {
        let input = "5 + @";
        let result = parse_bma_formula(input, &[]);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err("Cannot tokenize expression: Unexpected `@` at position `4`".to_string())
        );
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
        let result = parse_bma_formula(input, &[]);
        assert!(result.is_err());
        assert_eq!(result, Err("Expected formula, found nothing.".to_string()));
    }
}
