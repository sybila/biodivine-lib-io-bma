use crate::update_function::{AggregateFn, ArithOp, BmaUpdateFunction, InvalidBmaUpdateFunction};
use crate::{
    BmaModel, BmaModelError, BmaNetworkError, BmaRelationshipError, BmaVariable, BmaVariableError,
    RelationshipType, Validation,
};
use BmaRelationshipError::{RegulatorVariableNotFound, TargetVariableNotFound};
use anyhow::anyhow;
use biodivine_lib_param_bn::{
    BooleanNetwork, FnUpdate, Monotonicity, Regulation, RegulatoryGraph, VariableId,
};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::mem::swap;

/// Convert [`BmaModel`] into a [`BooleanNetwork`] instance. At the moment, this only supports
/// pure Boolean models (not multivalued that would need additional conversion).
///
/// By default, all regulations are considered as observable, and their sign is taken from the
/// BMA model as is. This may be inconsistent with the update functions, which may or may not be
/// intended. You can use [`BooleanNetwork::infer_valid_graph`] to fix this after the conversion.
///
/// TODO: For now, we do not handle multi-valued models. However, some internal
/// methods are made general to deal with multi-valued networks in future.
impl TryFrom<&BmaModel> for BooleanNetwork {
    type Error = anyhow::Error;

    fn try_from(model: &BmaModel) -> Result<Self, Self::Error> {
        // First, make sure all missing functions are now included in the model.
        let mut model = model.clone();
        model.populate_missing_functions();

        if !model.is_boolean() {
            return Err(anyhow!(
                "Multi-valued model cannot be converted into Boolean network"
            ));
        }

        let graph = RegulatoryGraph::try_from(&model)?;
        let mut bn = BooleanNetwork::new(graph);

        let bma_id_to_aeon_id = build_variable_id_map(&model);

        // Errors that prevent the model from being converted:
        //  - Anything that breaks the regulatory graph conversion (already resolved above).
        //  - Any variable with invalid update function.
        //  - Any variable with invalid range.
        if let Err(errors) = model.validate() {
            for e in errors {
                match e {
                    BmaModelError::Network(network_error) => match network_error {
                        BmaNetworkError::Variable(var_error) => {
                            if matches!(var_error, BmaVariableError::RangeInvalid { .. }) {
                                return Err(var_error.into());
                            }
                            if matches!(
                                var_error,
                                BmaVariableError::UpdateFunctionExpressionInvalid { .. }
                            ) {
                                return Err(var_error.into());
                            }
                        }
                        BmaNetworkError::Relationship(_) => (),
                    },
                    BmaModelError::Layout(_) => {}
                }
            }
        }

        // Build update functions:
        for bma_var in &model.network.variables {
            // Unwrap is safe because regulatory graph was constructed successfully.
            let aeon_var = *bma_id_to_aeon_id.get(&bma_var.id).unwrap();

            if bma_var.max_level() == 0 {
                // We can have zero constants, and we must deal with these accordingly.
                // BMA sets the update function to zero in this case regardless of the formula.
                // Setting a constant update function should never fail, hence unwrap is safe.
                bn.set_update_function(aeon_var, Some(FnUpdate::Const(false)))
                    .unwrap();
                continue;
            }

            let aeon_formula = model.export_function_to_aeon(bma_var, &bma_id_to_aeon_id)?;

            // Note that this operation can fail if some regulations in the BMA model are
            // not set up properly. Unless we "infer" the network structure from update functions,
            // we can't really prevent this.
            bn.set_update_function(aeon_var, Some(aeon_formula))
                .map_err(|e| anyhow!(e))?;
        }

        Ok(bn)
    }
}

/// Extract a regulatory graph from this BMA model.
///
/// Returns a [`RegulatoryGraph`] instance (extracting variables and regulations from
/// this model) and a mapping of BMA variable IDs to their canonical names used in
/// the new graph.
///
/// It is possible that the BMA model has more than one regulation between the same pair
/// of variables. If they have the same type, we simply add it once. If they have different
/// signs, we add a regulation with unspecified monotonicity.
/// Moreover, all regulations are made observable by default.
impl TryFrom<&BmaModel> for RegulatoryGraph {
    type Error = anyhow::Error;

