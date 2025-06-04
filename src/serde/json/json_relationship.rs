use crate::serde::quote_num::QuoteNum;
use crate::{BmaRelationship, RelationshipType};
use serde::{Deserialize, Serialize};

/// Structure to deserialize JSON info about an individual relationship.
///
/// All relationships must have their own ID, type, and IDs of both interacting
/// variables.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonRelationship {
    #[serde(rename = "Id", alias = "id")]
    pub id: QuoteNum,
    #[serde(
        rename = "FromVariable",
        alias = "fromVariable",
        alias = "fromVariableId",
        alias = "FromVariableId"
    )]
    pub from_variable: QuoteNum,
    #[serde(
        rename = "ToVariable",
        alias = "toVariable",
        alias = "toVariableId",
        alias = "ToVariableId"
    )]
    pub to_variable: QuoteNum,
    #[serde(rename = "Type", alias = "type")]
    pub r#type: RelationshipType,
}

impl From<JsonRelationship> for BmaRelationship {
    fn from(value: JsonRelationship) -> Self {
        BmaRelationship {
            id: value.id.into(),
            from_variable: value.from_variable.into(),
            to_variable: value.to_variable.into(),
            r#type: value.r#type,
        }
    }
}

impl From<BmaRelationship> for JsonRelationship {
    fn from(value: BmaRelationship) -> Self {
        JsonRelationship {
            id: value.id.into(),
            from_variable: value.from_variable.into(),
            to_variable: value.to_variable.into(),
            r#type: value.r#type,
        }
    }
}
