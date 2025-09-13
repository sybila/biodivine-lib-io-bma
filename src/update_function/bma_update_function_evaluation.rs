use crate::update_function::BmaExpressionNodeData::Terminal;
use crate::update_function::{
    AggregateFn, ArithOp, BmaExpressionNodeData, BmaUpdateFunction, Literal, UnaryFn,
};
use crate::{BmaNetwork, BmaVariable};
use anyhow::anyhow;
use num_traits::Zero;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy::MidpointAwayFromZero;
use std::cmp::{max, min};
use std::collections::{BTreeMap, HashSet};

/// A function table is a vector of tuples, where each tuple contains a variable valuation
/// and output value. Variable valuation is a mapping of variable IDs to their values. In theory,
/// a valid function table should contain all possible combinations of variable values that are
/// admissible in the associated BMA model (within their prescribed ranges). Also, the output
/// value should always be within the prescribed variable range for the associated variable.
///
/// Note that even though the function table only contains integers ("levels"), the actual
/// computation within the update function can involve
pub type FunctionTable = Vec<(BTreeMap<u32, u32>, u32)>;

impl BmaNetwork {
    /// Evaluate the BMA function expression assigned to the given variable. The result is a level
    /// within the allowed range of this variable (the value is truncated if it does not fit
    /// within this range) and the function performs all necessary normalization steps on
    /// the input levels. A `valuation` assigns values to all variables (ID-value mapping).
    ///
    /// *The operation fails if any of the following happens:*
    ///  - Any of the specified source/target variables does not exist in this model.
    ///  - The variable has no update function.
    ///  - The variable has an update function which is in the error state.
    ///  - The valuation does not contain all necessary values.
    ///  - Invalid arithmetic operation occurs (e.g., division by zero).
    ///
    /// See also: [`BmaNetwork::set_default_function`], [`BmaNetwork::populate_missing_functions`],
    /// [`BmaVariable::normalize_input_level`] and [`BmaUpdateFunction::evaluate_raw`].
    pub fn evaluate(&self, var_id: u32, valuation: &BTreeMap<u32, u32>) -> anyhow::Result<u32> {
        let target_var = self
            .find_variable(var_id)
            .ok_or_else(|| anyhow!("Target variable with id `{var_id}` not found"))?;

        let mut normalized_valuation = BTreeMap::new();
        for (source_id, level) in valuation {
            let source_var = self
                .find_variable(*source_id)
                .ok_or_else(|| anyhow!("Source variable with id `{source_id}` not found"))?;
            let normalized_level = target_var.normalize_input_level(source_var, *level);
            normalized_valuation.insert(*source_id, normalized_level);
        }

        if let Some(function) = &target_var.formula {
            let function = function.as_ref().map_err(|e| anyhow!(e.to_string()))?;
            let raw_result = function.evaluate_raw(&normalized_valuation)?;
            Ok(target_var.normalize_output_level(raw_result))
        } else {
            Err(anyhow!("No update function found for `{var_id}`"))
        }
    }

    /// Build a complete [`FunctionTable`] with all input-output combinations.
    ///
    /// If the update function is missing, the function computes a function table for the
    /// "default" update function (see [`BmaNetwork::build_default_update_function`]).
    ///
    /// The function can fail if the variable does not exist, its update function is in an
    /// error state, contains invalid variables, or performs division by zero.
    /// See also [`BmaNetwork::evaluate`] for possible error states.
    ///
    /// *[`FunctionTable`] will use inputs declared as regulators in
    /// [`BmaNetwork`], so the network has to be correctly configured.*
    ///
    /// For constant variables, the update function always contains exactly one row, and the
    /// output for that row is either the sole value in the variable's domain, or `0`.
    ///
    pub fn build_function_table(&self, var_id: u32) -> anyhow::Result<FunctionTable> {
        let target_var = self
            .find_variable(var_id)
            .ok_or_else(|| anyhow!("Target variable with id `{var_id}` not found"))?;

        let function = match &target_var.formula {
            None => self.build_default_update_function(var_id),
            Some(function) => function
                .as_ref()
                .cloned()
                .map_err(|e| anyhow!(e.to_string()))?,
        };

        // Regulators declared in the model, not what actually appears in function.
        let mut regulators_map = BTreeMap::new();
        for id in self.get_regulators(var_id, &None) {
            let var = self
                .find_variable(id)
                .ok_or_else(|| anyhow!("Regulator variable `{id}` does not exist"))?;
            regulators_map.insert(id, var);
        }

        if target_var.has_constant_range() {
            // For constant variables, the update function is built a bit differently, because
            // we technically allow them to be 0 even if that value is outside variable range.

            if !regulators_map.is_empty() {
                return Err(anyhow!("Constant variable cannot have regulators."));
            }

            let const_level = target_var.min_level();
            let output = match function.as_constant() {
                Some(value) => {
                    let Ok(value) = u32::try_from(value) else {
                        return Err(anyhow!("Constant value cannot be negative."));
                    };
                    if value == 0 || value == const_level {
                        value
                    } else {
                        return Err(anyhow!("Constant value does not match variable level."));
                    }
                }
                _ => return Err(anyhow!("Non-constant function in constant variable.")),
            };

            Ok(vec![(BTreeMap::new(), output)])
        } else {
            target_var.build_function_table(&function, &regulators_map)
        }
    }
}

