use crate::serde::json::{JsonBmaModel, JsonRelationship, JsonVariable};
use crate::utils::clone_into_vec;
use crate::{BmaNetwork, BmaVariable};
use serde::{Deserialize, Serialize};

/// Structure to deserialize JSON info about the main model network, with several
/// `variables` that have various `relationships`.
///
/// Variables and relationships are required. The name is optional, and default
/// empty string is used if not provided.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonNetwork {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "Variables", alias = "variables")]
    pub variables: Vec<JsonVariable>,
    #[serde(rename = "Relationships", alias = "relationships")]
    pub relationships: Vec<JsonRelationship>,
}

impl From<BmaNetwork> for JsonNetwork {
    fn from(value: BmaNetwork) -> Self {
        JsonNetwork {
            name: value.name,
            variables: clone_into_vec(&value.variables),
            relationships: clone_into_vec(&value.relationships),
        }
    }
}

impl TryFrom<(&JsonBmaModel, &JsonNetwork)> for BmaNetwork {
    type Error = anyhow::Error;

    fn try_from(value: (&JsonBmaModel, &JsonNetwork)) -> Result<Self, Self::Error> {
        let (model, network) = value;

        Ok(BmaNetwork {
            variables: network
                .variables
                .iter()
                .map(|var| BmaVariable::try_from((model, var)))
                .collect::<Result<Vec<BmaVariable>, Self::Error>>()?,
            relationships: clone_into_vec(&network.relationships),
            name: network.name.clone(),
        })
    }
}
