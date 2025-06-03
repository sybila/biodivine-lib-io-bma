use crate::enums::RelationshipType;
use serde::{Deserialize, Serialize};

/// A relationship of a given type between two variables.
/// All fields are required.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BmaRelationship {
    pub id: u32,
    pub from_variable: u32,
    pub to_variable: u32,
    #[serde(rename = "Type")]
    pub relationship_type: RelationshipType, // Corresponds to "Type" in JSON/XML
}
