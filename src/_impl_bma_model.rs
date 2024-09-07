use crate::bma_model::*;
use crate::enums::{RelationshipType, VariableType};
use crate::json_model::JsonBmaModel;
use crate::traits::{JsonSerDe, XmlDe};
use crate::update_fn::bma_fn_tree::BmaFnUpdate;
use crate::update_fn::parser::parse_bma_formula;
use crate::xml_model::XmlBmaModel;
use biodivine_lib_param_bn::{BooleanNetwork, RegulatoryGraph};
use std::collections::HashMap;

impl<'de> JsonSerDe<'de> for BmaModel {
    fn to_json_str(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn to_pretty_json_str(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn from_json_str(json_str: &'de str) -> Result<Self, String> {
        let json_model: JsonBmaModel = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
        let model = BmaModel::from(json_model);
        Ok(model)
    }
}

impl<'de> XmlDe<'de> for BmaModel {
    fn from_xml_str(xml_str: &'de str) -> Result<Self, String> {
        let xml_model: XmlBmaModel = serde_xml_rs::from_str(xml_str).map_err(|e| e.to_string())?;
        let model = BmaModel::from(xml_model);
        Ok(model)
    }
}

impl From<JsonBmaModel> for BmaModel {
    fn from(json_model: JsonBmaModel) -> Self {
        // Create a mapping from variable IDs to their names from the layout
        let layout_var_names: HashMap<u32, String> = json_model
            .layout
            .as_ref()
            .map(|layout| {
                layout
                    .variables
                    .iter()
                    .filter(|layout_var| layout_var.name.is_some())
                    .map(|layout_var| (layout_var.id, layout_var.name.clone().unwrap()))
                    .collect()
            })
            .unwrap_or_default();

        // Convert the model
        let model = Model {
            name: json_model.model.name,
            variables: json_model
                .model
                .variables
                .into_iter()
                .map(|var| Variable {
                    id: var.id,
                    name: var
                        .name
                        .unwrap_or(layout_var_names.get(&var.id).cloned().unwrap_or_default()), // Use the name from layout
                    range_from: var.range_from,
                    range_to: var.range_to,
                    formula: if var.formula.is_empty() {
                        // TODO: handle incorrectly parsed formulas
                        parse_bma_formula(&var.formula).ok()
                    } else {
                        None
                    },
                })
                .collect(),
            relationships: json_model
                .model
                .relationships
                .into_iter()
                .map(|rel| Relationship {
                    id: rel.id,
                    from_variable: rel.from_variable,
                    to_variable: rel.to_variable,
                    relationship_type: rel.r#type,
                })
                .collect(),
        };

        // Convert the layout
        let layout = json_model
            .layout
            .map(|layout| Layout {
                variables: layout
                    .variables
                    .into_iter()
                    .map(|var| LayoutVariable {
                        id: var.id,
                        name: var.name.unwrap_or_default(),
                        container_id: var.container_id,
                        variable_type: var.r#type,
                        position_x: var.position_x,
                        position_y: var.position_y,
                        cell_x: var.cell_x,
                        cell_y: var.cell_y,
                        angle: var.angle,
                        description: var.description.unwrap_or_default(),
                    })
                    .collect(),
                containers: layout
                    .containers
                    .into_iter()
                    .map(|container| Container {
                        id: container.id,
                        name: container.name.unwrap_or_default(),
                        size: container.size,
                        position_x: container.position_x,
                        position_y: container.position_y,
                    })
                    .collect(),
                description: layout.description.unwrap_or_default(),
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            })
            .unwrap_or_else(|| Layout {
                variables: vec![],
                containers: vec![],
                description: String::default(),
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            });

        // metadata not present in JsonBmaModel
        let metadata = HashMap::new();

        BmaModel {
            model,
            layout,
            metadata,
        }
    }
}

impl From<XmlBmaModel> for BmaModel {
    fn from(xml_model: XmlBmaModel) -> Self {
        // Convert the model
        let model = Model {
            name: xml_model.name,
            variables: xml_model
                .variables
                .variable
                .clone()
                .into_iter()
                .map(|var| Variable {
                    id: var.id,
                    name: var.name,
                    range_from: var.range_from,
                    range_to: var.range_to,
                    formula: if var.formula.is_empty() {
                        // TODO: handle incorrectly parsed formulas
                        parse_bma_formula(&var.formula).ok()
                    } else {
                        None
                    },
                })
                .collect(),
            relationships: xml_model
                .relationships
                .relationship
                .into_iter()
                .map(|rel| Relationship {
                    id: rel.id,
                    from_variable: rel.from_variable_id,
                    to_variable: rel.to_variable_id,
                    relationship_type: rel.r#type,
                })
                .collect(),
        };

        // Convert the layout
        let layout = Layout {
            variables: xml_model
                .variables
                .variable
                .into_iter()
                .map(|var| LayoutVariable {
                    id: var.id,
                    name: var.name,
                    variable_type: var.r#type,
                    container_id: var.container_id,
                    position_x: var.position_x,
                    position_y: var.position_y,
                    cell_x: Some(var.cell_x),
                    cell_y: Some(var.cell_y),
                    angle: var.angle,
                    description: String::default(),
                })
                .collect(),
            containers: xml_model
                .containers
                .container
                .into_iter()
                .map(|container| Container {
                    id: container.id,
                    name: container.name,
                    size: container.size,
                    position_x: container.position_x,
                    position_y: container.position_y,
                })
                .collect(),
            description: xml_model.description,
            zoom_level: Some(xml_model.layout.zoom_level),
            pan_x: Some(xml_model.layout.pan_x),
            pan_y: Some(xml_model.layout.pan_y),
        };

        // Metadata can be constructed from various XML fields
        let mut metadata = HashMap::new();
        metadata.insert("biocheck_version".to_string(), xml_model.biocheck_version);
        metadata.insert("created_date".to_string(), xml_model.created_date);
        metadata.insert("modified_date".to_string(), xml_model.modified_date);

        BmaModel {
            model,
            layout,
            metadata,
        }
    }
}

impl BmaModel {
    pub fn to_regulatory_graph(&self) -> Result<RegulatoryGraph, String> {
        let mut variables_map: HashMap<u32, String> = HashMap::new();
        for var in &self.model.variables {
            let inserted =
                variables_map.insert(var.id, format!("v_{}_{}", var.id, var.name.clone()));
            if inserted.is_some() {
                return Err(format!("Variable ID {} is not unique.", var.id));
            }
        }
        let variables = variables_map.clone().into_values().collect();
        let mut graph = RegulatoryGraph::new(variables);

        // add regulations
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

    pub fn to_boolean_network(&self) -> Result<BooleanNetwork, String> {
        let graph = self.to_regulatory_graph()?;
        let bn = BooleanNetwork::new(graph);

        // add update functions
        self.model.variables.iter().for_each(|_var| {
            // todo - convert the formula to update functions
        });

        Ok(bn)
    }

    /// Construct `BmaModel` instance with a given name from a provided BooleanNetwork `bn`.
    ///
    /// Boolean network must not use function symbols in any of its update functions.
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
}

#[cfg(test)]
mod tests {
    use crate::bma_model::BmaModel;
    use crate::enums::RelationshipType;
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
