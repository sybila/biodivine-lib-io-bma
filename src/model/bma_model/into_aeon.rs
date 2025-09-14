use crate::update_function::FunctionTable;
use crate::{BmaModel, BmaVariable};
use anyhow::anyhow;
use biodivine_lib_bdd::{
    Bdd, BddPartialValuation, BddVariable, BddVariableSet, BddVariableSetBuilder,
};
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate, Regulation, RegulatoryGraph, VariableId};
use std::collections::HashMap;
use std::ops::RangeInclusive;

/// Symbolic update function stores a [`Bdd`] condition for each output level of
/// a specific update function. The conditions should be mutually exclusive and exhaustive
/// (i.e. each input valuation satisfies exactly one of the stored BDDs). Levels should
/// represent a continuous interval, such that one BDD is given for every value from the
/// corresponding variable domain.
#[derive(Clone)]
struct SymbolicUpdateFunction(Vec<(u32, Bdd)>);

/// Allows encoding multivalued variables as symbolic states.
///
/// The encoding is unary, such that each variable is assigned to `|domain| - 1` symbolic
/// variables. The value of the variable is then the value of the highest active "bit",
/// although it is assumed that all smaller bits should be set as well (states where
/// this does not happen are generally considered invalid).
///
/// ```text
/// (0,0,0) => 0
/// (1,0,0) => 1
/// (1,1,0) => 2
/// (1,1,1) => 3
/// (0,1,0) => "2-like" (but can update to `(1,1,0)`)
/// ```
///
#[derive(Clone)]
pub(crate) struct SymbolicVariable {
    // Directly taken from the BMA variable.
    id: u32,
    // Minimum and maximum value (inclusive).
    range: (u32, u32),
    // BDD variables corresponding to each level except for the minimum.
    bdd_vars: Vec<BddVariable>,
}

impl SymbolicVariable {
    /// Make a new symbolic variable matching the given BMA variable while using the given
    /// BDD variables to encode the levels.
    ///
    /// The number of symbolic variables must be exactly the number of levels minus one, except
    /// for constants, where one variable is expected.
    pub(crate) fn new(var: &BmaVariable, bdd_vars: Vec<BddVariable>) -> SymbolicVariable {
        if var.has_constant_range() {
            assert_eq!(bdd_vars.len(), 1);
        } else {
            assert_eq!((var.range.1 - var.range.0) as usize, bdd_vars.len());
        }
        SymbolicVariable {
            id: var.id,
            range: var.range,
            bdd_vars,
        }
    }
}

/// Encodes the dynamics of a multivalued model using symbolic variables.
///
/// Do not confuse with [`biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext`],
/// which does something very similar. However, for now, this structure is for internal use
/// only. We might use this implementation as a base for some future "multivalued context".
#[derive(Clone)]
struct SymbolicContext {
    bdd_ctx: BddVariableSet,
    variables: Vec<(SymbolicVariable, SymbolicUpdateFunction)>,
}

// In this module, we assume that by construction, BDD variables and network
// variables have the same indices. It means we can safely do this:
fn cast_id(bdd_variable: BddVariable) -> VariableId {
    VariableId::from_index(bdd_variable.to_index())
}

/// Convert [`BmaModel`] into a [`BooleanNetwork`] instance, binarizing any multivalued variables.
///
/// A valid regulatory graph is inferred at the end of the conversion, because the binarization
/// tends to mess with the relationships anyway.
impl TryFrom<&BmaModel> for BooleanNetwork {
    type Error = anyhow::Error;

    fn try_from(model: &BmaModel) -> Result<Self, Self::Error> {
        let context = SymbolicContext::try_from(model)?;
        BooleanNetwork::try_from(&context)
    }
}

impl TryFrom<BmaModel> for BooleanNetwork {
    type Error = anyhow::Error;

    fn try_from(value: BmaModel) -> Result<Self, Self::Error> {
        BooleanNetwork::try_from(&value)
    }
}

