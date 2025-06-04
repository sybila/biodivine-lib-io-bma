use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) mod json_layout;
pub(crate) mod json_layout_container;
pub(crate) mod json_layout_variable;
pub(crate) mod json_network;
pub(crate) mod json_relationship;
pub(crate) mod json_variable;

pub(crate) use json_layout::JsonLayout;
pub(crate) use json_layout_container::JsonLayoutContainer;
pub(crate) use json_layout_variable::JsonLayoutVariable;

use crate::utils::take_if_not_blank;
pub(crate) use json_network::JsonNetwork;
pub(crate) use json_relationship::JsonRelationship;
pub(crate) use json_variable::JsonVariable;

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
    /// Names stored in the [`JsonVariable`] are preferred. If such a name is empty,
    /// the value stored in [`JsonLayoutVariable`] is used. If no name is provided, the
    /// variable will not be included in the final map.
    pub fn variable_name_map(&self) -> HashMap<u32, String> {
        let mut map = HashMap::new();

        // First collect variable names stored in the main network.
        for var in &self.network.variables {
            if let Some(name) = take_if_not_blank(var.name.as_str()) {
                map.insert(var.id.into(), name);
            }
        }

        // Then store variable names that are still missing using layout data.
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
