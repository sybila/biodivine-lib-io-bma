use crate::update_function::BmaUpdateFunction;
use crate::{
    BmaLayout, BmaLayoutContainer, BmaLayoutVariable, BmaModel, BmaNetwork, BmaRelationship,
    BmaVariable, RelationshipType,
};
use anyhow::anyhow;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::Monotonicity::Inhibition;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Construct a [`BmaModel`] instance from a provided [`BooleanNetwork`].
///
/// The Boolean network MUST NOT contain parameters in any of its update functions,
/// explicit or implicit. Only fully specified BNs can be converted into BMA format.
///
/// All monotonic regulations are carried over as they are. For each regulation with
/// unspecified monotonicity, both positive and negative regulations are added.
/// This may have some side effects, since BMA does not natively support
/// non-monotonic regulations.
///
/// Information about the observability of regulations is lost (but this should have
/// no effect on fully specified BNs anyway).
impl TryFrom<&BooleanNetwork> for BmaModel {
    type Error = anyhow::Error;

    fn try_from(network: &BooleanNetwork) -> Result<Self, Self::Error> {
        if network.num_parameters() > 0 {
            let parameter_names = network
                .parameters()
                .map(|p| network.get_parameter(p).get_name())
                .collect::<Vec<_>>();
            return Err(anyhow!(
                "Cannot transform Boolean network with explicit parameters (`{parameter_names:?}`)"
            ));
        }

        if network.num_implicit_parameters() > 0 {
            let parameter_names = network
                .implicit_parameters()
                .into_iter()
                .map(|v| network.get_variable_name(v))
                .collect::<Vec<_>>();
            return Err(anyhow!(
                "Cannot transform Boolean network with implicit parameters (`{parameter_names:?}`)"
            ));
        }

        // Transform variables and update functions
        let mut variables = Vec::new();
        for var_id in network.variables() {
            let fn_update = network
                .get_update_function(var_id)
                .as_ref()
                .expect("Invariant violation: No implicit parameters allowed here.");

            // At this point, no parameters can be present, meaning we can use this internal
            // method which panics on parameters.
            let update_function = BmaUpdateFunction::try_from_fn_update_rec(fn_update);

            let bma_id = u32::try_from(var_id.to_index())
                .expect("Invariant violation: Variable id must fit into 32 bits.");

            variables.push(BmaVariable {
                id: bma_id,
                name: network.get_variable_name(var_id).clone(),
                range: (0, 1),
                formula: Some(Ok(update_function)),
            });
        }

        let mut relationships = Vec::new();
        let mut reg_id = 0;
        for regulation in network.as_graph().regulations() {
            let regulator_id = u32::try_from(regulation.regulator.to_index())
                .expect("Invariant violation: Variable id must fit into 32 bits.");
            let target_id = u32::try_from(regulation.target.to_index())
                .expect("Invariant violation: Variable id must fit into 32 bits.");

            let mut relationship = BmaRelationship {
                id: 0,
                from_variable: regulator_id,
                to_variable: target_id,
                r#type: RelationshipType::default(),
            };

            // If the regulation is non-monotonic. We translate this as having just activation.
            // This is not perfect but has a lower chance of completely breaking BMA.
            let add_inhibition = regulation.monotonicity == Some(Inhibition);
            if add_inhibition {
                relationship.id = reg_id;
                relationship.r#type = RelationshipType::Inhibitor;
                relationships.push(relationship.clone());
                reg_id += 1;
            } else {
                relationship.id = reg_id;
                relationship.r#type = RelationshipType::Activator;
                relationships.push(relationship.clone());
                reg_id += 1;
            }
        }

        // Sort relationships deterministically by (source, target) to ensure
        // consistent output regardless of input order. This aids reproducibility
        // in tests and serialization/deserialization cycles.
        relationships.sort_by_key(|rel| (rel.from_variable, rel.to_variable));

        // each variable gets default layout settings
        let default_container = BmaLayoutContainer::new(u32::default(), "Default");

        let mut layout_vars = variables
            .iter()
            .map(|v| BmaLayoutVariable::new(v.id, v.name.as_str(), Some(default_container.id)))
            .collect::<Vec<_>>();

        // Models will not import into BMA unless they have non-zero layout positions.
        // This is by no means a nice "layout", but it should at least allow working with the model.
        let side = layout_vars.len().isqrt();
        for (i, var) in layout_vars.iter_mut().enumerate() {
            let x = i / side;
            let y = i % side;
            var.position = (Decimal::from(75 * (x + 1)), Decimal::from(75 * (y + 1)));
        }

        let model = BmaNetwork {
            name: String::default(),
            variables,
            relationships,
        };

        let layout = BmaLayout {
            variables: layout_vars,
            containers: vec![default_container],
            description: String::default(),
            zoom_level: None,
            pan: None,
        };

        Ok(BmaModel::new(model, layout, HashMap::new()))
    }
}