impl TryFrom<&SymbolicContext> for BooleanNetwork {
    type Error = anyhow::Error;

    fn try_from(value: &SymbolicContext) -> Result<Self, Self::Error> {
        let rg = RegulatoryGraph::try_from(value)?;
        let mut bn = BooleanNetwork::new(rg);

        // Build update functions
        for (var, update) in &value.variables {
            if var.is_constant() {
                // Constant variables are handled separately, because they don't really have
                // a "normal" update function but a special constant function.
                assert_eq!(var.bdd_vars.len(), 1);
                assert_eq!(update.0.len(), 2);
                let bdd_var = var.bdd_vars[0];
                let level_zero_bdd = &update.0[0].1;
                let is_true = !level_zero_bdd.is_true();
                bn.set_update_function(cast_id(bdd_var), Some(FnUpdate::Const(is_true)))
                    .map_err(|e| anyhow!("Generated invalid update function: {e}"))?;

                continue;
            }
            // Go through all levels except for the lowest one (that's the default).
            for (i, level) in var.range().skip(1).enumerate() {
                let level_var = cast_id(var.bdd_vars[i]);
                let (output, level_update) = &update.0[i + 1];

                // Just make sure the iterators are not broken...
                assert_eq!(*output, level);

                // Turn the DNF into update function.
                let optimized_dnf = level_update.to_optimized_dnf();
                let mut aeon_clauses = Vec::new();
                for bdd_clause in optimized_dnf {
                    let mut aeon_clause = Vec::new();
                    for (bdd_var, value) in bdd_clause.to_values() {
                        let var_fn = FnUpdate::mk_var(cast_id(bdd_var));
                        if value {
                            aeon_clause.push(var_fn);
                        } else {
                            aeon_clause.push(FnUpdate::mk_not(var_fn));
                        }
                    }
                    aeon_clauses.push(FnUpdate::mk_conjunction(&aeon_clause));
                }
                let level_fn = FnUpdate::mk_disjunction(&aeon_clauses);

                // Now we have four options:
                //  - There is no lower/higher level.
                //  - There is a lower level, but not higher.
                //  - There is a higher level, but not lower.
                //  - There is both a lower and higher level.
                // This is based on: https://link.springer.com/chapter/10.1007/978-3-642-49321-8_15
                // (just steal it on scihub if you need to read it)

                let lower_var = i
                    .checked_sub(1)
                    .and_then(|x| var.bdd_vars.get(x))
                    .map(|var| cast_id(*var));

                let higher_var = i
                    .checked_add(1)
                    .and_then(|x| var.bdd_vars.get(x))
                    .map(|var| cast_id(*var));

                let level_fn = match (lower_var, higher_var) {
                    (None, None) => {
                        // This is a Boolean variable, no need to change the update function.
                        level_fn
                    }
                    (Some(lower), None) => {
                        // This is the highest level of a multivalued variable.
                        // It can only activate if lower level is active.
                        level_fn.and(FnUpdate::mk_var(lower))
                    }
                    (None, Some(higher)) => {
                        // This is the lowest level of a multivalued variable.
                        // It has to stay active if higher level is active.
                        level_fn.or(FnUpdate::mk_var(higher))
                    }
                    (Some(lower), Some(higher)) => {
                        // This is a middle level of a multivalued variable.
                        // It has to stay active if higher level is active, and can
                        // only activate if the lower level is active.
                        level_fn
                            .and(FnUpdate::mk_var(lower))
                            .or(FnUpdate::mk_var(higher))
                    }
                };

                bn.set_update_function(level_var, Some(level_fn))
                    .map_err(|e| anyhow!("Generated invalid update function: {e}"))?;
            }
        }

        bn.infer_valid_graph()
            .map_err(|e| anyhow!("Cannot normalize graph: {e}"))
    }
}

impl TryFrom<&SymbolicContext> for RegulatoryGraph {
    type Error = anyhow::Error;

