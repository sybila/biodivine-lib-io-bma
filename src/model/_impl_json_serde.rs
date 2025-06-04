use crate::model::bma_model::*;

use crate::BmaNetwork;
use crate::data::json_model::JsonBmaModel;
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

impl TryFrom<JsonBmaModel> for BmaModel {
    type Error = String;

    /// Convert JsonBmaModel instance into a proper BmaModel instance.
    ///
    /// Returns error if the update function has an incorrect format.
    fn try_from(json_model: JsonBmaModel) -> Result<BmaModel, String> {
        // Convert the model
        let model = BmaNetwork::try_from((&json_model, &json_model.network))?;

        // Convert the layout
        let layout = json_model
            .layout
            .map(|layout| layout.into())
            .unwrap_or_default(); // Default empty layout, if layout is not provided.

        // Metadata is not present in JsonBmaModel
        let metadata = HashMap::new();

        Ok(BmaModel::new(model, layout, metadata))
    }
}
