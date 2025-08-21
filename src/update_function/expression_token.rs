use crate::update_function::expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
use std::collections::BTreeSet;
use thiserror::Error;

/// Enum of all possible tokens occurring in a BMA function string.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BmaExpressionToken {
    /// Constant or variable.
    Atomic(Literal),
    /// Unary function with argument(s).
    Unary(UnaryFn, Box<BmaExpressionToken>),
    /// A binary arithmetic operator
    Binary(ArithOp),
    /// Aggregation function with arguments.
    Aggregate(AggregateFn, Vec<BmaExpressionToken>),
    /// A closed parentheses group.
    TokenList(Vec<BmaExpressionToken>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("Cannot tokenize expression: {error_type} at position `{position}`")]
pub struct BmaTokenizationError {
    pub position: usize,
    pub error_type: String,
}

impl BmaTokenizationError {
    pub fn at(position: usize, error_type: String) -> BmaTokenizationError {
        BmaTokenizationError {
            position,
            error_type,
        }
    }
}

/// Tokenize a BMA function expression into tokens.
///
/// Arg `variable_id_hint` is a map of variable IDs to their names. It is needed when the model
/// uses variable names instead of IDs in the function expressions.
pub fn try_tokenize_bma_formula(
    formula: &str,
    variable_id_hint: &[(u32, String)],
) -> Result<Vec<BmaExpressionToken>, BmaTokenizationError> {
    let chars: Vec<char> = formula.chars().collect();
    let (tokens, length) = try_tokenize_recursive(&chars, 0, false, false, variable_id_hint)?;

    // If the tokenizer succeeds, it should always read the whole string.
    debug_assert!(length == chars.len());

    Ok(tokens)
}

/// Process an input string into a vector of [`BmaExpressionToken`] objects, starting from the
/// `start_at` position. The function also returns the *length of the tokenized region*.
///
/// If `ends_with_comma` or `ends_with_parenthesis` is specified, the tokenization
/// will terminate once this delimiting character is encountered. The character is counted
/// as tokenized, and the function returns the index of the first character after this
/// value. If both values are false, the function will try to tokenize the whole input string.
///
/// If provided, `variable_id_hint` will be used to resolve variable names into IDs.
///
#[allow(clippy::too_many_lines)]
fn try_tokenize_recursive(
    input: &[char],
    start_at: usize,
    ends_with_comma: bool,
    ends_with_parenthesis: bool,
    variable_id_hint: &[(u32, String)],
) -> Result<(Vec<BmaExpressionToken>, usize), BmaTokenizationError> {
    let mut result = Vec::new();
    let mut position = start_at;

    while position < input.len() {
        match input[position] {
            ',' => {
                return if ends_with_comma {
                    // We have found the stop character. This tokenization group is done, and
                    // next group can start from the next position.
                    Ok((result, position - start_at + 1))
                } else if ends_with_parenthesis {
                    let message = "Unclosed parenthesis (group closed by `,` before `)` was found)";
                    Err(BmaTokenizationError::at(position, message.to_string()))
                } else {
                    let message = "Unexpected `,`";
                    Err(BmaTokenizationError::at(position, message.to_string()))
                };
            }
            ')' => {
                return if ends_with_parenthesis {
                    return Ok((result, (position - start_at) + 1));
                } else {
                    let message = "Unexpected `)` (missing opening `(`)";
                    Err(BmaTokenizationError::at(position, message.to_string()))
                };
            }
            c if c.is_whitespace() => {
                // Ignore all whitespace.
                position += 1;
            }
            c if ['+', '-', '*', '/'].contains(&c) => {
                let op = ArithOp::try_from(c).unwrap();
                result.push(BmaExpressionToken::Binary(op));
                position += 1;
            }
            '(' => {
                // Start a nested token group.
                let (group, length) =
                    try_tokenize_recursive(input, position + 1, false, true, variable_id_hint)?;
                result.push(BmaExpressionToken::TokenList(group));
                position += length + 1;
            }
            // Parse integer constants
            '0'..='9' => {
                let number = collect_number_str(input, position);
                match number.parse::<i32>() {
                    Ok(constant) => {
                        result.push(BmaExpressionToken::Atomic(Literal::Const(constant)));
                        position += number.len();
                    }
                    Err(e) => {
                        let message = format!("Invalid number `{number}`: {e}");
                        return Err(BmaTokenizationError::at(position, message));
                    }
                }
            }
            // Parse  var literals and functions
            c if is_valid_start_name(c) => {
                let id = collect_identifier_str(input, position);
                position += id.len();
                match id.as_str() {
                    id if ["min", "max", "avg"].contains(&id) => {
                        let (args, length) =
                            collect_function_arguments(input, position, variable_id_hint)?;
                        // Must not fail due to the test above.
                        let op = AggregateFn::try_from(id).unwrap();
                        result.push(BmaExpressionToken::Aggregate(op, args));
                        position += length;
                    }
                    id if ["abs", "ceil", "floor"].contains(&id) => {
                        let (args, length) =
                            collect_function_arguments(input, position, variable_id_hint)?;
                        if args.len() != 1 {
                            let message = format!(
                                "Function `{}` expects exactly one argument; found `{}`",
                                id,
                                args.len()
                            );
                            return Err(BmaTokenizationError::at(position, message));
                        }
                        // Must not fail due to the test above.
                        let op = UnaryFn::try_from(id).unwrap();
                        let arg = args.into_iter().next().unwrap();
                        result.push(BmaExpressionToken::Unary(op, Box::new(arg)));
                        position += length;
                    }
                    "var" => {
                        let (identifier, length) = collect_variable_identifier(input, position)?;
                        let var_id = if let Ok(var_id) = identifier.parse::<u32>() {
                            var_id
                        } else {
                            let matching_vars = variable_id_hint
                                .iter()
                                .filter(|(_id, name)| name.as_str() == identifier.as_str())
                                .map(|(id, _)| *id)
                                .collect::<BTreeSet<_>>();
                            if matching_vars.is_empty() {
                                let message = format!("`{identifier}` is not a known regulator");
                                return Err(BmaTokenizationError::at(position, message));
                            } else if matching_vars.len() > 1 {
                                let message = format!(
                                    "`{identifier}` resolves to multiple regulator IDs: `{matching_vars:?}`"
                                );
                                return Err(BmaTokenizationError::at(position, message));
                            }
                            debug_assert_eq!(matching_vars.len(), 1);
                            matching_vars.into_iter().next().unwrap()
                        };
                        result.push(BmaExpressionToken::Atomic(Literal::Var(var_id)));
                        position += length;
                    }
                    id => {
                        let message = format!("`{id}` is not a recognized function or variable");
                        return Err(BmaTokenizationError::at(position - id.len(), message));
                    }
                }
            }
            c => {
                // Any other character is unexpected at this point.
                let message = format!("Unexpected `{c}`");
                return Err(BmaTokenizationError::at(position, message));
            }
        }
    }

    // Technically, if something ends with a comma, it must always also end with parenthesis,
    // but in theory, future implementations do not need to require this.
    if ends_with_parenthesis {
        let message = "Input ended while expecting `)`";
        return Err(BmaTokenizationError::at(position, message.to_string()));
    }
    if ends_with_comma {
        let message = "Input ended while expecting `,`";
        return Err(BmaTokenizationError::at(position, message.to_string()));
    }

    Ok((result, position - start_at))
}