    fn try_from(value: &SymbolicContext) -> Result<Self, Self::Error> {
        /// A helper method to make sure a regulation exists in a graph.
        fn ensure_regulation(
            rg: &mut RegulatoryGraph,
            edge: (BddVariable, BddVariable),
        ) -> Result<(), anyhow::Error> {
            let (regulator, target) = (cast_id(edge.0), cast_id(edge.1));
            if rg.find_regulation(regulator, target).is_none() {
                rg.add_raw_regulation(Regulation {
                    regulator,
                    target,
                    observable: true,
                    monotonicity: None,
                })
                .map_err(|e| anyhow!("{e}"))
            } else {
                Ok(())
            }
        }

        // Start by building the regulation graph.
        let variable_names = value.bdd_ctx.variable_names();
        let mut rg = RegulatoryGraph::new(variable_names);

        // Make regulations between variables that actually regulate each other.
        for (var, update) in &value.variables {
            // Invariant: The number of variables must be smaller than the number of
            // output levels by one.
            assert_eq!(var.bdd_vars.len() + 1, update.0.len());

            let non_minimal_levels = update.0.iter().skip(1);
            for (target_var, (_, bdd)) in var.bdd_vars.iter().zip(non_minimal_levels) {
                // Add a regulation from all variables that influence the update function.
                // At this point, we are checking the symbolic representation, not relationships
                // in BmaNetwork, mostly because not every level is going to be influenced by
                // every other level. This automatically removes unused relationships.
                // Sorting is just to make sure the iteration is deterministic.

                let mut support = Vec::from_iter(bdd.support_set());
                support.sort();
                for source_var in &support {
                    ensure_regulation(&mut rg, (*source_var, *target_var))?;
                }
            }

            // Also make regulations from x to x+1 and from x+1 to x.
            for (var_x, var_xx) in var.bdd_vars.iter().zip(var.bdd_vars.iter().skip(1)) {
                ensure_regulation(&mut rg, (*var_x, *var_xx))?;
                ensure_regulation(&mut rg, (*var_xx, *var_x))?;
            }
        }

        Ok(rg)
    }
}

impl TryFrom<&BmaModel> for SymbolicContext {
    type Error = anyhow::Error;

    fn try_from(model: &BmaModel) -> Result<Self, Self::Error> {
        // First, prepare the BDD context by declaring all symbolic variables.

        let mut builder = BddVariableSetBuilder::new();
        let mut variables = Vec::new();
        for var in &model.network.variables {
            let (min, max) = (var.min_level(), var.max_level());
            if min == max {
                // This is a constant. Constants are turned into Boolean "inputs" with a
                // constant update function. These will need some special handling later on.
                let name = var.mk_level_identifier(min);
                let bdd_var = builder.make_variable(name.as_str());
                variables.push(SymbolicVariable::new(var, vec![bdd_var]));
            } else {
                let mut bdd_variables = Vec::new();
                // For a variable with N values, we only build N-1 BDD variables,
                // because the lowest value is represented as all zeros.
                for level in (min + 1)..=max {
                    let name = var.mk_level_identifier(level);
                    let bdd_var = builder.make_variable(name.as_str());
                    bdd_variables.push(bdd_var);
                }
                variables.push(SymbolicVariable::new(var, bdd_variables));
            }
        }

        let bdd_ctx = builder.build();

        // Second, build all update functions.

        let mut variable_and_function = Vec::new();
        for var in &variables {
            let table = model.network.build_function_table(var.id)?;

            let symbolic_update = if var.is_constant() {
                // For constant variables, we don't build the update function normally.
                // We instead decide based on the constant value.

                assert_eq!(table.len(), 1); // Invariant: Constant update functions have one row.

                let const_level = var.range.0;
                let value = table[0].1;

                let (f, t) = (bdd_ctx.mk_false(), bdd_ctx.mk_true());
                if value == const_level {
                    SymbolicUpdateFunction(vec![(0, f), (const_level, t)])
                } else {
                    SymbolicUpdateFunction(vec![(0, t), (const_level, f)])
                }
            } else {
                SymbolicUpdateFunction::for_bma_function(&bdd_ctx, &variables, var.range, &table)?
            };

            variable_and_function.push((var.clone(), symbolic_update));
        }

        Ok(SymbolicContext {
            bdd_ctx,
            variables: variable_and_function,
        })
    }
}

