use crate::enums::RelationshipType;
use crate::model::BmaNetwork;
use crate::model::bma_model::*;
use crate::update_fn::bma_fn_update::BmaFnUpdate;
use biodivine_lib_param_bn::{BooleanNetwork, Regulation};
use std::cmp::max;
use std::collections::HashMap;

impl BmaModel {
    /// Create a new BMA model with a given network, layout, and metadata.
    /// This is just a wrapper, it does not check the validity of the model.
    pub fn new(model: BmaNetwork, layout: BmaLayout, metadata: HashMap<String, String>) -> Self {
        BmaModel {
            model,
            layout,
            metadata,
        }
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

    /// Get regulators of a particular variable.
    /// Returns a tuple of two vectors: positive and negative regulators.
    /// The regulators are represented by their IDs.
    pub fn get_regulators(&self, target_var: u32) -> (Vec<u32>, Vec<u32>) {
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        self.model
            .relationships
            .iter()
            .filter(|rel| rel.to_variable == target_var)
            .for_each(|rel| {
                if rel.relationship_type == RelationshipType::Activator {
                    positive.push(rel.from_variable);
                } else if rel.relationship_type == RelationshipType::Inhibitor {
                    negative.push(rel.from_variable);
                }
            });
        (positive, negative)
    }

    /// Construct `BmaModel` instance from a provided BooleanNetwork `bn`.
    ///
    /// The Boolean network MUST NOT contain parameters in any of its update functions,
    /// explicit or implicit. Only fully specified BNs can be converted into BMA format.
    ///
    /// All monotonic regulations are carried over as they are. For each regulation with
    /// unspecified monotonicity, both a positive and a negative regulation are added.
    /// This may have some side effects, but BMA does not support non-monotonic regulations.
    ///
    /// Information about observability of regulations is lost (but this should have no effect
    /// for fully specified BNs anyway).
    pub fn from_boolean_network(bn: &BooleanNetwork, name: &str) -> Result<BmaModel, String> {
        if bn.num_parameters() > 0 {
            return Err(
                "Boolean network with parameters can not be transfromed to BMA.".to_string(),
            );
        }

        // transform variables and update functions
        let variables = bn
            .variables()
            .map(|var_id| {
                let formula = if let Some(update_fn) = bn.get_update_function(var_id) {
                    // we unwrap since we already checked BN has no parameters
                    let bma_function = BmaFnUpdate::try_from_fn_update(update_fn)?;
                    Some(bma_function)
                } else {
                    None
                };
                let var = BmaVariable {
                    id: var_id.to_index() as u32,
                    name: bn.get_variable_name(var_id).clone(),
                    range_from: 0,
                    range_to: 1,
                    formula,
                };
                Ok(var)
            })
            .collect::<Result<Vec<BmaVariable>, String>>()?;

        // transform monotonic regulations into relationships, ignore non-monotonic for now
        // TODO: deal with non-monotonic regulations (ignored for now)
        let mut relationships: Vec<BmaRelationship> = bn
            .as_graph()
            .regulations()
            .filter(|reg| reg.monotonicity.is_some())
            .enumerate()
            .map(|(idx, reg)| BmaRelationship {
                id: idx as u32,
                from_variable: reg.regulator.to_index() as u32,
                to_variable: reg.target.to_index() as u32,
                relationship_type: RelationshipType::from(reg.monotonicity.unwrap()),
            })
            .collect();

        // for each non-monotonic regulation, add regulations of both signs
        let non_monotonic_regs: Vec<Regulation> = bn
            .as_graph()
            .regulations()
            .filter(|reg| reg.monotonicity.is_none())
            .cloned()
            .collect();
        let num_regs_before = relationships.len();
        for (base_idx, reg) in non_monotonic_regs.iter().enumerate() {
            relationships.push(BmaRelationship {
                id: (num_regs_before + base_idx * 2) as u32,
                from_variable: reg.regulator.to_index() as u32,
                to_variable: reg.target.to_index() as u32,
                relationship_type: RelationshipType::Activator,
            });
            relationships.push(BmaRelationship {
                id: (num_regs_before + base_idx * 2 + 1) as u32,
                from_variable: reg.regulator.to_index() as u32,
                to_variable: reg.target.to_index() as u32,
                relationship_type: RelationshipType::Inhibitor,
            });
        }

        // sort relationships for deterministic alphabetical order
        relationships.sort_by_key(|rel| (rel.from_variable, rel.to_variable));

        let model = BmaNetwork {
            name: name.to_string(),
            variables,
            relationships,
        };

        // each variable gets default layout settings
        let container_id = 0; // default container ID
        let layout_vars = bn
            .variables()
            .map(|var_id| {
                let id = var_id.to_index() as u32;
                let name = bn.get_variable_name(var_id).clone();
                BmaLayoutVariable::new_default(id, name, Some(container_id))
            })
            .collect();

        // a single default container for all the variables
        let container = BmaContainer::new_default(container_id, "Default".to_string());

        let layout = BmaLayout {
            variables: layout_vars,
            containers: vec![container],
            description: "".to_string(),
            zoom_level: None,
            pan_x: None,
            pan_y: None,
        };

        Ok(BmaModel::new(model, layout, HashMap::new()))
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::RelationshipType;
    use crate::model::BmaModel;
    use biodivine_lib_param_bn::BooleanNetwork;

    #[test]
    fn test_from_bn() {
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
        // relationships go alphabetically, sorted by regulator and then target
        let rel_a_self_activates = &bma_model.model.relationships[0];
        let rel_a_activates_b = &bma_model.model.relationships[1];
        let rel_b_inhibits_a = &bma_model.model.relationships[2];
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

    #[test]
    fn test_from_bn_non_monotonic() {
        let aeon_model = r#"
        $A: (A & B) | (!A & !B)
        $B: A
        B -? A
        A -? A
        A -> B
        "#;
        let bn = BooleanNetwork::try_from(aeon_model).unwrap();
        let bma_model = BmaModel::from_boolean_network(&bn, "Test Model").unwrap();

        // only check relationships here
        // relationships go alphabetically, sorted by regulator and then target
        assert_eq!(bma_model.model.relationships.len(), 5);
        let rel_a_activates_a = &bma_model.model.relationships[0];
        let rel_a_inhibits_a = &bma_model.model.relationships[1];
        let rel_a_activates_b = &bma_model.model.relationships[2];
        let rel_b_activates_a = &bma_model.model.relationships[3];
        let rel_b_inhibits_a = &bma_model.model.relationships[4];
        assert_eq!(rel_a_activates_a.from_variable, 0); // A -> A
        assert_eq!(rel_a_activates_a.to_variable, 0);
        assert_eq!(
            rel_a_activates_a.relationship_type,
            RelationshipType::Activator
        );

        assert_eq!(rel_a_inhibits_a.from_variable, 0); // A -| A
        assert_eq!(rel_a_inhibits_a.to_variable, 0);
        assert_eq!(
            rel_a_inhibits_a.relationship_type,
            RelationshipType::Inhibitor
        );

        assert_eq!(rel_a_activates_b.from_variable, 0); // A -> B
        assert_eq!(rel_a_activates_b.to_variable, 1);
        assert_eq!(
            rel_a_activates_b.relationship_type,
            RelationshipType::Activator
        );

        assert_eq!(rel_b_activates_a.from_variable, 1); // B -> A
        assert_eq!(rel_b_activates_a.to_variable, 0);
        assert_eq!(
            rel_b_activates_a.relationship_type,
            RelationshipType::Activator
        );

        assert_eq!(rel_b_inhibits_a.from_variable, 1); // B -| A
        assert_eq!(rel_b_inhibits_a.to_variable, 0);
        assert_eq!(
            rel_b_inhibits_a.relationship_type,
            RelationshipType::Inhibitor
        );
    }

    #[test]
    fn test_from_parametrized_bn() {
        let aeon_model = r#"
        $A: f(A)
        A -?? A
        "#;
        let bn = BooleanNetwork::try_from(aeon_model).unwrap();

        let result = BmaModel::from_boolean_network(&bn, "Test Model");
        assert!(result.is_err());
    }
}
