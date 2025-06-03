use crate::json_model::*;
use crate::model::bma_model::*;
use crate::update_fn::bma_fn_update::BmaFnUpdate;

use crate::model::bma_relationship::BmaRelationship;
use crate::model::{BmaLayout, BmaLayoutVariable, BmaNetwork, BmaVariable};
use std::collections::HashMap;

impl BmaModel {
    /// Convert the `BmaModel` into a JSON string.
    /// Internally, we use serde_json for the conversion.
    pub fn to_json_str(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Convert the `BmaModel` into a pretty formatted JSON string.
    /// Internally, we use serde_json for the conversion.
    pub fn to_pretty_json_str(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    /// Create new BMA model from a model string in JSON format.
    /// Internally, we use json_serde serialization into an intermediate `JsonBmaModel` structure.
    pub fn from_json_str(json_str: &str) -> Result<Self, String> {
        let json_model: JsonBmaModel = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
        BmaModel::try_from(json_model)
    }
}

impl BmaModel {
    /// Convert JsonVariable into a proper BmaVariable instance.
    /// If there is no name in the JSON variable, we try to find it in the layout variables.
    /// If there is no name in the layout either, we use a default empty string.
    ///
    /// If the update function has incorrect format, we return an error.
    fn convert_json_variable(
        json_var: JsonVariable,
        json_model: &JsonBmaModel,
        all_vars: &HashMap<u32, String>,
    ) -> Result<BmaVariable, String> {
        // We have already precomputed set of all ID-name mappings in the model and layout
        let name = all_vars.get(&json_var.id).unwrap();

        // Get a set of regulators for the variable that we'll pass to update fn parser
        let regulators = json_model.get_regulators(json_var.id);
        let named_regulators = all_vars
            .clone()
            .into_iter()
            .filter(|(id, _)| regulators.contains(id))
            .collect::<HashMap<u32, String>>();

        // Try to parse the update function from the JSON variable
        let formula = if !json_var.formula.is_empty() {
            Some(BmaFnUpdate::parse_from_str(
                &json_var.formula,
                &named_regulators,
            )?)
        } else {
            None
        };

        Ok(BmaVariable {
            id: json_var.id,
            name: name.clone(),
            range_from: json_var.range_from,
            range_to: json_var.range_to,
            formula,
        })
    }

    /// Convert JSON relationship into a proper BmaRelationship instance.
    fn convert_json_relationship(json_rel: JsonRelationship) -> BmaRelationship {
        BmaRelationship {
            id: json_rel.id,
            from_variable: json_rel.from_variable,
            to_variable: json_rel.to_variable,
            relationship_type: json_rel.r#type,
        }
    }

    /// Convert JsonLayoutVariable instance into a proper BmaLayoutVariable
    /// instance. If there was no name or description in the JSON layout variable, we use
    /// a default empty string.
    fn convert_json_layout_variable(json_var: JsonLayoutVariable) -> BmaLayoutVariable {
        BmaLayoutVariable {
            id: json_var.id,
            name: json_var.name,
            container_id: json_var.container_id,
            variable_type: json_var.r#type,
            position_x: json_var.position_x,
            position_y: json_var.position_y,
            cell_x: json_var.cell_x,
            cell_y: json_var.cell_y,
            angle: json_var.angle,
            description: json_var.description,
        }
    }

    /// Convert JsonContainer instance into a proper BmaContainer instance.
    /// If there was no name in the JSON container, we use a default empty string.
    fn convert_json_container(json_container: JsonContainer) -> BmaContainer {
        BmaContainer {
            id: json_container.id,
            name: json_container.name,
            size: json_container.size,
            position_x: json_container.position_x,
            position_y: json_container.position_y,
        }
    }
}

impl TryFrom<JsonBmaModel> for BmaModel {
    type Error = String;

    /// Convert JsonBmaModel instance into a proper BmaModel instance.
    ///
    /// Returns error if the update function has incorrect format.
    fn try_from(json_model: JsonBmaModel) -> Result<BmaModel, String> {
        // For all variables, collect ID-name mapping (combining info from model and layout)
        let all_variables: HashMap<u32, String> = json_model.collect_all_variables();

        // Convert the model
        let model = BmaNetwork {
            variables: json_model
                .model
                .variables
                .iter()
                .map(|var| Self::convert_json_variable(var.clone(), &json_model, &all_variables))
                .collect::<Result<Vec<BmaVariable>, String>>()?,
            relationships: json_model
                .model
                .relationships
                .into_iter()
                .map(Self::convert_json_relationship)
                .collect(),
            name: json_model.model.name,
        };

        // Convert the layout
        let layout = json_model
            .layout
            .map(|layout| BmaLayout {
                variables: layout
                    .variables
                    .into_iter()
                    .map(Self::convert_json_layout_variable)
                    .collect(),
                containers: layout
                    .containers
                    .into_iter()
                    .map(Self::convert_json_container)
                    .collect(),
                description: layout.description,
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            })
            .unwrap_or_default(); // Default empty layout if not provided

        // Metadata not present in JsonBmaModel
        let metadata = HashMap::new();

        Ok(BmaModel::new(model, layout, metadata))
    }
}
