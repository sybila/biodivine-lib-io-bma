use crate::update_fn::bma_fn_tree::{BmaFnNodeType, BmaFnUpdate};
use crate::update_fn::expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
use num_rational::Rational32;
use num_traits::sign::Signed;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{dec, Decimal};
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

/// A function table is a vector of tuples, where each tuple contains a variable valuation
/// and output value. Variable valuation is a mapping of variable IDs to their values (as
/// a HashMap).
type FunctionTable = Vec<(HashMap<u32, u32>, u32)>;

impl BmaFnUpdate {
    /// Convert the BMA expression into corresponding BN update function string
    /// matching the format of the [biodivine_lib_param_bn] library.
    ///
    /// Note that currently, WE ONLY SUPPORT BOOLEAN MODELS, even though some methods
    /// are already implemented to handle more general multi-valued cases as well.
    ///
    /// Map `max_levels` indicates the maximum level for each variable in the model. For
    /// Boolean networks, this is set to 1 for all variables.
    /// Arg `var_name_mapping` maps each BMA variable ID to its canonical name in the
    /// constructed BN.
    /// Arg `this_var_max_lvl` is the maximum level of the variable for which we are  
    /// creating the update function.
    ///
    /// TODO: implementation via explicit construction of the function table
    pub fn to_update_fn_boolean(
        &self,
        max_levels: &HashMap<u32, u32>,
        var_name_mapping: &HashMap<u32, String>,
        this_var_max_lvl: u32,
    ) -> Result<String, String> {
        // To convert the BMA expression into an update function, we essetially create
        // an explicit function table mapping all valuations of inputs to output values.
        // In BNs, this corresponds to a truth table.
        // We can then use this function table to create a new logical update formula.

        // Collect all variable IDs used in the expression, and sort them
        let mut variables_in_fn: Vec<u32> = self.collect_variables().into_iter().collect();
        variables_in_fn.sort();

        // Create a function table and convert it into DNF formula
        let function_table =
            self.build_function_table(&variables_in_fn, max_levels, this_var_max_lvl)?;
        let mut conjunction_clauses = Vec::new();
        for (valuation, fn_value) in function_table {
            if fn_value == 1 {
                let conjunction_str = valuation
                    .iter()
                    .map(|(id, value)| {
                        let bn_var_name = var_name_mapping.get(id).unwrap(); // unwrap is safe here
                                                                             // create positive or negative literal based on the value
                        if *value == 0 {
                            format!("!{bn_var_name}")
                        } else {
                            bn_var_name.clone()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" & ");

                conjunction_clauses.push(format!("({conjunction_str})"));
            }
        }

        let dnf_formula = if conjunction_clauses.is_empty() {
            "false".to_string()
        } else {
            conjunction_clauses.join(" | ")
        };

        Ok(dnf_formula)
    }

    /// Evaluate the BMA function expression in a given valuation.
    /// A `valuation`` assigns values to all variables (ID-value mapping).
    pub fn evaluate_in_valuation(
        &self,
        valuation: &HashMap<u32, Rational32>,
    ) -> Result<Rational32, String> {
        match &self.expression_tree {
            BmaFnNodeType::Terminal(Literal::Var(var_id)) => {
                if let Some(value) = valuation.get(var_id) {
                    Ok(*value)
                } else {
                    Err(format!("Variable `{var_id}` not found in the valuation."))
                }
            }
            BmaFnNodeType::Terminal(Literal::Const(value)) => Ok(Rational32::new(*value, 1)),
            BmaFnNodeType::Arithmetic(operator, left, right) => {
                let left_value = left.evaluate_in_valuation(valuation)?;
                let right_value = right.evaluate_in_valuation(valuation)?;
                let res = match operator {
                    ArithOp::Plus => left_value + right_value,
                    ArithOp::Minus => left_value - right_value,
                    ArithOp::Mult => left_value * right_value,
                    ArithOp::Div => left_value / right_value,
                };
                Ok(res)
            }
            BmaFnNodeType::Unary(function, child_node) => {
                let child_value = child_node.evaluate_in_valuation(valuation)?;
                let res = match function {
                    UnaryFn::Abs => Rational32::abs(&child_value),
                    UnaryFn::Ceil => Rational32::ceil(&child_value),
                    UnaryFn::Floor => Rational32::floor(&child_value),
                };
                Ok(res)
            }
            BmaFnNodeType::Aggregation(function, arguments) => {
                let args_values: Vec<Rational32> = arguments
                    .iter()
                    .map(|arg| arg.evaluate_in_valuation(valuation))
                    .collect::<Result<Vec<Rational32>, String>>()?;
                let res = match function {
                    AggregateFn::Avg => {
                        let count = args_values.len() as i32;
                        let sum: Rational32 = args_values.iter().cloned().sum();
                        sum / Rational32::from_integer(count)
                    }
                    AggregateFn::Max => args_values
                        .iter()
                        .cloned()
                        .max()
                        .expect("List of numbers is empty"),
                    AggregateFn::Min => args_values
                        .iter()
                        .cloned()
                        .min()
                        .expect("List of numbers is empty"),
                };
                Ok(res)
            }
        }
    }

    /// Collect all variable IDs used in this BMA function's expression.
    fn collect_variables(&self) -> HashSet<u32> {
        match &self.expression_tree {
            BmaFnNodeType::Terminal(Literal::Var(var_id)) => {
                let mut set = HashSet::new();
                set.insert(*var_id);
                set
            }
            BmaFnNodeType::Terminal(Literal::Const(_)) => HashSet::new(),
            BmaFnNodeType::Arithmetic(_, left, right) => {
                let left_set = left.collect_variables();
                let right_set = right.collect_variables();
                left_set.union(&right_set).cloned().collect()
            }
            BmaFnNodeType::Unary(_, child_node) => child_node.collect_variables(),
            BmaFnNodeType::Aggregation(_, arguments) => arguments
                .iter()
                .map(|arg| arg.collect_variables())
                .fold(HashSet::new(), |x, y| x.union(&y).cloned().collect()),
        }
    }

    /// Build a "function table" mapping all input valuations to the corresponding function
    /// value.
    /// For Boolean networks, the function table will essentially be a truth table,
    /// mapping boolean combinations of input variables to the output value.
    ///
    /// Arg `variables_in_fn` specifies the variables used in the function expression in order.
    /// Arg `this_var_max_lvl` specifies the maximum level of the variable for which we are
    /// creating the function table.
    ///
    /// This method can also handle multi-valued variables (arg `max_levels` specifies
    /// maximum level for all model variable), but the table needs to be further binarized
    /// to be used in a Boolean network.
    pub fn build_function_table(
        &self,
        variables_in_fn: &[u32],
        max_levels: &HashMap<u32, u32>,
        this_var_max_lvl: u32,
    ) -> Result<FunctionTable, String> {
        let input_valuations = generate_input_combinations(variables_in_fn, max_levels);
        let mut function_table = Vec::new();

        // Evaluate the function for each valuation, and round the result to an integer
        for valuation in input_valuations {
            // Evaluate the function, with result as a rational number
            let rational_result = self
                .evaluate_in_valuation(&valuation)
                .map_err(|err| format!("Internal error during function evaluation: {err}"))?;

            // Convert the valuation values to u32 (all input values are natural nums by design anyway)
            let int_valuation = valuation
                .iter()
                .map(|(var_id, value)| (*var_id, value.to_integer() as u32))
                .collect::<HashMap<u32, u32>>();

            // Convert the result to integer (rounding if necessary)
            let mut result_int = if rational_result.is_integer() {
                // Ideally, most numbers will not actually be fractions, and we dont have to round
                rational_result.to_integer()
            } else {
                // Otherwise, we need to convert the fraction into integer by rounding.
                // Note that BMA is written in C/C# which performs "round half up" arithemtic.
                // However, Rust performs "round half even" airthmetic, meaning we might return
                // a different value compared to BMA. We have to run it through this magic
                // formula that will actually perform a proper "round half up" rounding.

                let numerator_decimal = Decimal::from(*rational_result.numer());
                let denominator_decimal = Decimal::from(*rational_result.denom());
                let result_decimal = numerator_decimal / denominator_decimal;
                if result_decimal.fract() >= dec!(0.5) {
                    result_decimal.ceil().to_i32().unwrap()
                } else {
                    result_decimal.floor().to_i32().unwrap()
                }
            };

            // Ensure the result is non-negative, and in the valid range
            result_int = max(0, result_int);
            function_table.push((int_valuation, min(result_int as u32, this_var_max_lvl)));
        }

        Ok(function_table)
    }
}

/// Generate all possible input combinations for the given variables, respecting their
/// possible levels.
///
/// This function can handle multi-valued variables (arg `max_levels` specifies maximum
/// level for each variable).
pub fn generate_input_combinations(
    variables: &[u32],
    max_levels: &HashMap<u32, u32>,
) -> Vec<HashMap<u32, Rational32>> {
    let mut results = Vec::new();
    let mut current_combination = HashMap::new();
    generate_input_combinations_rec(
        variables,
        max_levels,
        &mut current_combination,
        0,
        &mut results,
    );
    results
}

/// Recursive helper function to generate input value combinations.
/// It builds combinations by iterating through each variable and its possible levels.
///
/// This function can handle multi-valued variables (arg `max_levels` specifies maximum
/// level for each variable).
pub fn generate_input_combinations_rec(
    variables: &[u32],
    max_levels: &HashMap<u32, u32>,
    current: &mut HashMap<u32, Rational32>,
    index: usize,
    results: &mut Vec<HashMap<u32, Rational32>>,
) {
    if index == variables.len() {
        results.push(current.clone());
        return;
    }

    let var_id = &variables[index];
    let max_level = max_levels.get(var_id).cloned().unwrap_or(0);

    for level in 0..=max_level {
        current.insert(*var_id, Rational32::new(level as i32, 1));
        generate_input_combinations_rec(variables, max_levels, current, index + 1, results);
    }
}

#[cfg(test)]
mod tests {
    use crate::update_fn::parser::parse_bma_formula;
    use num_rational::Rational32;
    use std::collections::HashMap;

    #[test]
    fn test_evaluate_terminal_str() {
        let vars = HashMap::from([(1, "x".to_string())]);
        let expression = parse_bma_formula("var(x)", &vars).unwrap();
        let valuation = HashMap::from([(1, Rational32::new(5, 1))]);
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_terminal_int() {
        let vars = HashMap::new();
        let expression = parse_bma_formula("7", &vars).unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(7, 1));
    }

    #[test]
    fn test_evaluate_arithmetic_plus() {
        let vars = HashMap::new();
        let expression = parse_bma_formula("2 + 3", &vars).unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_arithmetic_mult() {
        let vars = HashMap::from([(1, "x".to_string())]);
        let expression = parse_bma_formula("4 * var(x)", &vars).unwrap();
        let valuation = HashMap::from([(1, Rational32::new(2, 1))]);
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(8, 1));
    }

    #[test]
    fn test_evaluate_unary_abs() {
        let vars = HashMap::new();
        let expression = parse_bma_formula("abs(5 - 10)", &vars).unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_aggregation_avg() {
        let vars = HashMap::new();
        let expression = parse_bma_formula("avg(1, 2, 3)", &vars).unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(2, 1));
    }

    #[test]
    fn test_evaluate_aggregation_max() {
        let vars = HashMap::new();
        let expression = parse_bma_formula("max(1, 4, 3)", &vars).unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(4, 1));
    }

    #[test]
    fn test_evaluate_aggregation_min() {
        let vars = HashMap::new();
        let expression = parse_bma_formula("min(1, 2 - 4, 3)", &vars).unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(-2, 1));
    }
}
