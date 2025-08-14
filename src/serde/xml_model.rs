use crate::serde::xml::{XmlContainer, XmlLayout, XmlRelationship, XmlVariable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An intermediate structure purely for deserializing XML BMA models.
///
/// We require only the functional parts of the model - variables and relationships.
/// Additional important strings (id, name, description) are set to empty if not provided.
/// Layout information and containers are optional.
/// We also parse some additional metadata items often present, but that is optional as well.
///
/// This structure is intended purely to simplify serialization. It does not provide much of a
/// consistency checking. The serialized instances may contain semantically invalid serde, such as
/// incorrectly formatted update functions, or variables not matching in layout and model.
/// The full correctness of the model is checked when constructing the final `BmaModel` struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Model")]
pub(crate) struct XmlBmaModel {
    #[serde(rename = "Variables")]
    pub variables: XmlVariables,
    #[serde(rename = "Relationships")]
    pub relationships: XmlRelationships,

    #[serde(default, rename = "@Id", alias = "Id")]
    pub id: String,
    #[serde(default, rename = "@Name", alias = "Name", alias = "@ModelName")]
    pub name: String,
    #[serde(default, rename = "Description")]
    pub description: String,
    #[serde(rename = "Layout")]
    pub layout: Option<XmlLayout>,
    #[serde(rename = "Containers")]
    pub containers: Option<XmlContainers>,

    #[serde(rename = "@BioCheckVersion", alias = "BioCheckVersion")]
    pub biocheck_version: Option<String>,
    #[serde(rename = "CreatedDate")]
    pub created_date: Option<String>,
    #[serde(rename = "ModifiedDate")]
    pub modified_date: Option<String>,
}

/// Structure to deserialize XML info about container list. Just a wrapper
/// for actual containers list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlContainers {
    #[serde(default, rename = "Container")]
    pub container: Vec<XmlContainer>,
}

/// Structure to deserialize XML info about variables list. Just a wrapper
/// for actual variables list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlVariables {
    #[serde(default, rename = "Variable")]
    pub variable: Vec<XmlVariable>,
}

/// Structure to deserialize XML info about relationships list. Just a wrapper
/// for actual relationships list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlRelationships {
    #[serde(default, rename = "Relationship")]
    pub relationship: Vec<XmlRelationship>,
}

impl XmlBmaModel {
    /// Collects set of all variables in the model, creating ID-name mapping.
    pub fn collect_all_variables(&self) -> HashMap<u32, String> {
        self.variables
            .variable
            .iter()
            .map(|var| (var.id.into(), var.name.clone()))
            .collect::<HashMap<u32, String>>()
    }
}