impl BmaVariable {
    /// Normalizes the input level of the given [`BmaVariable`] such that it is compatible
    /// with the range of this variable (see also the discussion in
    /// [`BmaUpdateFunction::evaluate_raw`]).
    #[must_use]
    pub fn normalize_input_level(&self, input: &BmaVariable, value: u32) -> Decimal {
        // input \in [a,b]
        // self \in [c,d]
        // (value-a)*(d-c)/(b-a)+c
        let value = i64::from(value);
        let (a, b) = (i64::from(input.min_level()), i64::from(input.max_level()));
        let (c, d) = (i64::from(self.min_level()), i64::from(self.max_level()));

        if a == b {
            // For constants, the value is always taken as is.
            return Decimal::from(value);
        }
        let numerator = (value - a) * (d - c);
        let denominator = b - a;
        (Decimal::from(numerator) / Decimal::from(denominator)) + Decimal::from(c)
    }

    /// Normalize the output level of this variable. This means (a) round the output correctly,
    /// (b) truncate it to the range of this variable.
    #[must_use]
    pub fn normalize_output_level(&self, value: Decimal) -> u32 {
        let (low, high) = (i64::from(self.min_level()), i64::from(self.max_level()));
        // BMA seems to be using round half up / round half away from zero convention, which
        // is also implemented here. However, if you see any weird behavior in your results,
        // it may be good to make sure this is actually the correct rounding.
        let raw_result = value.round_dp_with_strategy(0, MidpointAwayFromZero);
        let raw_result = i64::try_from(raw_result)
            .expect("Invariant violation: Rounded output level is not a 64-bit number.");

        // The `u32` conversion must succeed because the number now fits into the bounds
        // of the variable, which are defined as `u32` values.
        let trunc_result = max(min(raw_result, high), low);
        u32::try_from(trunc_result).expect("Invariant violation: Result must fit into `u32`")
    }

    /// Internal version of [`BmaModel::build_function_table`] which assumes you already have
    /// some of the inputs pre-computed.
    pub(crate) fn build_function_table(
        &self,
        function: &BmaUpdateFunction,
        regulators_map: &BTreeMap<u32, &BmaVariable>,
    ) -> anyhow::Result<FunctionTable> {
        let regulators: Vec<_> = regulators_map.values().copied().collect();

        let mut table = Vec::new();
        for valuation in generate_input_valuations(&regulators) {
            let mut normalized_valuation = BTreeMap::new();
            for (source_id, level) in &valuation {
                let source_var = regulators_map
                    .get(source_id)
                    .expect("Invariant violation: Invalid regulator");
                let normalized_level = self.normalize_input_level(source_var, *level);
                normalized_valuation.insert(*source_id, normalized_level);
            }

            let raw_result = function.evaluate_raw(&normalized_valuation)?;

            table.push((valuation, self.normalize_output_level(raw_result)));
        }

        Ok(table)
    }
}

impl BmaUpdateFunction {
    /// Collect all variable IDs used in this BMA function expression.
    #[must_use]
    pub fn collect_variables(&self) -> HashSet<u32> {
        fn collect_rec(function: &BmaUpdateFunction, result: &mut HashSet<u32>) {
            match &function.as_data() {
                Terminal(Literal::Var(var_id)) => {
                    result.insert(*var_id);
                }
                Terminal(Literal::Const(_)) => (),
                BmaExpressionNodeData::Arithmetic(_, left, right) => {
                    collect_rec(left, result);
                    collect_rec(right, result);
                }
                BmaExpressionNodeData::Unary(_, child_node) => collect_rec(child_node, result),
                BmaExpressionNodeData::Aggregation(_, arguments) => {
                    for arg in arguments {
                        collect_rec(arg, result);
                    }
                }
            }
        }

        let mut result = HashSet::new();
        collect_rec(self, &mut result);
        result
    }

