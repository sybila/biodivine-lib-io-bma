use crate::update_function::bma_fn_update::{BmaUpdateFunction, BmaUpdateFunctionNode};
use crate::update_function::expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
use biodivine_lib_param_bn::{FnUpdate, VariableId};
use num_rational::Rational32;
use num_traits::sign::Signed;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, dec};
use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, HashSet};

/// A function table is a vector of tuples, where each tuple contains a variable valuation
/// and output value. Variable valuation is a mapping of variable IDs to their values (as
/// a HashMap).
type FunctionTable = Vec<(BTreeMap<u32, u32>, u32)>;

impl BmaUpdateFunction {
    /// Convert the BMA expression into corresponding BN update function string
    /// matching the format of the [biodivine_lib_param_bn] library.
    ///
    /// Note that currently, WE ONLY SUPPORT BOOLEAN MODELS, even though some methods
    /// are already implemented to handle more general multivalued cases as well.
    ///
    /// Map `max_levels` indicates the maximum level for each variable in the model. For
    /// Boolean networks, this is set to 1 for all variables.
    /// Arg `var_name_mapping` maps each BMA variable ID to its canonical name in the
    /// constructed BN.
    /// Arg `this_var_max_lvl` is the maximum level of the variable for which we are  
    /// creating the update function.
    pub fn to_update_fn_boolean(
        &self,
        max_levels: &HashMap<u32, u32>,
        var_bma_to_aeon: &HashMap<u32, VariableId>,
        this_var_max_lvl: u32,
    ) -> Result<FnUpdate, String> {
        // To convert the BMA expression into an update function, we essentially create
        // an explicit function table mapping all valuations of inputs to output values.
        // In BNs, this corresponds to a truth table.
        // We can then use this function table to create a new logical update formula.

        // Collect all variable IDs used in the expression, and sort them
        let mut variables_in_fn: Vec<u32> = self.collect_variables().into_iter().collect();
        variables_in_fn.sort();

        // Create a function table and convert it into DNF formula
        let function_table =
            self.build_function_table(&variables_in_fn, max_levels, this_var_max_lvl)?;
        let mut conjunctive_clauses = Vec::new();
        for (valuation, fn_value) in function_table {
            if fn_value == 1 {
                let literals = valuation
                    .iter()
                    .map(|(id, value)| {
                        let aeon_var = var_bma_to_aeon.get(id).unwrap(); // unwrap is safe here
                        // create positive or negative literal based on the value
                        if *value == 0 {
                            FnUpdate::mk_not(FnUpdate::mk_var(*aeon_var))
                        } else {
                            FnUpdate::mk_var(*aeon_var)
                        }
                    })
                    .collect::<Vec<FnUpdate>>();
                conjunctive_clauses.push(FnUpdate::mk_conjunction(&literals));
            }
        }

        let dnf_formula = if conjunctive_clauses.is_empty() {
            FnUpdate::mk_false()
        } else {
            FnUpdate::mk_disjunction(&conjunctive_clauses)
        };

        Ok(dnf_formula)
    }

