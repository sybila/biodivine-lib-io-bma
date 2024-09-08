use crate::update_fn::enums::{AggregateFn, ArithOp, Literal, UnaryFn};
use std::iter::Peekable;
use std::str::Chars;

/// Enum of all possible tokens occurring in a BMA function string.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BmaFnToken {
    Atomic(Literal),
    Unary(UnaryFn, Box<BmaFnToken>),
    Binary(ArithOp),
    Aggregate(AggregateFn, Vec<BmaFnToken>),
    TokenList(Vec<BmaFnToken>),
}

/// Tokenize a BMA formula string into tokens,
///
/// This is a wrapper for the (more general) recursive [try_tokenize_recursive]` function.
pub fn try_tokenize_bma_formula(formula: String) -> Result<Vec<BmaFnToken>, String> {
    let (tokens, _) = try_tokenize_recursive(&mut formula.chars().peekable(), true, false)?;
    Ok(tokens)
}

/// Process a peekable iterator of characters into a vector of `BmaFnToken`s. This function is used
/// for both tokenizing a top-level expression and expressions that are fn's arguments.
///
/// Returns a vector of (nested) tokens, and a last character. The last character is important when
/// we are parsing function arguments (to find out if another argument is expected or we already
/// processed the closing parenthesis). When parsing the top-level formula expression (not a function
/// argument), we simply return '$'.
///
/// `top_fn_level` is used in case we are processing an expression passed as argument to some
/// function symbol (then ',' is valid delimiter).
fn try_tokenize_recursive(
    input_chars: &mut Peekable<Chars>,
    top_level: bool,
    top_fn_level: bool,
) -> Result<(Vec<BmaFnToken>, char), String> {
    let mut output = Vec::new();

    while let Some(c) = input_chars.next() {
        match c {
            c if c.is_whitespace() => {}
            '+' => output.push(BmaFnToken::Binary(ArithOp::Plus)),
            '-' => output.push(BmaFnToken::Binary(ArithOp::Minus)),
            '*' => output.push(BmaFnToken::Binary(ArithOp::Mult)),
            '/' => output.push(BmaFnToken::Binary(ArithOp::Div)),
            '(' => {
                // start a nested token group
                let (token_group, _) = try_tokenize_recursive(input_chars, false, false)?;
                output.push(BmaFnToken::TokenList(token_group));
            }
            ')' => {
                return if !top_level {
                    Ok((output, ')'))
                } else {
                    Err("Unexpected ')' without opening counterpart.".to_string())
                }
            }
            ',' if top_fn_level => {
                // in case we are collecting something inside a function, a comma is valid delimiter
                return Ok((output, ','));
            }
            // parse literals, function names
            c if is_valid_start_name(c) => {
                let name = format!("{c}{}", collect_name(input_chars));
                match name.as_str() {
                    "abs" => {
                        let args = collect_fn_arguments(input_chars)?;
                        output.push(BmaFnToken::Unary(
                            UnaryFn::Abs,
                            Box::new(args[0].to_owned()),
                        ))
                    }
                    "ceil" => {
                        let args = collect_fn_arguments(input_chars)?;
                        output.push(BmaFnToken::Unary(
                            UnaryFn::Ceil,
                            Box::new(args[0].to_owned()),
                        ))
                    }
                    "floor" => {
                        let args = collect_fn_arguments(input_chars)?;
                        output.push(BmaFnToken::Unary(
                            UnaryFn::Floor,
                            Box::new(args[0].to_owned()),
                        ))
                    }
                    "min" => {
                        let args = collect_fn_arguments(input_chars)?;
                        output.push(BmaFnToken::Aggregate(AggregateFn::Min, args));
                    }
                    "max" => {
                        let args = collect_fn_arguments(input_chars)?;
                        output.push(BmaFnToken::Aggregate(AggregateFn::Max, args));
                    }
                    "avg" => {
                        let args = collect_fn_arguments(input_chars)?;
                        output.push(BmaFnToken::Aggregate(AggregateFn::Avg, args));
                    }
                    _ => {
                        // Assume it’s a literal
                        output.push(BmaFnToken::Atomic(Literal::Str(name)));
                    }
                }
            }
            '0'..='9' => {
                let number = format!("{c}{}", collect_number_str(input_chars));
                let int_number = number
                    .parse::<i32>()
                    .map_err(|_| "Failed to parse number".to_string())?;
                output.push(BmaFnToken::Atomic(Literal::Int(int_number)));
            }
            _ => {
                return Err(format!("Unexpected character: '{c}'"));
            }
        }
    }

    if top_level {
        Ok((output, '$'))
    } else {
        Err("Expected ')' to previously encountered opening counterpart.".to_string())
    }
}

/// Check all whitespaces at the front of the iterator.
fn skip_whitespaces(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next(); // Skip the whitespace character
        } else {
            break; // Stop skipping when a non-whitespace character is found
        }
    }
}