impl SymbolicVariable {
    /// Fix BDD variables in the given [`BddPartialValuation`] such that they represent exactly
    /// all valuations that map to the given `level` in the symbolic variable encoding (or those
    /// that are technically invalid, but most related to that level).
    ///
    /// Precondition: `level` must be valid for this variable.
    pub fn write_symbolic_level(&self, valuation: &mut BddPartialValuation, level: u32) {
        // Right now, these are not part of the public API, so we can be a bit stricter
        // about error handling. If we every make this available to users, the method
        // must throw an error instead.
        assert!(level >= self.range.0);
        assert!(level <= self.range.1);

        // We need to skip one, because the first value is the "default" (all BDD vars are false).
        for (i, l) in self.range().skip(1).enumerate() {
            if level > l {
                // These variables can be true/false, we don't care. We only start setting
                // variables once l == level.
                continue;
            }
            let value = level >= l;
            valuation.set_value(self.bdd_vars[i], value);
        }
    }

    /// A range of all variable levels.
    pub fn range(&self) -> RangeInclusive<u32> {
        self.range.0..=self.range.1
    }

    /// True if the variable represents a constant.
    pub fn is_constant(&self) -> bool {
        self.range.0 == self.range.1
    }
}

impl SymbolicUpdateFunction {
    /// Build a symbolic update function representation based on:
    ///  - A prepared BDD context.
    ///  - List of system variables describing the encoding.
    ///  - Expected variable range (in case some output levels do not appear
    ///    in the function explicitly)
    ///  - The actual function table.
    pub fn for_bma_function(
        bdd_ctx: &BddVariableSet,
        variables: &[SymbolicVariable],
        range: (u32, u32),
        function: &FunctionTable,
    ) -> anyhow::Result<SymbolicUpdateFunction> {
        let (min_level, max_level) = range;

        // Build a DNF representation for each level based on each input-output pair.

        let mut level_dnf_vec = Vec::new();
        for level in min_level..=max_level {
            level_dnf_vec.push((level, Vec::new()));
        }

        // Just a map to quickly resolve symbolic variables based on IDs.
        let var_id_map = variables
            .iter()
            .map(|it| (it.id, it))
            .collect::<HashMap<_, _>>();

        for (input, output) in function {
            // If this is violated, there is something very wrong with the function table.
            if *output < min_level || *output > max_level {
                return Err(anyhow!(
                    "Output level {output} outside of expected range [{min_level}..={max_level}]"
                ));
            }

            // Write all input variable values into the valuation.
            let mut input_valuation = BddPartialValuation::empty();
            for (var_id, var_level) in input {
                let Some(var_ref) = var_id_map.get(var_id) else {
                    return Err(anyhow!("Function table uses unknown variable `{var_id}`"));
                };
                var_ref.write_symbolic_level(&mut input_valuation, *var_level);
            }

            let output_index =
                usize::try_from(*output - min_level).expect("16-bit devices are not supported.");

            // Add the input valuation to the respective output BDD.
            level_dnf_vec[output_index].1.push(input_valuation);
        }

        let level_bdd_vec = level_dnf_vec
            .into_iter()
            .map(|(level, dnf)| (level, bdd_ctx.mk_dnf(&dnf)))
            .collect::<Vec<_>>();

        // At this point, the function should cover all valuations.
        let all_valuations = level_bdd_vec
            .iter()
            .fold(bdd_ctx.mk_false(), |a, (_, b)| a.or(b));

        if !all_valuations.is_true() {
            return Err(anyhow!(
                "Function table does not cover every possible input."
            ));
        }

        // And also all levels should be pair-wise disjoint.
        for (l_a, bdd_a) in &level_bdd_vec {
            for (l_b, bdd_b) in &level_bdd_vec {
                if l_a == l_b {
                    continue;
                }

                if !bdd_a.and(bdd_b).is_false() {
                    return Err(anyhow!(
                        "Function levels {l_a} and {l_b} are not exclusive."
                    ));
                }
            }
        }

        Ok(SymbolicUpdateFunction(level_bdd_vec))
    }
}