    /// Evaluate the BMA function expression in a given valuation.
    /// A `valuation`` assigns values to all variables (ID-value mapping).
    pub fn evaluate_in_valuation(
        &self,
        valuation: &BTreeMap<u32, Rational32>,
    ) -> Result<Rational32, String> {
        match &self.expression_tree {
            BmaUpdateFunctionNode::Terminal(Literal::Var(var_id)) => {
                if let Some(value) = valuation.get(var_id) {
                    Ok(*value)
                } else {
                    Err(format!("Variable `{var_id}` not found in the valuation."))
                }
            }
            BmaUpdateFunctionNode::Terminal(Literal::Const(value)) => {
                Ok(Rational32::new(*value, 1))
            }
            BmaUpdateFunctionNode::Arithmetic(operator, left, right) => {
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
            BmaUpdateFunctionNode::Unary(function, child_node) => {
                let child_value = child_node.evaluate_in_valuation(valuation)?;
                let res = match function {
                    UnaryFn::Abs => Rational32::abs(&child_value),
                    UnaryFn::Ceil => Rational32::ceil(&child_value),
                    UnaryFn::Floor => Rational32::floor(&child_value),
                };
                Ok(res)
            }
            BmaUpdateFunctionNode::Aggregation(function, arguments) => {
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
            BmaUpdateFunctionNode::Terminal(Literal::Var(var_id)) => {
                let mut set = HashSet::new();
                set.insert(*var_id);
                set
            }
            BmaUpdateFunctionNode::Terminal(Literal::Const(_)) => HashSet::new(),
            BmaUpdateFunctionNode::Arithmetic(_, left, right) => {
                let left_set = left.collect_variables();
                let right_set = right.collect_variables();
                left_set.union(&right_set).cloned().collect()
            }
            BmaUpdateFunctionNode::Unary(_, child_node) => child_node.collect_variables(),
            BmaUpdateFunctionNode::Aggregation(_, arguments) => arguments
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
    /// This method can also handle multivalued variables (arg `max_levels` specifies
    /// maximum level for all model variable), but the table needs to be further binarized
    /// to be used in a Boolean network.
    pub fn build_function_table(
        &self,
        variables_in_fn: &[u32],
        max_levels: &HashMap<u32, u32>,
        this_var_max_lvl: u32,
    ) -> Result<FunctionTable, String> {
        let input_valuations = generate_input_valuations(variables_in_fn, max_levels);
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
                .collect::<BTreeMap<u32, u32>>();

            // Convert the result to integer (rounding if necessary)
            let mut result_int = if rational_result.is_integer() {
                // Ideally, most numbers will not actually be fractions, and we don't have to round
                rational_result.to_integer()
            } else {
                // Otherwise, we need to convert the fraction into integer by rounding.
                // Note that BMA is written in C/C# which performs "round half up" arithmetic.
                // However, Rust performs "round half even" arithmetic, meaning we might return
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
/// This function can handle multivalued variables (arg `max_levels` specifies maximum
/// level for each variable).
///
/// The valuations are generated starting at 0, and going up to the maximum level, last
/// variable first. For instance, in binary case, valuations are generated in the order:
/// 00, 01, 10, 11.
fn generate_input_valuations(
    variables: &[u32],
    max_levels: &HashMap<u32, u32>,
) -> Vec<BTreeMap<u32, Rational32>> {
    let mut results = Vec::new();
    let mut current_valuation = BTreeMap::new();
    generate_input_valuations_rec(
        variables,
        max_levels,
        &mut current_valuation,
        0,
        &mut results,
    );
    results
}

/// Recursive helper function to generate input value combinations.
/// It builds combinations by iterating through each variable and its possible levels.
///
/// This function can handle multivalued variables (arg `max_levels` specifies maximum
/// level for each variable).
fn generate_input_valuations_rec(
    variables: &[u32],
    max_levels: &HashMap<u32, u32>,
    current: &mut BTreeMap<u32, Rational32>,
    index: usize,
    results: &mut Vec<BTreeMap<u32, Rational32>>,
) {
    if index == variables.len() {
        results.push(current.clone());
        return;
    }

    let var_id = &variables[index];
    let max_level = max_levels.get(var_id).cloned().unwrap_or(0);

    for level in 0..=max_level {
        current.insert(*var_id, Rational32::new(level as i32, 1));
        generate_input_valuations_rec(variables, max_levels, current, index + 1, results);
    }
}

/// A simple wrapper to easily put together a boolean FunctionTable (a truth table).
/// This is meant to be used for testing purposes.
///
/// You provide a vector of N variable IDs (will be sorted, so ideally sort beforehand
/// already) and a vector of 2^N function values (0 or 1).
/// The table starts with zero valuation at index 0, and going up to the ones valuation,
/// last variable updates first. For instance, in binary case, valuations are generated in
/// the order: 00, 01, 10, 11.
#[allow(dead_code)]
pub fn prepare_truth_table(mut var_ids: Vec<u32>, fn_values: Vec<u32>) -> FunctionTable {
    let mut function_table = Vec::new();
    var_ids.sort();
    let num_vars = var_ids.len();
    let num_rows = 1 << num_vars; // 2^N
    assert_eq!(
        fn_values.len(),
        num_rows,
        "Function values length does not match the number of rows."
    );

    for (i, fn_value) in fn_values.iter().enumerate().take(num_rows) {
        let mut valuation = BTreeMap::new();
        for (j, var_id) in var_ids.iter().rev().enumerate().take(num_vars) {
            let value = (i >> j) & 1;
            valuation.insert(*var_id, value as u32);
        }
        function_table.push((valuation, *fn_value));
    }
    function_table
}

#[cfg(test)]
mod tests {
    use crate::update_function::{
        _impl_to_update_fn::prepare_truth_table, parser::parse_bma_formula,
    };
    use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate, RegulatoryGraph, VariableId};
    use num_rational::Rational32;
    use std::collections::{BTreeMap, HashMap, HashSet};

    #[test]
    fn test_collect_variables() {
        let vars = vec![
            (1, "a".to_string()),
            (2, "b".to_string()),
            (3, "c".to_string()),
        ];

        // this one references all three variables by IDs
        let expression =
            parse_bma_formula("var(1) + (1 - min((var(2) + var(3)), 1))", &vars).unwrap();
        assert_eq!(expression.collect_variables(), HashSet::from([1, 2, 3]));

        // this one references all three variables by names
        let expression =
            parse_bma_formula("var(a) + (1 - min((var(b) + var(c)), 1))", &vars).unwrap();
        assert_eq!(expression.collect_variables(), HashSet::from([1, 2, 3]));

        // this one only references two variables
        let expression = parse_bma_formula("(1 - min((var(b) + var(c)), 1))", &vars).unwrap();
        assert_eq!(expression.collect_variables(), HashSet::from([2, 3]));
    }

    #[test]
    fn test_evaluate_terminal_str() {
        let vars = vec![(1, "x".to_string())];
        let expression = parse_bma_formula("var(x)", &vars).unwrap();
        let valuation = BTreeMap::from([(1, Rational32::new(5, 1))]);
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_terminal_int() {
        let expression = parse_bma_formula("7", &[]).unwrap();
        let valuation = BTreeMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(7, 1));
    }

    #[test]
    fn test_evaluate_arithmetic_plus() {
        let expression = parse_bma_formula("2 + 3", &[]).unwrap();
        let valuation = BTreeMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_arithmetic_mult() {
        let vars = vec![(1, "x".to_string())];
        let expression = parse_bma_formula("4 * var(x)", &vars).unwrap();
        let valuation = BTreeMap::from([(1, Rational32::new(2, 1))]);
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(8, 1));
    }

    #[test]
    fn test_evaluate_unary_abs() {
        let expression = parse_bma_formula("abs(5 - 10)", &[]).unwrap();
        let valuation = BTreeMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(5, 1));
    }

    #[test]
    fn test_evaluate_aggregation_avg() {
        let expression = parse_bma_formula("avg(1, 2, 3)", &[]).unwrap();
        let valuation = BTreeMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(2, 1));
    }

    #[test]
    fn test_evaluate_aggregation_max() {
        let expression = parse_bma_formula("max(1, 4, 3)", &[]).unwrap();
        let valuation = BTreeMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(4, 1));
    }

    #[test]
    fn test_evaluate_aggregation_min() {
        let expression = parse_bma_formula("min(1, 2 - 4, 3)", &[]).unwrap();
        let valuation = BTreeMap::new();
        let result = expression.evaluate_in_valuation(&valuation);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Rational32::new(-2, 1));
    }

    #[test]
    fn test_build_fn_table_binary_and() {
        // prepare 2 boolean variables and a formula for their product
        let vars = vec![(1, "a".to_string()), (2, "b".to_string())];
        let max_levels = HashMap::from([(1, 1), (2, 1)]);
        let expression = parse_bma_formula("var(1) * var(2)", &vars).unwrap();

        let expected_table = prepare_truth_table(vec![1, 2], vec![0, 0, 0, 1]);
        let result_table = expression.build_function_table(&[1, 2], &max_levels, 1);

        assert!(result_table.is_ok());
        assert_eq!(result_table.unwrap(), expected_table);
    }

    #[test]
    fn test_build_fn_table_ternary() {
        // prepare 3 boolean variables and a formula for A | !(B | C)
        let vars = vec![
            (1, "a".to_string()),
            (2, "b".to_string()),
            (3, "c".to_string()),
        ];

        let max_levels = HashMap::from([(1, 1), (2, 1), (3, 1)]);
        let expression =
            parse_bma_formula("var(1) + (1 - min((var(2) + var(3)), 1))", &vars).unwrap();

        let expected_table = prepare_truth_table(vec![1, 2, 3], vec![1, 0, 0, 0, 1, 1, 1, 1]);
        let result_table = expression.build_function_table(&[1, 2, 3], &max_levels, 1);

        assert!(result_table.is_ok());
        assert_eq!(result_table.unwrap(), expected_table);
    }

    #[test]
    fn test_to_update_fn_boolean_binary() {
        // prepare 2 boolean variables and a formula for their product
        let vars = vec![(1, "a".to_string()), (2, "b".to_string())];
        let max_levels = HashMap::from([(1, 1), (2, 1)]);
        let expression = parse_bma_formula("var(1) * var(2)", &vars).unwrap();

        // DNF formula for the AND function is just "(a & b)" - only this one clause has function value 1
        let vars = HashMap::from([
            (1, VariableId::from_index(0)),
            (2, VariableId::from_index(1)),
        ]);
        let result_fn = expression.to_update_fn_boolean(&max_levels, &vars, 1);

        let dummy_rg = RegulatoryGraph::new(vec!["a".to_string(), "b".to_string()]);
        let dummy_bn = BooleanNetwork::new(dummy_rg);
        let expected_fn = FnUpdate::try_from_str("(a & b)", &dummy_bn).unwrap();

        assert!(result_fn.is_ok());
        assert_eq!(result_fn.unwrap(), expected_fn);
    }

    #[test]
    fn test_to_update_fn_boolean_ternary() {
        // prepare 3 boolean variables and a formula for A | !(B | C)
        let vars = vec![
            (0, "a".to_string()),
            (0, "b".to_string()),
            (0, "c".to_string()),
        ];

        let max_levels = HashMap::from([(1, 1), (2, 1), (3, 1)]);
        let expression =
            parse_bma_formula("var(1) + (1 - min((var(2) + var(3)), 1))", &vars).unwrap();

        // expected function values are [1, 0, 0, 0, 1, 1, 1, 1]
        // that means DNF formula with 5 clauses (starting from zero valuation)

        let dummy_rg =
            RegulatoryGraph::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        let dummy_bn = BooleanNetwork::new(dummy_rg);
        let expected_fn =
            "(!a & !b & !c) | (a & !b & !c) | (a & !b & c) | (a & b & !c) | (a & b & c)";
        let expected_fn = FnUpdate::try_from_str(expected_fn, &dummy_bn).unwrap();
        let vars = HashMap::from([
            (1, VariableId::from_index(0)),
            (2, VariableId::from_index(1)),
            (3, VariableId::from_index(2)),
        ]);
        let result_fn = expression.to_update_fn_boolean(&max_levels, &vars, 1);

        assert!(result_fn.is_ok());
        assert_eq!(result_fn.unwrap(), expected_fn);
    }
}
