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
    pub fn to_update_fn(&self, max_levels: &HashMap<String, u32>) -> FnUpdate {
        let mut variables: Vec<String> = self.collect_variables().into_iter().collect();
        variables.sort();
        let _function_table = self.build_function_table(&variables, max_levels);

        todo!()
    }

    /// Evaluate the BMA function expression in a given valuation.
    /// A valuation assigns values to all variables.
    pub fn evaluate_in_valuation(
        &self,
        valuation: &HashMap<String, Rational32>,
    ) -> Result<Rational32, String> {
        match &self.expression_tree {
            BmaFnNodeType::Terminal(Literal::Str(name)) => {
                if let Some(value) = valuation.get(name) {
                    Ok(*value)
                } else {
                    Err(format!("Variable `{name}` not found in the valuation."))
                }
            }
            BmaFnNodeType::Terminal(Literal::Int(value)) => Ok(Rational32::new(*value, 1)),
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

    fn collect_variables(&self) -> HashSet<String> {
        match &self.expression_tree {
            BmaFnNodeType::Terminal(Literal::Str(name)) => {
                let mut set = HashSet::new();
                set.insert(name.clone());
                set
            }
            BmaFnNodeType::Terminal(Literal::Int(_)) => HashSet::new(),
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

    /// Build a function table that maps all input combinations (valuations) to output values.
    pub fn build_function_table(
        &self,
        variables: &[String],
        max_levels: &HashMap<String, u32>,
    ) -> Vec<(HashMap<String, Rational32>, Rational32)> {
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

/// Generate all possible input combinations for the given variables, respecting their possible levels.
pub fn generate_input_combinations(
    variables: &[String],
    max_levels: &HashMap<String, u32>,
) -> Vec<HashMap<String, Rational32>> {
    let mut results = Vec::new();
    let mut current_combination = HashMap::new();
    recursive_combinations(
        variables,
        max_levels,
        &mut current_combination,
        0,
        &mut results,
    );
    results
}

/// Recursive helper function to generate input combinations.
pub fn recursive_combinations(
    variables: &[String],
    max_levels: &HashMap<String, u32>,
    current: &mut HashMap<String, Rational32>,
    index: usize,
    results: &mut Vec<HashMap<String, Rational32>>,
) {
    if index == variables.len() {
        results.push(current.clone());
        return;
    }

    let var_name = &variables[index];
    let max_level = max_levels.get(var_name).cloned().unwrap_or(0);

    for level in 0..=max_level {
        current.insert(var_name.clone(), Rational32::new(level as i32, 1));
        recursive_combinations(variables, max_levels, current, index + 1, results);
    }
}

#[cfg(test)]
mod tests {
    use crate::update_fn::parser::parse_bma_formula;
    use num_rational::Rational32;
    use std::collections::HashMap;

    #[test]
    fn test_evaluate_terminal_str() {
        let expression = parse_bma_formula("x").unwrap();
        let valuation = HashMap::from([("x".to_string(), Rational32::new(5, 1))]);
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_terminal_int() {
        let expression = parse_bma_formula("7").unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(7, 1));
    }

    #[test]
    fn test_evaluate_arithmetic_plus() {
        let expression = parse_bma_formula("2 + 3").unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_arithmetic_mult() {
        let expression = parse_bma_formula("4 * x").unwrap();
        let valuation = HashMap::from([("x".to_string(), Rational32::new(2, 1))]);
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(8, 1));
    }

    #[test]
    fn test_evaluate_unary_abs() {
        let expression = parse_bma_formula("abs(5 - 10)").unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_aggregation_avg() {
        let expression = parse_bma_formula("avg(1, 2, 3)").unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(2, 1));
    }

    #[test]
    fn test_evaluate_aggregation_max() {
        let expression = parse_bma_formula("max(1, 4, 3)").unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(4, 1));
    }

    #[test]
    fn test_evaluate_aggregation_min() {
        let expression = parse_bma_formula("min(1, 2 - 4, 3)").unwrap();
        let valuation = HashMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(-2, 1));
    }
}