/// Check if given char can appear in a name.
fn is_valid_in_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Check if given char can appear at the beginning of a name.
fn is_valid_start_name(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Collects a name (e.g., for variables, functions) from the input character iterator.
fn collect_name(input_chars: &mut Peekable<Chars>) -> String {
    let mut name = String::new();
    while let Some(&c) = input_chars.peek() {
        if is_valid_in_name(c) {
            name.push(c);
            input_chars.next(); // consume the character
        } else {
            break;
        }
    }
    name
}

/// Collects a number (integer) from the input character iterator.
fn collect_number_str(input_chars: &mut Peekable<Chars>) -> String {
    let mut number_str = String::new();
    while let Some(&c) = input_chars.peek() {
        if c.is_ascii_digit() {
            number_str.push(c);
            input_chars.next(); // consume the character
        } else {
            break;
        }
    }
    number_str
}

/// Collects the arguments for a function from the input character iterator.
fn collect_fn_arguments(input_chars: &mut Peekable<Chars>) -> Result<Vec<BmaFnToken>, String> {
    skip_whitespaces(input_chars);

    if Some('(') != input_chars.next() {
        return Err("Function name must be followed by `(`.".to_string());
    }

    let mut args = Vec::new();
    let mut last_delim = ',';

    while last_delim != ')' {
        assert_eq!(last_delim, ',');
        let (token_group, last_char) = try_tokenize_recursive(input_chars, false, true)?;
        if token_group.is_empty() {
            return Err("Function argument cannot be empty.".to_string());
        }
        args.push(BmaFnToken::TokenList(token_group));
        last_delim = last_char;
    }

    Ok(args)
}

#[cfg(test)]
mod tests {
    use crate::update_fn::enums::{AggregateFn, ArithOp, Literal, UnaryFn};
    use crate::update_fn::tokenizer::{try_tokenize_bma_formula, BmaFnToken};

    #[test]
    fn test_simple_arithmetic() {
        let input = "3 + 5 - 2".to_string();
        let result = try_tokenize_bma_formula(input);
        assert_eq!(
            result,
            Ok(vec![
                BmaFnToken::Atomic(Literal::Int(3)),
                BmaFnToken::Binary(ArithOp::Plus),
                BmaFnToken::Atomic(Literal::Int(5)),
                BmaFnToken::Binary(ArithOp::Minus),
                BmaFnToken::Atomic(Literal::Int(2))
            ])
        );
    }

    #[test]
    fn test_function_with_single_argument() {
        let input = "abs(5)".to_string();
        let result = try_tokenize_bma_formula(input);
        assert_eq!(
            result,
            Ok(vec![BmaFnToken::Unary(
                UnaryFn::Abs,
                Box::new(BmaFnToken::TokenList(vec![BmaFnToken::Atomic(
                    Literal::Int(5)
                )])),
            )])
        );
    }

    #[test]
    fn test_aggregate_function_with_multiple_arguments() {
        let input = "min(5, 3)".to_string();
        let result = try_tokenize_bma_formula(input);
        assert_eq!(
            result,
            Ok(vec![BmaFnToken::Aggregate(
                AggregateFn::Min,
                vec![
                    BmaFnToken::TokenList(vec![BmaFnToken::Atomic(Literal::Int(5))]),
                    BmaFnToken::TokenList(vec![BmaFnToken::Atomic(Literal::Int(3))])
                ]
            )])
        );
    }

    #[test]
    fn test_nested_function_calls() {
        let input = "max(abs(5), ceil(3))".to_string();
        let result = try_tokenize_bma_formula(input);
        assert_eq!(
            result,
            Ok(vec![BmaFnToken::Aggregate(
                AggregateFn::Max,
                vec![
                    BmaFnToken::TokenList(vec![BmaFnToken::Unary(
                        UnaryFn::Abs,
                        Box::new(BmaFnToken::TokenList(vec![BmaFnToken::Atomic(
                            Literal::Int(5)
                        )])),
                    )]),
                    BmaFnToken::TokenList(vec![BmaFnToken::Unary(
                        UnaryFn::Ceil,
                        Box::new(BmaFnToken::TokenList(vec![BmaFnToken::Atomic(
                            Literal::Int(3)
                        )])),
                    )])
                ]
            )])
        );
    }

    #[test]
    fn test_unmatched_parentheses() {
        let input = "min(5, 3".to_string();
        let result = try_tokenize_bma_formula(input);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err("Expected ')' to previously encountered opening counterpart.".to_string())
        );
    }

    #[test]
    fn test_unexpected_character() {
        let input = "5 + @".to_string();
        let result = try_tokenize_bma_formula(input);
        assert!(result.is_err());
        assert_eq!(result, Err("Unexpected character: '@'".to_string()));
    }

    #[test]
    fn test_compound_expression_with_nested_parentheses() {
        let input = "3 + (5 * (2 + 1))".to_string();
        let result = try_tokenize_bma_formula(input);
        assert_eq!(
            result,
            Ok(vec![
                BmaFnToken::Atomic(Literal::Int(3)),
                BmaFnToken::Binary(ArithOp::Plus),
                BmaFnToken::TokenList(vec![
                    BmaFnToken::Atomic(Literal::Int(5)),
                    BmaFnToken::Binary(ArithOp::Mult),
                    BmaFnToken::TokenList(vec![
                        BmaFnToken::Atomic(Literal::Int(2)),
                        BmaFnToken::Binary(ArithOp::Plus),
                        BmaFnToken::Atomic(Literal::Int(1))
                    ])
                ])
            ])
        );
    }

    #[test]
    fn test_function_with_no_arguments_invalid() {
        let input = "abs()".to_string();
        let result = try_tokenize_bma_formula(input);
        assert!(result.is_err());
    }
}
