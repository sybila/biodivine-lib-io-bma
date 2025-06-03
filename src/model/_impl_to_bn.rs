use crate::update_fn::bma_fn_update::BmaFnUpdate;
use crate::update_fn::expression_enums::{AggregateFn, ArithOp};
use crate::{BmaModel, BmaVariable};
use biodivine_lib_param_bn::{BooleanNetwork, RegulatoryGraph};
use regex::Regex;
use std::collections::{BTreeMap, HashMap};

impl BmaModel {
    /// Generate a canonical name for a BMA variable by combining its ID and name.
    /// This canonical name will be used in a BooleanNetwork.
    fn canonical_var_name(var: &BmaVariable) -> String {
        if let Some(name) = var.name.as_ref() {
            // Regex that matches non-alphanumeric and non-underscore characters
            let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
            let sanitized_name = re.replace_all(name, "");
            format!("v_{}_{}", var.id, sanitized_name)
        } else {
            format!("v_{}", var.id)
        }
    }

    /// Extract a regulatory graph from this BMA model.
    ///
    /// Returns a `RegulatoryGraph` instance (extracting variables and regulations from
    /// this model) and a mapping of BMA variable IDs to their canonical names used in
    /// the new graph.
    ///
    /// See [Self::canonical_var_name] for how the variable names are derived. Variables are
    /// sorted by these canonical names (which means they are basically sorted by their BMA IDs).
    /// Regulations are sorted by their regulator (first key) and their target (second key).
    ///
    /// It is possible that the BMA model has more than one regulation between the same pair
    /// of variables. If they have the same type, we simply add it once. If they have different
    /// signs, we add a regulation with unspecified monotonicity.
    /// Moreover, all regulations are made observable by default.
    pub fn to_regulatory_graph(&self) -> Result<(RegulatoryGraph, HashMap<u32, String>), String> {
        // Sort variables by their IDs before inserting them into the graph to ensure deterministic
        // ordering. We use a BTreeMap to ensure variables remain sorted.
        let mut variables_map: BTreeMap<u32, String> = BTreeMap::new();
        let mut variables_sorted = self.network.variables.clone();
        variables_sorted.sort_by_key(|var| (var.id, var.name.clone()));

        for var in &variables_sorted {
            let inserted = variables_map.insert(var.id, BmaModel::canonical_var_name(var));
            if inserted.is_some() {
                return Err(format!("Variable ID {} is not unique.", var.id));
            }
        }
        let variables = variables_map.clone().into_values().collect();
        let mut graph = RegulatoryGraph::new(variables);

        // add regulations (in the order of variables, first by regulator, then target)
        let mut relationships_sorted = self.network.relationships.clone();
        relationships_sorted.sort_by_key(|rel| (rel.from_variable, rel.to_variable));
        for bma_relationship in relationships_sorted {
            let regulator_bma_id = bma_relationship.from_variable;
            let target_bma_id = bma_relationship.to_variable;
            let regulator = variables_map.get(&regulator_bma_id).ok_or(format!(
                "Regulator var {} does not exist.",
                regulator_bma_id
            ))?;
            let target = variables_map
                .get(&target_bma_id)
                .ok_or(format!("Target var {} does not exist.", target_bma_id))?;
            let monotonicity = Some(bma_relationship.r#type.into());

            // check for doubled regulations (BMA allows multiple regulations between the same vars)
            let regulator_aeon_id = graph.find_variable(regulator).unwrap(); // safe to unwrap
            let target_aeon_id = graph.find_variable(target).unwrap(); // safe to unwrap
            if let Some(existing_reg) = graph.find_regulation(regulator_aeon_id, target_aeon_id) {
                // if the two regulations have different signs, add non-monotonic instead
                // otherwise do nothing
                if existing_reg.monotonicity != monotonicity {
                    graph.remove_regulation(regulator_aeon_id, target_aeon_id)?;
                    graph.add_regulation(regulator, target, true, None)?;
                }
            } else {
                graph.add_regulation(regulator, target, true, monotonicity)?;
            }
        }

        // return variables_map as well, but as a standard HashMap
        let variables_map = variables_map.into_iter().collect::<HashMap<_, _>>();
        Ok((graph, variables_map))
    }

    /// Create a default update function for a variable in the BMA model with
    ///  an originally empty formula.
    ///
    /// This function is created the same way as BMA does it, even though that
    /// can feel weird at times.
    pub fn create_default_update_fn(&self, var_id: u32) -> BmaFnUpdate {
        let (positive, negative) = self.get_regulators(var_id);
        if positive.is_empty() && negative.is_empty() {
            // This is an undetermined input, in which case we set it to zero,
            // because that's what BMA does.
            return BmaFnUpdate::mk_constant(0);
        }

        // We build the default function the same way as BMA does.

        // First, we average the positive regulators
        let p_avr = if !positive.is_empty() {
            let p_args = positive
                .iter()
                .map(|&x| BmaFnUpdate::mk_variable(x))
                .collect();
            BmaFnUpdate::mk_aggregation(AggregateFn::Avg, p_args)
        } else {
            // This makes little sense because it means any variable with only negative
            // regulators is ALWAYS a constant zero. But this is how BMA seems to be doing it, so
            // that's what we are doing as well...
            BmaFnUpdate::mk_constant(0)
        };

        // Now we average the negative regulators
        let n_avr = if !negative.is_empty() {
            let n_args = negative
                .iter()
                .map(|&x| BmaFnUpdate::mk_variable(x))
                .collect();
            BmaFnUpdate::mk_aggregation(AggregateFn::Avg, n_args)
        } else {
            BmaFnUpdate::mk_constant(0)
        };

        // Finally, we subtract the negative average from the positive average
        BmaFnUpdate::mk_arithmetic(p_avr, n_avr, ArithOp::Minus)
    }

    /// Convert BmaModel into a BooleanNetwork instance. At the moment, this only supports
    /// pure Boolean models (not multi-valued that would need additional conversion).
    ///
    /// The network will contain the same set of variables and regulations as this model.
    /// See [Self::to_regulatory_graph] for details on how the regulation graph is extracted,
    /// and [Self::canonical_var_name] for how the variable names are derived.
    /// The update functions are transformed using [BmaFnUpdate::to_update_fn_boolean].
    ///
    /// By default, all regulations are considered as observable, and their sign is taken from the
    /// BMA model as is. This may be inconsistent with the update functions, which may or may not be
    /// intended. If `repair_graph` is set to true, the regulation properties (observability and
    /// monotonicity) are instead inferred from the update function BDDs directly.
    ///
    /// TODO: For now, we do not handle multi-valued models. However, some internal
    /// methods are made general to deal with multi-valued networks in future.
    pub fn to_boolean_network(&self, repair_graph: bool) -> Result<BooleanNetwork, String> {
        if !self.is_boolean_model() {
            return Err(
                "Currently, converting multi-valued models into BNs is not supported.".to_string(),
            );
        }

        // Extract the regulatory graph and variable name mapping (BMA var id -> BN var name)
        let (graph, var_name_mapping) = self.to_regulatory_graph()?;
        let mut bn = BooleanNetwork::new(graph);

        // collect max levels of variables
        let mut max_levels = HashMap::new();
        // for boolean models, this should be `1` except for zero constants
        // we deal with zero constants making into boolean variables with constant update
        let mut zero_constants = Vec::new();
        for bma_var in &self.network.variables {
            if bma_var.max_level() == 0 {
                zero_constants.push(bma_var.id); // remember to deal with these specially
                max_levels.insert(bma_var.id, 1); // standard boolean variable now
            } else {
                max_levels.insert(bma_var.id, bma_var.max_level());
            }
        }

        // add update functions
        for bma_var in &self.network.variables {
            // unwrap is safe in both cases here
            let bn_var_name = var_name_mapping.get(&bma_var.id).unwrap();
            let var_max_lvl = max_levels.get(&bma_var.id).unwrap();

            if zero_constants.contains(&bma_var.id) {
                // We can have zero constants, and we must deal with these accordingly.
                // BMA sets the update function to zero in this case regardless of the formula.
                bn.add_string_update_function(bn_var_name, "false").unwrap();
                continue;
            }

            if let Some(bma_formula) = bma_var.formula.clone() {
                // We have a formula, so we need to convert it to a proper update function.
                // todo: to_update_fn is not fully finished yet
                let update_fn_formula = bma_formula.to_update_fn_boolean(
                    &max_levels,
                    &var_name_mapping,
                    *var_max_lvl,
                )?;
                bn.add_string_update_function(bn_var_name, &update_fn_formula)?;
            } else {
                // The formula is empty, which means we have to build a default one
                // the same way as BMA is doing this.
                let default_bma_formula = self.create_default_update_fn(bma_var.id);
                // Convert this default BMA expression to a logical update fn.
                let update_fn_formula = default_bma_formula.to_update_fn_boolean(
                    &max_levels,
                    &var_name_mapping,
                    *var_max_lvl,
                )?;
                bn.add_string_update_function(bn_var_name, &update_fn_formula)?;
            }
        }

        if repair_graph {
            bn.infer_valid_graph()
        } else {
            Ok(bn)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BmaModel;
    use biodivine_lib_param_bn::BooleanNetwork;
    use biodivine_lib_param_bn::RegulatoryGraph;

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
        BmaModel::from_xml_str(model_str).expect("XML was not well-formatted")
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
        BmaModel::from_xml_str(model_str).expect("XML was not well-formatted")
    }

    #[test]
    fn test_to_reg_graph_simple() {
        let bma_model = get_simple_test_model();
        let (result_graph, _) = bma_model.to_regulatory_graph().unwrap();

        let expected_regulations = vec!["v_1_a -| v_2_b".to_string(), "v_2_b -> v_1_a".to_string()];
        let expected_graph =
            RegulatoryGraph::try_from_string_regulations(expected_regulations).unwrap();

        assert_eq!(result_graph, expected_graph);
    }

    #[test]
    fn test_to_reg_graph() {
        let bma_model = get_test_model();
        let (result_graph, _) = bma_model.to_regulatory_graph().unwrap();

        let expected_regulations = vec![
            "v_1_a -| v_2_b".to_string(),
            "v_1_a -> v_3_c".to_string(),
            "v_2_b -> v_1_a".to_string(),
            "v_2_b -> v_3_c".to_string(),
            "v_3_c -> v_3_c".to_string(),
        ];
        let expected_graph =
            RegulatoryGraph::try_from_string_regulations(expected_regulations).unwrap();

        assert_eq!(result_graph, expected_graph);
    }

    #[test]
    fn test_to_bn_simple() {
        let bma_model = get_simple_test_model();
        let result_bn = bma_model.to_boolean_network(true);

        let bn_str = r#"
            v_1_a -| v_2_b
            v_2_b -> v_1_a
            $v_1_a: v_2_b
            $v_2_b: !v_1_a
        "#;
        let expected_bn = BooleanNetwork::try_from(bn_str).unwrap();

        assert!(result_bn.is_ok());
        assert_eq!(result_bn.unwrap(), expected_bn);
    }

    #[test]
    fn test_to_bn() {
        let bma_model = get_test_model();
        let result_bn = bma_model.to_boolean_network(true);

        let bn_str = r#"
            v_1_a -| v_2_b
            v_1_a -> v_3_c
            v_2_b -> v_1_a
            v_2_b -> v_3_c
            v_3_c -> v_3_c
            $v_1_a: v_2_b
            $v_2_b: !v_1_a
            $v_3_c: (v_1_a & v_2_b & v_3_c)
        "#;
        let expected_bn = BooleanNetwork::try_from(bn_str).unwrap();

        assert!(result_bn.is_ok());
        assert_eq!(result_bn.unwrap(), expected_bn);
    }
}
