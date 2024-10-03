use crate::enums::{RelationshipType, VariableType};
use crate::model::bma_model::*;
use crate::update_fn::bma_fn_tree::BmaFnUpdate;
use crate::update_fn::enums::{AggregateFn, ArithOp};

use biodivine_lib_param_bn::{BooleanNetwork, Monotonicity, RegulatoryGraph};

use regex::Regex;
use std::cmp::max;
use std::collections::HashMap;

impl BmaModel {
    /// Utility to generate a canonical name for a BMA `Variable` by combining its ID and name.
    /// This canonical name will be used in a BooleanNetwork.
    fn canonical_var_name(var: &Variable) -> String {
        // Regex that matches non-alphanumeric and non-underscore characters
        let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
        let sanitized_name = re.replace_all(&var.name, "");
        format!("v_{}_{}", var.id, sanitized_name)
    }

    /// Convert BmaModel into a RegulatoryGraph instance.
    /// It will contain the same variables (see [Self::canonical_var_name] for variable names) and
    /// regulations.
    pub fn to_regulatory_graph(&self) -> Result<RegulatoryGraph, String> {
        let mut variables_map: HashMap<u32, String> = HashMap::new();
        for var in &self.model.variables {
            let inserted = variables_map.insert(var.id, BmaModel::canonical_var_name(var));
            if inserted.is_some() {
                return Err(format!("Variable ID {} is not unique.", var.id));
            }
        }
        let variables = variables_map.clone().into_values().collect();
        let mut graph = RegulatoryGraph::new(variables);

        // add regulations
        // TODO: decide how to handle "doubled" regulations (of the same vs of different type)
        self.model
            .relationships
            .iter()
            .try_for_each(|relationship| {
                let regulator_id = relationship.from_variable;
                let target_id = relationship.to_variable;
                let regulator = variables_map
                    .get(&regulator_id)
                    .ok_or(format!("Regulator var {} does not exist.", regulator_id))?;
                let target = variables_map
                    .get(&target_id)
                    .ok_or(format!("Target var {} does not exist.", target_id))?;
                let monotonicity = Some(relationship.relationship_type.into());
                graph.add_regulation(regulator, target, true, monotonicity)
            })?;

        Ok(graph)
    }

