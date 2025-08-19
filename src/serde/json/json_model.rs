use crate::serde::json::{JsonLayout, JsonNetwork};
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
    #[serde(default, rename = "Layout", alias = "layout")]
    pub layout: Option<JsonLayout>,
}

impl JsonBmaModel {
    /// Collect all regulators of a specific variable.
    pub fn regulators(&self, variable: u32) -> Vec<(u32, String)> {
        self.network
            .relationships
            .iter()
            .filter(|r| u32::from(r.to_variable) == variable)
            .map(|r| u32::from(r.from_variable))
            .filter_map(|id| {
                self.network
                    .variables
                    .iter()
                    .find(|v| u32::from(v.id) == id)
                    .map(|v| (id, v.name.clone()))
            })
            .collect::<Vec<_>>()
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

impl From<JsonBmaModel> for BmaModel {
    fn from(json_model: JsonBmaModel) -> BmaModel {
        // Convert the model
        let model = BmaNetwork::from((&json_model, &json_model.network));

        // Convert the layout
        let layout = json_model
            .layout
            .map(|layout| layout.into())
            .unwrap_or_default(); // Default empty layout, if layout is not provided.

        // Metadata is not present in JsonBmaModel
        let metadata = HashMap::new();

        BmaModel::new(model, layout, metadata)
    }
}