    /// Raw evaluation function which returns the rational value of the function expression
    /// without truncation to the valid variable interval. The function expects the valuation
    /// to be in the "normalized" format (the level of each variable is adjusted to the
    /// level of the target variable). This normalized format is explained as follows
    /// (adapted from FAQ on the BMA website; see also [`BmaVariable::normalize_input_level`]):
    ///
    /// ```text
    /// BioModelAnalyzer is built to facilitate the usage of the default target function.
    /// What should happen if I have a variable X with range [0,1] activating a variable
    /// Y with range [0,2] (and no other influences)? In this case, it makes sense that
    /// the maximal value of X should lead to Y having maximal value as well. This means
    /// that the ranges of X and Y need to be adjusted when they affect each other. So
    /// BioModelAnalyzer does an automatic range conversion when X appears in the target
    /// function of Y (or vice versa). If you just use the default target functions this
    /// is not something you have to worry about. It essentially means that the entire
    /// range of Y will be possible in the example above (and similar cases). However,
    /// if you are building your own target functions this may lead to unexpected behaviour.
    ///
    /// The exact range conversion that BioModelAnalyzer applies is as follows. If a constant
    /// with range [n,n] appears in the target function of a variable with a different range,
    /// then no range conversion is applied. If a variable X with range [a,b] appears in the
    /// target function of a variable Y with range [c,d], then whenever X appears in the target
    /// function of Y it will be modified to (X-a)*(d-c)/(b-a)+c.
    /// ```
    ///
    /// The function returns an error if the `valuation` does not contain all required variables,
    /// if there is division by zero, or if one of the aggregation operators has no arguments. Note
    /// that aggregation operations with no arguments should be caught as errors by the parser
    /// or constructor, but the user could make a custom function with no arguments.
    ///
    /// See also [`BmaNetwork::evaluate`].
    pub fn evaluate_raw(&self, valuation: &BTreeMap<u32, Decimal>) -> anyhow::Result<Decimal> {
        match &self.as_data() {
            Terminal(Literal::Const(value)) => Ok(Decimal::from(*value)),
            Terminal(Literal::Var(var_id)) => {
                if let Some(value) = valuation.get(var_id) {
                    Ok(*value)
                } else {
                    Err(anyhow!(format!(
                        "Missing input value for variable `{var_id}`"
                    )))
                }
            }
            BmaExpressionNodeData::Arithmetic(operator, left, right) => {
                let left_value = left.evaluate_raw(valuation)?;
                let right_value = right.evaluate_raw(valuation)?;
                let res = match operator {
                    ArithOp::Plus => left_value + right_value,
                    ArithOp::Minus => left_value - right_value,
                    ArithOp::Mult => left_value * right_value,
                    ArithOp::Div => {
                        if right_value == Decimal::zero() {
                            return Err(anyhow!("Division by zero"));
                        }
                        left_value / right_value
                    }
                };
                Ok(res)
            }
            BmaExpressionNodeData::Unary(function, child_node) => {
                let child_value = child_node.evaluate_raw(valuation)?;
                let res = match function {
                    UnaryFn::Abs => child_value.abs(),
                    UnaryFn::Ceil => child_value.ceil(),
                    UnaryFn::Floor => child_value.floor(),
                };
                Ok(res)
            }
            BmaExpressionNodeData::Aggregation(function, arguments) => {
                if arguments.is_empty() {
                    return Err(anyhow!(
                        "At least one argument is required for `{function}`"
                    ));
                }
                let arg_values = arguments
                    .iter()
                    .map(|arg| arg.evaluate_raw(valuation))
                    .collect::<anyhow::Result<Vec<_>>>()?;
                let res = match function {
                    AggregateFn::Avg => {
                        let count = i64::try_from(arg_values.len())
                            .expect("Invariant violation: Number of arguments is too large.");
                        let sum: Decimal = arg_values.iter().copied().sum();
                        sum / Decimal::from(count)
                    }
                    AggregateFn::Max => arg_values
                        .iter()
                        .copied()
                        .max()
                        .expect("Invariant violation: Missing arguments."),
                    AggregateFn::Min => arg_values
                        .iter()
                        .copied()
                        .min()
                        .expect("Invariant violation: Missing arguments."),
                };
                Ok(res)
            }
        }
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
fn generate_input_valuations(variables: &[&BmaVariable]) -> Vec<BTreeMap<u32, u32>> {
    fn generate_input_valuations_rec(
        variables: &[&BmaVariable],
        current: &mut BTreeMap<u32, u32>,
        results: &mut Vec<BTreeMap<u32, u32>>,
    ) {
        if variables.is_empty() {
            results.push(current.clone());
            return;
        }

        let variable = &variables[0];

        for level in variable.min_level()..=variable.max_level() {
            current.insert(variable.id, level);
            generate_input_valuations_rec(&variables[1..], current, results);
        }
    }

    let mut results = Vec::new();
    let mut current_valuation = BTreeMap::new();
    generate_input_valuations_rec(variables, &mut current_valuation, &mut results);

    results
}

#[cfg(test)]
mod tests {
    use crate::update_function::expression_parser::parse_bma_formula;
    use crate::update_function::tests::{and_model, complex_model};
    use crate::update_function::{BmaUpdateFunction, FunctionTable};
    use rust_decimal::Decimal;
    use std::collections::{BTreeMap, HashSet};

    /// Utility method for quickly building decimals.
    fn d(x: u32) -> Decimal {
        Decimal::from(x)
    }

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
        let expression = BmaUpdateFunction::parse_with_hint("var(x)", &vars).unwrap();
        let valuation = BTreeMap::from([(1, d(5))]);
        let result = expression.evaluate_raw(&valuation).unwrap();
        assert_eq!(result, d(5));
    }

    #[test]
    fn test_evaluate_terminal_int() {
        let expression = BmaUpdateFunction::try_from("7").unwrap();
        let valuation = BTreeMap::new();
        let result = expression.evaluate_raw(&valuation).unwrap();
        assert_eq!(result, d(7));
    }

    #[test]
    fn test_evaluate_arithmetic_plus() {
        let expression = BmaUpdateFunction::try_from("2 + 3").unwrap();
        let result = expression.evaluate_raw(&BTreeMap::default()).unwrap();
        assert_eq!(result, Decimal::from(5));
    }

    #[test]
    fn test_evaluate_arithmetic_mult() {
        let vars = vec![(1, "x".to_string())];
        let expression = BmaUpdateFunction::parse_with_hint("4 * var(x)", &vars).unwrap();
        let valuation = BTreeMap::from([(1, d(2))]);
        let result = expression.evaluate_raw(&valuation).unwrap();
        assert_eq!(result, d(8));
    }

    #[test]
    fn test_evaluate_unary_abs() {
        let expression = BmaUpdateFunction::try_from("abs(5 - 10)").unwrap();
        let result = expression.evaluate_raw(&BTreeMap::default()).unwrap();
        assert_eq!(result, d(5));
    }

    #[test]
    fn test_evaluate_aggregation_avg() {
        let expression = BmaUpdateFunction::try_from("avg(1, 2, 3)").unwrap();
        let result = expression.evaluate_raw(&BTreeMap::default()).unwrap();
        assert_eq!(result, d(2));
    }

    #[test]
    fn test_evaluate_aggregation_max() {
        let expression = BmaUpdateFunction::try_from("max(1, 4, 3)").unwrap();
        let result = expression.evaluate_raw(&BTreeMap::default()).unwrap();
        assert_eq!(result, d(4));
    }

    #[test]
    fn test_evaluate_aggregation_min() {
        let expression = BmaUpdateFunction::try_from("min(1, 2 - 4, 3)").unwrap();
        let result = expression.evaluate_raw(&BTreeMap::default()).unwrap();
        assert_eq!(result, Decimal::from(-2));
    }

    #[test]
    fn test_build_fn_table_binary_and() {
        let model = and_model();

        let result_table = model.network.build_function_table(1).unwrap();
        let expected_table = prepare_truth_table(&[1, 2], &[0, 0, 0, 1]);

        assert_eq!(result_table, expected_table);
    }

    #[test]
    fn test_build_fn_table_ternary() {
        let model = complex_model();

        let expected_table = prepare_truth_table(&[1, 2, 3], &[1, 0, 0, 0, 1, 1, 1, 1]);
        let result_table = model.network.build_function_table(1).unwrap();

        assert_eq!(result_table, expected_table);
    }

    /// A simple wrapper to easily put together a boolean `FunctionTable` (a truth table).
    /// This is meant to be used for testing purposes.
    ///
    /// You provide a vector of N variable IDs (will be sorted, so ideally sort beforehand
    /// already) and a vector of 2^N function values (0 or 1).
    /// The table starts with zero valuation at index 0, and going up to the ones valuation,
    /// last variable updates first. For instance, in binary case, valuations are generated in
    /// the order: 00, 01, 10, 11.
    fn prepare_truth_table(var_ids: &[u32], fn_values: &[u32]) -> FunctionTable {
        let mut function_table = Vec::new();
        let mut var_ids = var_ids.to_vec();
        var_ids.sort_unstable();
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
                valuation.insert(*var_id, u32::try_from(value).unwrap());
            }
            function_table.push((valuation, *fn_value));
        }
        function_table
    }
}
