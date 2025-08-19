use crate::serde::xml::{XmlContainers, XmlLayout, XmlRelationships, XmlVariables};
use crate::utils::clone_into_vec;
use crate::{BmaLayout, BmaModel, BmaNetwork};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An intermediate structure purely for deserializing XML BMA models.
///
/// We require only the functional parts of the model - variables and relationships.
/// Additional important strings (id, name, description) are set to empty if not provided.
/// Layout information and containers are optional.
/// We also parse some additional metadata items often present, but that is optional as well.
///
/// This structure is intended purely to simplify serialization. The serialized instances may
/// contain semantically invalid data, such as incorrectly formatted update functions, or
/// variables not matching in layout and model. The full correctness of the model is checked
/// by the final `BmaModel` struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Model")]
pub(crate) struct XmlBmaModel {
    #[serde(default, rename = "Variables")]
    pub variables: XmlVariables,
    #[serde(default, rename = "Relationships")]
    pub relationships: XmlRelationships,

    #[serde(default, rename = "@Id", alias = "Id")]
    pub id: String,
    #[serde(default, rename = "@Name", alias = "Name", alias = "@ModelName")]
    pub name: String,
    #[serde(default, rename = "Description")]
    pub description: String,
    #[serde(default, rename = "Layout")]
    pub layout: Option<XmlLayout>,
    #[serde(default, rename = "Containers")]
    pub containers: Option<XmlContainers>,

    #[serde(default, rename = "@BioCheckVersion", alias = "BioCheckVersion")]
    pub biocheck_version: Option<String>,
    #[serde(default, rename = "CreatedDate")]
    pub created_date: Option<String>,
    #[serde(default, rename = "ModifiedDate")]
    pub modified_date: Option<String>,
}

impl XmlBmaModel {
    /// Collect all regulators of a specific variable.
    pub fn regulators(&self, variable: u32) -> Vec<(u32, String)> {
        self.relationships
            .relationship
            .iter()
            .filter(|r| r.to_variable_id == variable)
            .map(|r| r.from_variable_id)
            .filter_map(|id| {
                self.variables
                    .variable
                    .iter()
                    .find(|v| v.id == id)
                    .map(|v| (id, v.name.clone()))
            })
            .collect()
    }
}

impl From<BmaModel> for XmlBmaModel {
    fn from(model: BmaModel) -> Self {
        XmlBmaModel {
            variables: XmlVariables {
                variable: clone_into_vec(&model.network.variables),
            },
            relationships: XmlRelationships {
                relationship: clone_into_vec(&model.network.relationships),
            },
            id: "".to_string(),
            name: model.network.name.clone(),
            description: model.layout.description.clone(),
            layout: Some(model.layout.clone().into()),
            containers: Some(XmlContainers {
                container: clone_into_vec(&model.layout.containers),
            }),
            biocheck_version: model.metadata.get("biocheck_version").cloned(),
            created_date: model.metadata.get("created_date").cloned(),
            modified_date: model.metadata.get("modified_date").cloned(),
        }
    }
}

impl From<XmlBmaModel> for BmaModel {
    fn from(value: XmlBmaModel) -> Self {
        let network = BmaNetwork {
            name: value.name.clone(),
            variables: value
                .variables
                .variable
                .iter()
                .map(|v| (&value, v).into())
                .collect::<Vec<_>>(),
            relationships: clone_into_vec(&value.relationships.relationship),
        };

        let layout = BmaLayout::from(&value);

        // Metadata can be constructed from various XML fields
        let mut metadata = HashMap::new();
        if let Some(biocheck_version) = &value.biocheck_version {
            metadata.insert("biocheck_version".to_string(), biocheck_version.clone());
        }
        if let Some(created_date) = &value.created_date {
            metadata.insert("created_date".to_string(), created_date.clone());
        }
        if let Some(modified_date) = &value.modified_date {
            metadata.insert("modified_date".to_string(), modified_date.clone());
        }

        BmaModel {
            network,
            layout,
            metadata,
        }
    }
}