    /// Convert BmaModel into a BooleanNetwork instance.
    /// It will contain the same variables (see [Self::canonical_var_name] for variable names) and
    /// regulations. The update functions are transformed using [BmaFnUpdate::to_update_fn].
    pub fn to_boolean_network(&self) -> Result<BooleanNetwork, String> {
        // TODO: for now, we do not handle multi-valued models

        if !self.is_boolean_model() {
            return Err("Cannot convert multi-valued model to a Boolean network.".to_string());
        }

        let graph = self.to_regulatory_graph()?;
        let mut bn = BooleanNetwork::new(graph);

        // TODO: this will have to change to handle multi-valued models
        let mut max_levels = HashMap::new();
        for var in &self.model.variables {
            let var_name = BmaModel::canonical_var_name(var);
            max_levels.insert(var_name, var.range_to);
        }

        // add update functions
        for var in &self.model.variables {
            let var_name = BmaModel::canonical_var_name(var);
            let var_id = bn.as_graph().find_variable(&var_name).unwrap();

            if var.range_to == 0 {
                // We can have zero constants and we must deal with these accordingly.
                bn.add_string_update_function(&var_name, "false").unwrap()
            }

            if let Some(bma_formula) = var.formula.clone() {
                let update_fn = bma_formula.to_update_fn(&max_levels);
                bn.set_update_function(var_id, Some(update_fn))?;
            } else {
                // The formula is empty, which means we have to build a default one
                // the same way as BMA is doing this.
                // We then convert this default BMA expression to a logical formula.

                let regulators = bn.regulators(var_id);
                if regulators.is_empty() {
                    // This is an undetermined input, in which case we set it to zero,
                    // because that's what BMA does.
                    bn.add_string_update_function(&var_name, "false").unwrap()
                }

                // We build the default function the same way as BMA does.
                let mut positive = Vec::new();
                let mut negative = Vec::new();
                for regulator in regulators {
                    let regulator_name = bn.get_variable_name(regulator);
                    let reg = bn.as_graph().find_regulation(regulator, var_id).unwrap();
                    // BMA variables must be monotonic
                    match reg.monotonicity.unwrap() {
                        Monotonicity::Activation => positive.push(regulator_name),
                        Monotonicity::Inhibition => negative.push(regulator_name),
                    }
                }

                let p_avr = if !positive.is_empty() {
                    let p_args = positive
                        .iter()
                        .map(|x| BmaFnUpdate::mk_variable(x))
                        .collect();
                    BmaFnUpdate::mk_aggregation(AggregateFn::Avg, p_args)
                } else {
                    // This does not make much sense, because it means any variable with only negative
                    // regulators is ALWAYS a constant zero. But this is how BMA seems to be doing it, so
                    // that's what we are doing as well...
                    BmaFnUpdate::mk_constant(0)
                };
                let n_avr = if !negative.is_empty() {
                    let n_args = negative
                        .iter()
                        .map(|x| BmaFnUpdate::mk_variable(x))
                        .collect();
                    BmaFnUpdate::mk_aggregation(AggregateFn::Avg, n_args)
                } else {
                    BmaFnUpdate::mk_constant(0)
                };
                let default_bma_formula = BmaFnUpdate::mk_arithmetic(p_avr, n_avr, ArithOp::Minus);
                let update_fn = default_bma_formula.to_update_fn(&max_levels);
                bn.set_update_function(var_id, Some(update_fn))?;
            }
        }

        Ok(bn)
    }

    /// Construct `BmaModel` instance with a given name from a provided BooleanNetwork `bn`.
    ///
    /// The Boolean network must contain parameters in any of its update functions.
    ///
    /// TODO: for now, we only utilize monotonic regulations (and ignore the rest), and we do not use observability
    pub fn from_boolean_network(bn: &BooleanNetwork, name: &str) -> Result<BmaModel, String> {
        if bn.num_parameters() > 0 {
            return Err("Boolean network with parameters can not be translated.".to_string());
        }
        // transform variables and update functions
        let variables = bn
            .variables()
            .map(|var_id| {
                let formula = if let Some(update_fn) = bn.get_update_function(var_id) {
                    // we unwrap since we already checked BN has no parameters
                    let bma_function = BmaFnUpdate::try_from_fn_update(update_fn).unwrap();
                    Some(bma_function)
                } else {
                    None
                };
                Variable {
                    id: var_id.to_index() as u32,
                    name: bn.get_variable_name(var_id).clone(),
                    range_from: 0,
                    range_to: 1,
                    formula,
                }
            })
            .collect();

        // transform regulations into relationships
        // TODO: deal with non-monotonic regulations
        let relationships = bn
            .as_graph()
            .regulations()
            .filter(|reg| reg.monotonicity.is_some())
            .enumerate()
            .map(|(idx, reg)| Relationship {
                id: idx as u32,
                from_variable: reg.regulator.to_index() as u32,
                to_variable: reg.target.to_index() as u32,
                relationship_type: RelationshipType::from(reg.monotonicity.unwrap()),
            })
            .collect();

        let model = Model {
            name: name.to_string(),
            variables,
            relationships,
        };

        // each variable gets default layout settings
        let layout_vars = bn
            .variables()
            .map(|var_id| LayoutVariable {
                id: var_id.to_index() as u32,
                name: bn.get_variable_name(var_id).clone(),
                variable_type: VariableType::Default,
                container_id: 0,
                position_x: 0.0,
                position_y: 0.0,
                cell_x: None,
                cell_y: None,
                angle: 0.0,
                description: "".to_string(),
            })
            .collect();

        // a single default container for all the variables
        let container = Container {
            id: 0,
            name: "".to_string(),
            size: 1,
            position_x: 0.0,
            position_y: 0.0,
        };

        let layout = Layout {
            variables: layout_vars,
            containers: vec![container],
            description: "".to_string(),
            zoom_level: None,
            pan_x: None,
            pan_y: None,
        };

        Ok(BmaModel {
            model,
            layout,
            metadata: HashMap::new(),
        })
    }