#[cfg(test)]
mod tests {
    use crate::BmaModel;
    use anyhow::anyhow;
    use biodivine_lib_param_bn::BooleanNetwork;
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
    use std::cmp::max;

    #[test]
    fn basic_binarization_test() {
        let folders = [
            "./models/json-repo",
            "./models/json-export-from-repo",
            "./models/json-export-from-tool",
        ];

        // These three models have a constant node with weird/invalid update functions.
        let unsafe_models = vec![
            "./models/json-export-from-repo/Skin2D_5X2_TF.json",
            "./models/json-export-from-repo/Skin2D_5X2.json",
            "./models/json-export-from-tool/Leukaemia.json",
        ];

        for folder in &folders {
            for file in std::fs::read_dir(folder).unwrap() {
                let file = file.unwrap();
                let file_name = file.file_name().to_str().unwrap().to_owned();
                if !file_name.ends_with(".json") {
                    continue;
                }

                let path = format!("{}/{}", folder, file_name);
                if unsafe_models.contains(&path.as_str()) {
                    println!("Skipping {}", path);
                    continue;
                }

                println!("File: {}/{}", folder, file_name);

                // Even though remaining JSON models have some validation issues, they should
                // not affect the Boolean conversion.
                let json_data = std::fs::read_to_string(file.path()).unwrap();
                let model = BmaModel::from_json_string(json_data.as_str()).unwrap();
                let network = BooleanNetwork::try_from(&model).unwrap();

                // We can easily test the variable count.
                let mut expected_var_count = 0;
                for var in &model.network.variables {
                    expected_var_count += max(1, var.max_level() - var.min_level());
                }
                assert_eq!(network.num_vars(), expected_var_count as usize);

                // And we can also test that the symbolic graph is "valid".
                assert!(SymbolicAsyncGraph::new(&network).is_ok());

                assert_eq!(network.num_implicit_parameters(), 0);
                assert_eq!(network.num_parameters(), 0);
            }
        }
    }

    /// Wrapper to get a simple BMA model for testing.
    ///
    /// The model has:
    /// - two variables `(a=1, b=2)`
    /// - two relationships `(a -| b, b -> a)`
    /// - the following update functions: `(a: var(2), b: 1-var(a))`
    ///
    /// There is no layout or additional information in the model.
    fn get_simple_test_model() -> BmaModel {
        let model_str = r#"<?xml version="1.0" encoding="utf-8"?>
    <AnalysisInput ModelName="New Model">
        <Variables>
            <Variable Id="1">
                <Name>a</Name>
                <RangeFrom>0</RangeFrom>
                <RangeTo>1</RangeTo>
                <Function>var(2)</Function>
            </Variable>
            <Variable Id="2">
                <Name>b</Name>
                <RangeFrom>0</RangeFrom>
                <RangeTo>1</RangeTo>
                <Function>1-var(1)</Function>
            </Variable>
        </Variables>
        <Relationships>
            <Relationship Id="1">
                <FromVariableId>1</FromVariableId>
                <ToVariableId>2</ToVariableId>
                <Type>Inhibitor</Type>
            </Relationship>
            <Relationship Id="2">
                <FromVariableId>2</FromVariableId>
                <ToVariableId>1</ToVariableId>
                <Type>Activator</Type>
            </Relationship>
        </Relationships>
    </AnalysisInput>"#;
        BmaModel::from_xml_string(model_str).expect("XML was not well-formatted")
    }

