use crate::model::bma_model::*;
use crate::update_fn::bma_fn_update::BmaFnUpdate;

use crate::data::json_model::{JsonBmaModel, JsonVariable};
use crate::{BmaLayout, BmaNetwork, BmaVariable};
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

    /// Create a new BMA model from a model string in JSON format.
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
    /// If the update function has an incorrect format, we return an error.
    fn convert_json_variable(
        json_var: JsonVariable,
        json_model: &JsonBmaModel,
        all_vars: &HashMap<u32, String>,
    ) -> Result<BmaVariable, String> {
        // We have already precomputed a set of all ID-name mappings in the model and layout
        let name = all_vars.get(&json_var.id.into()).unwrap();

        // Get a set of regulators for the variable that we'll pass to update fn parser
        let regulators = json_model.get_regulators(json_var.id.into());
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
            id: json_var.id.into(),
            name: Some(name.clone()),
            range: (json_var.range_from.into(), json_var.range_to.into()),
            formula,
        })
    }
}

impl TryFrom<JsonBmaModel> for BmaModel {
    type Error = String;

    /// Convert JsonBmaModel instance into a proper BmaModel instance.
    ///
    /// Returns error if the update function has an incorrect format.
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
                .map(|it| it.into())
                .collect(),
            name: Some(json_model.model.name),
        };

        // Convert the layout
        let layout = json_model
            .layout
            .map(|layout| BmaLayout {
                variables: layout.variables.into_iter().map(|it| it.into()).collect(),
                containers: layout.containers.into_iter().map(|it| it.into()).collect(),
                description: Some(layout.description),
                zoom_level: None,
                pan: None,
            })
            .unwrap_or_default(); // Default empty layout, if layout is not provided.

        // Metadata is not present in JsonBmaModel
        let metadata = HashMap::new();

        Ok(BmaModel::new(model, layout, metadata))
    }
}