/// Check all whitespaces at the front of the iterator.
fn next_non_whitespace_character(input: &[char], mut position: usize) -> usize {
    while position < input.len() && input[position].is_whitespace() {
        position += 1;
    }
    position
}

/// Check if given char can appear in a name.
///
/// Apparently, "-" is valid too, as it is present in variable names in
/// most XML BMA model files...
fn is_valid_in_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

/// Check if given char can appear at the beginning of a name.
fn is_valid_start_name(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Collects a name (e.g., for variables, functions) from the input characters.
fn collect_identifier_str(input: &[char], start_at: usize) -> String {
    let mut name = String::new();
    let mut position = start_at;
    while position < input.len() && is_valid_in_name(input[position]) {
        name.push(input[position]);
        position += 1;
    }
    name
}

/// Collects a variable name/identifier from the input characters.
///
/// This function is used when parsing a variable in the form `var(x)`.
/// It expects the name to be enclosed in parentheses, with possible whitespace.
fn collect_variable_identifier(
    input: &[char],
    start_at: usize,
) -> Result<(String, usize), BmaTokenizationError> {
    let mut position = next_non_whitespace_character(input, start_at);

    if position >= input.len() || input[position] != '(' {
        let message = "Expected `var` to be followed by `(`";
        return Err(BmaTokenizationError::at(position, message.to_string()));
    }

    position = next_non_whitespace_character(input, position + 1);
    let identifier = collect_identifier_str(input, position);

    if identifier.is_empty() {
        let message = "No identifier found in `var` expression";
        return Err(BmaTokenizationError::at(position, message.to_string()));
    }

    position += identifier.len();

    if position >= input.len() || input[position] != ')' {
        let message = "Expected `var` to be closed by `)`";
        return Err(BmaTokenizationError::at(position, message.to_string()));
    }

    Ok((identifier, position - start_at + 1))
}

/// Collects a number (integer) from input characters.
fn collect_number_str(input: &[char], start_at: usize) -> String {
    let mut number_str = String::new();
    let mut position = start_at;
    while position < input.len() && input[position].is_ascii_digit() {
        number_str.push(input[position]);
        position += 1;
    }
    number_str
}

/// Collects the arguments for a function from the input characters. The method guarantees that
/// the items of the returned list are separated by commas and have proper parentheses.
fn collect_function_arguments(
    input: &[char],
    start_at: usize,
    variable_id_hint: &[(u32, String)],
) -> Result<(Vec<BmaExpressionToken>, usize), BmaTokenizationError> {
    let mut position = next_non_whitespace_character(input, start_at);

    if position >= input.len() || input[position] != '(' {
        let message = "Expected argument list, but opening `(` is missing";
        return Err(BmaTokenizationError::at(position, message.to_string()));
    }

    position = next_non_whitespace_character(input, position + 1);

    let mut args = Vec::new();
    loop {
        // Tokenization of a single argument can end if comma or parenthesis is found.
        let (group, length) =
            try_tokenize_recursive(input, position, true, true, variable_id_hint)?;

        if group.is_empty() {
            let message = "Argument is empty";
            return Err(BmaTokenizationError::at(position, message.to_string()));
        }

        args.push(BmaExpressionToken::TokenList(group));

        debug_assert!(length > 0);
        position += length;

        // If the last character of this group is parenthesis, we are done.
        if input[position - 1] == ')' {
            break;
        }
        debug_assert_eq!(input[position - 1], ',');
        position = next_non_whitespace_character(input, position);
    }

    Ok((args, position - start_at))
}

#[cfg(test)]
mod tests {
    use crate::update_function::expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
    use crate::update_function::expression_token::{
        BmaExpressionToken, try_tokenize_bma_formula, try_tokenize_recursive,
    };

    #[test]
    fn test_simple_arithmetic() {
        let input = "3 + 5 - 2";
        let result = try_tokenize_bma_formula(input, &[]).unwrap();
        assert_eq!(
            result,
            vec![
                BmaExpressionToken::Atomic(Literal::Const(3)),
                BmaExpressionToken::Binary(ArithOp::Plus),
                BmaExpressionToken::Atomic(Literal::Const(5)),
                BmaExpressionToken::Binary(ArithOp::Minus),
                BmaExpressionToken::Atomic(Literal::Const(2))
            ]
        );
    }

    #[test]
    fn test_function_with_single_argument() {
        let input = "abs(5)";
        let result = try_tokenize_bma_formula(input, &[]).unwrap();
        assert_eq!(
            result,
            vec![BmaExpressionToken::Unary(
                UnaryFn::Abs,
                Box::new(BmaExpressionToken::TokenList(vec![
                    BmaExpressionToken::Atomic(Literal::Const(5))
                ])),
            )]
        );
    }

    #[test]
    fn test_aggregate_function_with_multiple_arguments() {
        let input = "min(5, 3)";
        let result = try_tokenize_bma_formula(input, &[]).unwrap();
        assert_eq!(
            result,
            vec![BmaExpressionToken::Aggregate(
                AggregateFn::Min,
                vec![
                    BmaExpressionToken::TokenList(vec![BmaExpressionToken::Atomic(
                        Literal::Const(5)
                    )]),
                    BmaExpressionToken::TokenList(vec![BmaExpressionToken::Atomic(
                        Literal::Const(3)
                    )])
                ]
            )]
        );
    }

    #[test]
    fn test_nested_function_calls() {
        let input = "max(abs(5), ceil(3))";
        let result = try_tokenize_bma_formula(input, &[]);
        assert_eq!(
            result,
            Ok(vec![BmaExpressionToken::Aggregate(
                AggregateFn::Max,
                vec![
                    BmaExpressionToken::TokenList(vec![BmaExpressionToken::Unary(
                        UnaryFn::Abs,
                        Box::new(BmaExpressionToken::TokenList(vec![
                            BmaExpressionToken::Atomic(Literal::Const(5))
                        ])),
                    )]),
                    BmaExpressionToken::TokenList(vec![BmaExpressionToken::Unary(
                        UnaryFn::Ceil,
                        Box::new(BmaExpressionToken::TokenList(vec![
                            BmaExpressionToken::Atomic(Literal::Const(3))
                        ])),
                    )])
                ]
            )])
        );
    }

    #[test]
    fn test_compound_expression_with_nested_parentheses() {
        let input = "3 + (5 * (2 + 1))";
        let result = try_tokenize_bma_formula(input, &[]);
        assert_eq!(
            result,
            Ok(vec![
                BmaExpressionToken::Atomic(Literal::Const(3)),
                BmaExpressionToken::Binary(ArithOp::Plus),
                BmaExpressionToken::TokenList(vec![
                    BmaExpressionToken::Atomic(Literal::Const(5)),
                    BmaExpressionToken::Binary(ArithOp::Mult),
                    BmaExpressionToken::TokenList(vec![
                        BmaExpressionToken::Atomic(Literal::Const(2)),
                        BmaExpressionToken::Binary(ArithOp::Plus),
                        BmaExpressionToken::Atomic(Literal::Const(1))
                    ])
                ])
            ])
        );
    }

    #[test]
    fn test_variable() {
        // try both variable name and ID

        let vars = vec![
            (42u32, "x".to_string()),
            (1u32, "y".to_string()),
            (2u32, "y".to_string()),
        ];

        let var_literal = BmaExpressionToken::Atomic(Literal::Var(42));

        // Variable can be found if among regulators, by both name an ID.
        let input = "var(x)";
        let result = try_tokenize_bma_formula(input, &vars).unwrap();
        assert_eq!(result, vec![var_literal.clone()]);

        let input = "var(42)";
        let result = try_tokenize_bma_formula(input, &vars).unwrap();
        assert_eq!(result, vec![var_literal.clone()]);

        let input = "var(y)";
        let result = try_tokenize_bma_formula(input, &vars).unwrap_err();
        assert_eq!(
            result.error_type,
            "`y` resolves to multiple regulator IDs: `{1, 2}`"
        );
        assert_eq!(result.position, 3);

        let input = "var(z)";
        let result = try_tokenize_bma_formula(input, &vars).unwrap_err();
        assert_eq!(result.error_type, "`z` is not a known regulator");
        assert_eq!(result.position, 3);

        let input = "var()";
        let result = try_tokenize_bma_formula(input, &vars).unwrap_err();
        assert_eq!(result.error_type, "No identifier found in `var` expression");
        assert_eq!(result.position, 4);

        let input = "var x";
        let result = try_tokenize_bma_formula(input, &vars).unwrap_err();
        assert_eq!(result.error_type, "Expected `var` to be followed by `(`");
        assert_eq!(result.position, 4);

        let input = "var(x";
        let result = try_tokenize_bma_formula(input, &vars).unwrap_err();
        assert_eq!(result.error_type, "Expected `var` to be closed by `)`");
        assert_eq!(result.position, 5);
    }

    #[test]
    fn test_unmatched_parentheses() {
        let input = "min(5, 3";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(result.error_type, "Input ended while expecting `)`");
        assert_eq!(result.position, 8);
    }

    #[test]
    fn test_unmatched_comma() {
        // This is currently not possible under normal circumstances, because every group
        // that can be terminated by a comma can be also terminated by parenthesis.
        // But it could appear as a use case in the future.
        let input = "2 * 3";
        let input_chars = Vec::from_iter(input.chars());
        let result = try_tokenize_recursive(&input_chars, 0, true, false, &[]).unwrap_err();
        assert_eq!(result.error_type, "Input ended while expecting `,`");
        assert_eq!(result.position, 5);
    }

    #[test]
    fn test_unexpected_character() {
        let input = "5 + @";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(result.error_type, "Unexpected `@`");
        assert_eq!(result.position, 4);
    }

    #[test]
    fn test_function_with_no_arguments_invalid() {
        let input = "abs()";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(result.error_type, "Argument is empty");
        assert_eq!(result.position, 4);
    }

    #[test]
    fn test_unclosed_parenthesis_in_arguments() {
        let input = "max(1 + (2 -, 3)";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.error_type.as_str(),
            "Unclosed parenthesis (group closed by `,` before `)` was found)"
        );
        assert_eq!(result.position, 12);
    }

    #[test]
    fn test_comma_outside_arguments() {
        let input = "1 + 2, 3";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(result.error_type.as_str(), "Unexpected `,`");
        assert_eq!(result.position, 5);
    }

    #[test]
    fn test_extra_closing_parenthesis() {
        let input = "1 + 2) - 3";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.error_type.as_str(),
            "Unexpected `)` (missing opening `(`)"
        );
        assert_eq!(result.position, 5);
    }

    #[test]
    fn test_invalid_number() {
        let input = "12345678901234567890";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.error_type.as_str(),
            "Invalid number `12345678901234567890`: number too large to fit in target type"
        );
        assert_eq!(result.position, 0);
    }

    #[test]
    fn test_unary_arguments() {
        let input = "abs(1, 2)";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.error_type.as_str(),
            "Function `abs` expects exactly one argument; found `2`"
        );
        assert_eq!(result.position, 3);
    }

    #[test]
    fn test_unknown_function() {
        let input = "foo(1, 2)";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.error_type.as_str(),
            "`foo` is not a recognized function or variable"
        );
        assert_eq!(result.position, 0);
    }

    #[test]
    fn test_missing_arguments() {
        let input = "max 1, 2";
        let result = try_tokenize_bma_formula(input, &[]).unwrap_err();
        assert_eq!(
            result.error_type.as_str(),
            "Expected argument list, but opening `(` is missing"
        );
        assert_eq!(result.position, 4);
    }
}