    /// Wrapper to get a little bit more complex BMA model for testing.
    ///
    /// The model has:
    /// - three variables `(a=1, b=2, c=3)`
    /// - five relationships `(a -| b, b -> a, a -> c, b -> c, c -> c)`
    /// - the following update functions: `(a: var(2), b: 1-var(a), c: var(1) * var(2) * var(3))`
    fn get_test_model() -> BmaModel {
        let model_str = r#"<?xml version="1.0" encoding="utf-8"?>
    <AnalysisInput ModelName="New Model">
        <Variables>
            <Variable Id="1">
                <Name>a</Name>
                <RangeFrom>0</RangeFrom>
                <RangeTo>1</RangeTo>
                <Function>var(2)</Function>
            </Variable>
            <Variable Id="2">
                <Name>b</Name>
                <RangeFrom>0</RangeFrom>
                <RangeTo>1</RangeTo>
                <Function>1-var(1)</Function>
            </Variable>
            <Variable Id="3">
                <Name>c</Name>
                <RangeFrom>0</RangeFrom>
                <RangeTo>1</RangeTo>
                <Function>var(1) * var(2) * var(3)</Function>
            </Variable>
        </Variables>
        <Relationships>
            <Relationship Id="1">
                <FromVariableId>1</FromVariableId>
                <ToVariableId>2</ToVariableId>
                <Type>Inhibitor</Type>
            </Relationship>
            <Relationship Id="2">
                <FromVariableId>2</FromVariableId>
                <ToVariableId>1</ToVariableId>
                <Type>Activator</Type>
            </Relationship>
            <Relationship Id="3">
                <FromVariableId>1</FromVariableId>
                <ToVariableId>3</ToVariableId>
                <Type>Activator</Type>
            </Relationship>
            <Relationship Id="4">
                <FromVariableId>2</FromVariableId>
                <ToVariableId>3</ToVariableId>
                <Type>Activator</Type>
            </Relationship>
            <Relationship Id="5">
                <FromVariableId>3</FromVariableId>
                <ToVariableId>3</ToVariableId>
                <Type>Activator</Type>
            </Relationship>
        </Relationships>
    </AnalysisInput>"#;
        BmaModel::from_xml_string(model_str).expect("XML was not well-formatted")
    }

    #[test]
    fn test_to_bn_simple() {
        let bma_model = get_simple_test_model();
        let result_bn = BooleanNetwork::try_from(&bma_model)
            .and_then(|it| it.infer_valid_graph().map_err(|e| anyhow!(e)));

        let bn_str = r#"
        v_1_a_b1 -| v_2_b_b1
        v_2_b_b1 -> v_1_a_b1
        $v_1_a_b1: v_2_b_b1
        $v_2_b_b1: !v_1_a_b1
    "#;
        let expected_bn = BooleanNetwork::try_from(bn_str).unwrap();

        assert!(result_bn.is_ok());
        assert_eq!(result_bn.unwrap(), expected_bn);
    }

    #[test]
    fn test_to_bn() {
        let bma_model = get_test_model();
        let result_bn = BooleanNetwork::try_from(&bma_model)
            .and_then(|it| it.infer_valid_graph().map_err(|e| anyhow!(e)));

        let bn_str = r#"
        v_1_a_b1 -| v_2_b_b1
        v_1_a_b1 -> v_3_c_b1
        v_2_b_b1 -> v_1_a_b1
        v_2_b_b1 -> v_3_c_b1
        v_3_c_b1 -> v_3_c_b1
        $v_1_a_b1: v_2_b_b1
        $v_2_b_b1: !v_1_a_b1
        $v_3_c_b1: (v_1_a_b1 & v_2_b_b1 & v_3_c_b1)
    "#;
        let expected_bn = BooleanNetwork::try_from(bn_str).unwrap();
        assert_eq!(result_bn.unwrap(), expected_bn);
    }
}
