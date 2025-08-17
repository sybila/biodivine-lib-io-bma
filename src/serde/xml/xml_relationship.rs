use crate::{BmaRelationship, RelationshipType};
use serde::{Deserialize, Serialize};

/// Structure to deserialize XML info about an individual relationship.
///
/// All relationships must have their own ID, type, and IDs of both interacting
/// variables.
///
/// The container ID is optional, and is set to None if not provided.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlRelationship {
    // By default, ID is an attribute, but it can be also present as a child tag.
    #[serde(rename = "@Id", alias = "Id")]
    pub id: u32,
    #[serde(rename = "FromVariableId")]
    pub from_variable_id: u32,
    #[serde(rename = "ToVariableId")]
    pub to_variable_id: u32,
    #[serde(rename = "Type")]
    pub r#type: RelationshipType,
    #[serde(default, rename = "ContainerId")]
    pub container_id: Option<u32>,
}

impl From<XmlRelationship> for BmaRelationship {
    fn from(value: XmlRelationship) -> Self {
        BmaRelationship {
            id: value.id.into(),
            from_variable: value.from_variable_id.into(),
            to_variable: value.to_variable_id.into(),
            r#type: value.r#type,
        }
    }
}

impl From<BmaRelationship> for XmlRelationship {
    fn from(value: BmaRelationship) -> Self {
        XmlRelationship {
            id: value.id.into(),
            from_variable_id: value.from_variable.into(),
            to_variable_id: value.to_variable.into(),
            r#type: value.r#type,
            container_id: None,
        }
    }
}
