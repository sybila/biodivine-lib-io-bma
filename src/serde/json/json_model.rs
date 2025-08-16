use crate::serde::json::{JsonLayout, JsonNetwork};
use crate::utils::take_if_not_blank;
use crate::{BmaModel, BmaNetwork};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An intermediate structure purely for deserializing JSON BMA models.
///
/// The functional part of the model is stored in `model` field. The additional `layout`
/// information is optional.
///
/// This structure is intended purely to simplify serialization. It provides virtually no
/// consistency checking. The serialized instances may contain semantically invalid data, such as
/// incorrectly formatted update functions, or variables not matching in layout and model.
/// The full correctness of the model is checked when constructing the final `BmaModel` struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonBmaModel {
    #[serde(rename = "Model", alias = "model")]
    pub network: JsonNetwork,
    #[serde(rename = "Layout", alias = "layout")]
    pub layout: Option<JsonLayout>,
}

impl JsonBmaModel {
    /// Collect all variable names that are known in the model.
    ///
    /// Names stored in the [`crate::JsonVariable`] are preferred. If such a name is empty,
    /// the value stored in [`crate::JsonLayoutVariable`] is used. If no name is provided, the
    /// variable will not be included in the final map.
    pub fn variable_name_map(&self) -> HashMap<u32, String> {
        let mut map = HashMap::new();

        // First collect variable names stored in the main network.
        for var in &self.network.variables {
            if let Some(name) = take_if_not_blank(var.name.as_str()) {
                map.insert(var.id.into(), name);
            }
        }

        // Then store variable names that are still missing using layout serde.
        if let Some(layout) = self.layout.as_ref() {
            for var in &layout.variables {
                if map.contains_key(&var.id.into()) {
                    continue;
                }
                if let Some(name) = take_if_not_blank(var.name.as_str()) {
                    map.insert(var.id.into(), name);
                }
            }
        }

        map
    }
}

impl From<BmaModel> for JsonBmaModel {
    fn from(value: BmaModel) -> Self {
        JsonBmaModel {
            network: value.network.into(),
            layout: Some(value.layout.into()),
        }
    }
}

impl TryFrom<JsonBmaModel> for BmaModel {
    type Error = anyhow::Error; // TODO: Replace with type safe error.

    fn try_from(json_model: JsonBmaModel) -> Result<BmaModel, anyhow::Error> {
        // Convert the model
        let model = BmaNetwork::from((&json_model, &json_model.network));

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
