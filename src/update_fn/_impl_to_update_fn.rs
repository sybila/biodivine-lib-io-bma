use crate::update_fn::bma_fn_tree::{BmaFnNodeType, BmaFnUpdate};
use crate::update_fn::expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
use biodivine_lib_param_bn::FnUpdate;
use num_rational::Rational32;
use num_traits::sign::Signed;
use std::collections::{HashMap, HashSet};

impl BmaFnUpdate {
    /// Convert the BMA expression into corresponding `FnUpdate` instance of
    /// [biodivine_lib_param_bn] library.
    ///
    /// TODO: implementation via explicit construction of the function table
    pub fn to_update_fn(&self, max_levels: &HashMap<u32, u32>) -> FnUpdate {
        // To convert the BMA expression into an update function, we essetially create
        // an explicit function table mapping all valuations of inputs to output values.
        // In BNs, this corresponds to a truth table.
        // We can then use this function table to create a new `FnUpdate` instance.

        let mut variables: Vec<u32> = self.collect_variables().into_iter().collect();
        variables.sort();
        let _function_table = self.build_function_table(&variables, max_levels);

        todo!()
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

    /// Build a "function table" mapping all input value combinations (valuations)
    /// to output values.
    ///
    /// For Boolean networks, the function table will essentially be a truth table,
    /// mapping boolean combinations of input variables to the output value.
    ///
    /// This method can also handle multi-valued variables (arg `max_levels` specifies
    /// maximum level for each variable), but the table needs to be further binarized
    /// to be used in a Boolean network.
    pub fn build_function_table(
        &self,
        variables: &[u32],
        max_levels: &HashMap<u32, u32>,
    ) -> Vec<(HashMap<u32, Rational32>, Rational32)> {
        let input_combinations = generate_input_combinations(variables, max_levels);
        let mut function_table = Vec::new();

        // Evaluate the function for each combination.
        for combination in input_combinations {
            match self.evaluate_in_valuation(&combination) {
                Ok(output_value) => function_table.push((combination, output_value)),
                Err(err) => eprintln!("Error evaluating function: {err}"),
            }
        }

        function_table
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