    /// Check if all variables in the model are Boolean (max level is 1).
    pub fn is_boolean_model(&self) -> bool {
        self.get_max_var_level() <= 1
    }

    /// Get maximum level of any variable in the BMA model.
    pub fn get_max_var_level(&self) -> u32 {
        let mut max_level = 0;
        self.model.variables.iter().for_each(|v| {
            // just in case, lets check both `range_from` and `range_to`
            max_level = max(max_level, v.range_from);
            max_level = max(max_level, v.range_to);
        });
        max_level
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::RelationshipType;
    use crate::model::BmaModel;
    use biodivine_lib_param_bn::BooleanNetwork;

    #[test]
    fn test_from_boolean_network_aeon() {
        let aeon_model = r#"
        $A: A & !B
        $B: A
        B -| A
        A -> A
        A -> B
        "#;
        let bn = BooleanNetwork::try_from(aeon_model).unwrap();

        let bma_model = BmaModel::from_boolean_network(&bn, "Test Model").unwrap();

        /* === VARIABLES AND UPDATE FUNCTIONS === */

        assert_eq!(bma_model.model.variables.len(), 2);
        let var_a_bma = &bma_model.model.variables[0];
        let var_b_bma = &bma_model.model.variables[1];

        assert_eq!(var_a_bma.name, "A");
        assert!(var_a_bma.formula.is_some());
        let formula_a = var_a_bma.formula.as_ref().unwrap().to_string();
        assert_eq!(formula_a, "(var(0) * (1 - var(1)))");

        assert_eq!(var_b_bma.name, "B");
        assert!(var_b_bma.formula.is_some());
        let formula_b = var_b_bma.formula.as_ref().unwrap().to_string();
        assert_eq!(formula_b, "var(0)");

        /* === RELATIONSHIPS === */

        assert_eq!(bma_model.model.relationships.len(), 3);
        let rel_b_inhibits_a = &bma_model.model.relationships[0];
        let rel_a_self_activates = &bma_model.model.relationships[1];
        let rel_a_activates_b = &bma_model.model.relationships[2];
        assert_eq!(rel_b_inhibits_a.from_variable, 1); // B -| A
        assert_eq!(rel_b_inhibits_a.to_variable, 0);
        assert_eq!(
            rel_b_inhibits_a.relationship_type,
            RelationshipType::Inhibitor
        );

        assert_eq!(rel_a_self_activates.from_variable, 0); // A -> A
        assert_eq!(rel_a_self_activates.to_variable, 0);
        assert_eq!(
            rel_a_self_activates.relationship_type,
            RelationshipType::Activator
        );

        assert_eq!(rel_a_activates_b.from_variable, 0); // A -> B
        assert_eq!(rel_a_activates_b.to_variable, 1);
        assert_eq!(
            rel_a_activates_b.relationship_type,
            RelationshipType::Activator
        );

        /* === LAYOUT === */

        assert_eq!(bma_model.layout.variables.len(), 2);
        let layout_var_a = &bma_model.layout.variables[0];
        let layout_var_b = &bma_model.layout.variables[1];
        assert_eq!(layout_var_a.name, "A");
        assert_eq!(layout_var_b.name, "B");

        // Verify that there is a default container
        assert_eq!(bma_model.layout.containers.len(), 1);
        let container = &bma_model.layout.containers[0];
        assert_eq!(container.id, 0);
    }
}