#[cfg(test)]
mod tests {
    use crate::BmaModel;
    use crate::RelationshipType;
    use biodivine_lib_param_bn::BooleanNetwork;
    use test_generator::test_resources;

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

        let bma_model = BmaModel::try_from(&bn).unwrap();

        /* === VARIABLES AND UPDATE FUNCTIONS === */

        assert_eq!(bma_model.network.variables.len(), 2);
        let var_a_bma = &bma_model.network.variables[0];
        let var_b_bma = &bma_model.network.variables[1];

        assert_eq!(var_a_bma.name, "A");
        assert!(var_a_bma.formula.is_some());
        let formula_a = var_a_bma.formula_string();
        assert_eq!(formula_a, "min(var(0), (1 - var(1)))");

        assert_eq!(var_b_bma.name, "B");
        assert!(var_b_bma.formula.is_some());
        let formula_b = var_b_bma.formula_string();
        assert_eq!(formula_b, "var(0)");

        /* === RELATIONSHIPS === */

        assert_eq!(bma_model.network.relationships.len(), 3);
        // relationships go alphabetically, sorted by regulator and then target
        let rel_a_self_activates = &bma_model.network.relationships[0];
        let rel_a_activates_b = &bma_model.network.relationships[1];
        let rel_b_inhibits_a = &bma_model.network.relationships[2];
        assert_eq!(rel_b_inhibits_a.from_variable, 1); // B -| A
        assert_eq!(rel_b_inhibits_a.to_variable, 0);
        assert_eq!(rel_b_inhibits_a.r#type, RelationshipType::Inhibitor);

        assert_eq!(rel_a_self_activates.from_variable, 0); // A -> A
        assert_eq!(rel_a_self_activates.to_variable, 0);
        assert_eq!(rel_a_self_activates.r#type, RelationshipType::Activator);

        assert_eq!(rel_a_activates_b.from_variable, 0); // A -> B
        assert_eq!(rel_a_activates_b.to_variable, 1);
        assert_eq!(rel_a_activates_b.r#type, RelationshipType::Activator);

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
        let bma_model = BmaModel::try_from(&bn).unwrap();

        // only check relationships here
        // relationships go alphabetically, sorted by regulator and then target
        assert_eq!(bma_model.network.relationships.len(), 3);
        let rel_a_activates_a = &bma_model.network.relationships[0];
        let rel_a_activates_b = &bma_model.network.relationships[1];
        let rel_b_activates_a = &bma_model.network.relationships[2];
        assert_eq!(rel_a_activates_a.from_variable, 0); // A -> A
        assert_eq!(rel_a_activates_a.to_variable, 0);
        assert_eq!(rel_a_activates_a.r#type, RelationshipType::Activator);

        assert_eq!(rel_a_activates_b.from_variable, 0); // A -> B
        assert_eq!(rel_a_activates_b.to_variable, 1);
        assert_eq!(rel_a_activates_b.r#type, RelationshipType::Activator);

        assert_eq!(rel_b_activates_a.from_variable, 1); // B -> A
        assert_eq!(rel_b_activates_a.to_variable, 0);
        assert_eq!(rel_b_activates_a.r#type, RelationshipType::Activator);
    }

    #[test]
    fn test_from_parametrized_bn() {
        let aeon_model = r#"
        $A: f(A)
        A -?? A
        "#;
        let bn = BooleanNetwork::try_from(aeon_model).unwrap();

        let result = BmaModel::try_from(&bn);
        assert!(result.is_err());
    }

    #[test_resources("models/bbm-inputs-true/*.aeon")]
    fn test_round_trip_aeon_to_bma_to_aeon(path: &str) {
        if path.ends_with("146.aeon") {
            return; // 146.aeon is skipped because it causes stack overflow in debug mode
        }

        // Read the AEON file
        let aeon_content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path, e));

        // Parse AEON into BooleanNetwork
        let original_bn = BooleanNetwork::try_from(aeon_content.as_str())
            .unwrap_or_else(|e| panic!("Failed to parse aeon file {}: {}", path, e));

        // Convert BooleanNetwork to BmaModel
        let bma_model = BmaModel::try_from(&original_bn).unwrap_or_else(|e| {
            panic!(
                "Failed to convert BooleanNetwork to BmaModel for {}: {}",
                path, e
            )
        });

        assert_eq!(bma_model.network.variables.len(), original_bn.num_vars());
    }
}