    fn try_from(model: &BmaModel) -> Result<Self, anyhow::Error> {
        // Errors that prevent the model from being converted:
        //  - Variables with duplicate ID (can cause canonical names to clash).
        //  - Relationships using unknown variable IDs.
        if let Err(errors) = model.validate() {
            for e in errors {
                match e {
                    BmaModelError::Network(network_error) => match network_error {
                        BmaNetworkError::Variable(var_error) => {
                            if matches!(var_error, BmaVariableError::IdNotUnique { .. }) {
                                return Err(var_error.into());
                            }
                        }
                        BmaNetworkError::Relationship(rel_error) => {
                            if matches!(rel_error, TargetVariableNotFound { .. }) {
                                return Err(rel_error.into());
                            }
                            if matches!(rel_error, RegulatorVariableNotFound { .. }) {
                                return Err(rel_error.into());
                            }
                        }
                    },
                    BmaModelError::Layout(_) => {}
                }
            }
        }

        let variable_names = model
            .network
            .variables
            .iter()
            .map(canonical_var_name)
            .collect::<Vec<_>>();

        let bma_id_to_aeon_id = build_variable_id_map(model);

        // This must be successful, because variable names are unique (because they use
        // unique IDs).
        let mut regulatory_graph = RegulatoryGraph::new(variable_names);

        for relationship in &model.network.relationships {
            // These unwrap operations must succeed, because regulations
            // only use valid variable IDs.
            let source = bma_id_to_aeon_id
                .get(&relationship.from_variable)
                .expect("Invariant violation: Relationship source variable not found.");
            let target = bma_id_to_aeon_id
                .get(&relationship.to_variable)
                .expect("Invariant violation: Relationship target variable not found.");
            let mut regulation = Regulation {
                regulator: *source,
                target: *target,
                observable: true,
                monotonicity: Monotonicity::try_from(relationship.r#type.clone()).ok(),
            };
            // If no invariants are broken, these operations should not panic.
            if let Some(existing) = regulatory_graph.find_regulation(*source, *target) {
                if existing.monotonicity != regulation.monotonicity {
                    regulation.monotonicity = None;
                    regulatory_graph
                        .remove_regulation(*source, *target)
                        .expect("Invariant violated: Regulation already missing.");
                    regulatory_graph
                        .add_raw_regulation(regulation)
                        .expect("Invariant violated: Regulation already exists.");
                }
            } else {
                regulatory_graph
                    .add_raw_regulation(regulation)
                    .expect("Invariant violated: Regulation already exists.");
            }
        }

        Ok(regulatory_graph)
    }
}

/// Generate a canonical name for a BMA variable by combining its ID and name.
/// This canonical name will be used in a `BooleanNetwork`.
fn canonical_var_name(var: &BmaVariable) -> String {
    // Regex that matches non-alphanumeric and non-underscore characters
    let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
    let sanitized_name = re.replace_all(var.name.as_str(), "");
    format!("v_{}_{}", var.id, sanitized_name)
}

/// Build a map which assigns each BMA variable ID an AEON variable ID.
fn build_variable_id_map(model: &BmaModel) -> HashMap<u32, VariableId> {
    model
        .network
        .variables
        .iter()
        .enumerate()
        .map(|(i, v)| (v.id, VariableId::from_index(i)))
        .collect::<HashMap<u32, VariableId>>()
}

#[cfg(test)]
mod tests {
    use crate::BmaModel;
    use anyhow::anyhow;
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
    fn test_to_reg_graph_simple() {
        let bma_model = get_simple_test_model();
        let result_graph = RegulatoryGraph::try_from(&bma_model).unwrap();

        let expected_regulations = vec!["v_1_a -| v_2_b".to_string(), "v_2_b -> v_1_a".to_string()];
        let expected_graph =
            RegulatoryGraph::try_from_string_regulations(expected_regulations).unwrap();

        assert_eq!(result_graph, expected_graph);
    }

    #[test]
    fn test_to_reg_graph() {
        let bma_model = get_test_model();
        let result_graph = RegulatoryGraph::try_from(&bma_model).unwrap();

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
        let result_bn = BooleanNetwork::try_from(&bma_model)
            .and_then(|it| it.infer_valid_graph().map_err(|e| anyhow!(e)));

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
        let result_bn = BooleanNetwork::try_from(&bma_model)
            .and_then(|it| it.infer_valid_graph().map_err(|e| anyhow!(e)));

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
