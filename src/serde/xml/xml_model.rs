use crate::serde::xml::{XmlContainers, XmlLayout, XmlRelationships, XmlVariables};
use crate::utils::{clone_into_vec, take_if_not_blank};
use crate::{BmaLayout, BmaModel, BmaNetwork, BmaVariable};
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
    /// Collects set of all variables in the model, creating ID-name mapping.
    pub fn collect_all_variables(&self) -> HashMap<u32, String> {
        self.variables
            .variable
            .iter()
            .map(|var| (var.id.into(), var.name.clone()))
            .collect::<HashMap<u32, String>>()
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
            name: model.network.name_or_default(),
            description: model.layout.description.clone().unwrap_or_default(),
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

impl TryFrom<XmlBmaModel> for BmaModel {
    type Error = anyhow::Error; // TODO: Replace with type safe error.

    fn try_from(value: XmlBmaModel) -> Result<Self, Self::Error> {
        let network = BmaNetwork {
            name: take_if_not_blank(value.name.as_str()),
            variables: value
                .variables
                .variable
                .iter()
                .map(|v| (&value, v).try_into())
                .collect::<Result<Vec<BmaVariable>, Self::Error>>()?,
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

        Ok(BmaModel {
            network,
            layout,
            metadata,
        })
    }
}
