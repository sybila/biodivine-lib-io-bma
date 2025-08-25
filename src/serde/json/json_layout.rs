use crate::BmaLayout;
use crate::serde::json::{JsonLayoutContainer, JsonLayoutVariable};
use crate::utils::clone_into_vec;
use serde::{Deserialize, Serialize};

/// Structure to deserialize JSON info about layout, which contains variables,
/// containers, and a description.
///
/// All of these can be missing in the JSON. If not provided, default empty values
/// are used.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonLayout {
    #[serde(default, rename = "Variables", alias = "variables")]
    pub variables: Vec<JsonLayoutVariable>,
    #[serde(default, rename = "Containers", alias = "containers")]
    pub containers: Vec<JsonLayoutContainer>,
    #[serde(default, rename = "Description", alias = "description")]
    pub description: String,
}

impl From<JsonLayout> for BmaLayout {
    fn from(value: JsonLayout) -> Self {
        BmaLayout {
            variables: clone_into_vec(&value.variables),
            containers: clone_into_vec(&value.containers),
            description: value.description,
            zoom_level: None,
            pan: None,
        }
    }
}

impl From<BmaLayout> for JsonLayout {
    fn from(value: BmaLayout) -> Self {
        JsonLayout {
            variables: clone_into_vec(&value.variables),
            containers: clone_into_vec(&value.containers),
            description: value.description,
        }
    }
}
