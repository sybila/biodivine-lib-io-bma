use crate::enums::{RelationshipType, VariableType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Custom deserializer function for (potentially) quoted integers (like "42").
///
/// For some reason, this is XML-specific, and we have to use different variant for JSON.
fn deser_quoted_int<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let trimmed = s.trim_matches('"');
    u32::from_str(trimmed).map_err(serde::de::Error::custom)
}

/// An intermediate structure for deserializing XML BMA models.
///
/// This structure is intended purely to simplify serialization. It does not provide much of a
/// consistency checking. The serialized instances may contain semantically invalid data, such as
/// incorrectly formatted update functions, or variables not matching in layout and model.
/// The full correctness of the model is checked when constructing the final `BmaModel` struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Model")]
pub(crate) struct XmlBmaModel {
    #[serde(rename = "Id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "BioCheckVersion")]
    pub biocheck_version: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "CreatedDate")]
    pub created_date: String,
    #[serde(rename = "ModifiedDate")]
    pub modified_date: String,
    #[serde(rename = "Layout")]
    pub layout: XmlLayout,
    #[serde(rename = "Containers")]
    pub containers: XmlContainers,
    #[serde(rename = "Variables")]
    pub variables: XmlVariables,
    #[serde(rename = "Relationships")]
    pub relationships: XmlRelationships,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlLayout {
    #[serde(rename = "Columns", deserialize_with = "deser_quoted_int")]
    pub columns: u32,
    #[serde(rename = "Rows", deserialize_with = "deser_quoted_int")]
    pub rows: u32,
    #[serde(rename = "ZoomLevel")]
    pub zoom_level: f32,
    #[serde(rename = "PanX")]
    pub pan_x: i32,
    #[serde(rename = "PanY")]
    pub pan_y: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlContainers {
    #[serde(rename = "Container")]
    pub container: Vec<XmlContainer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlContainer {
    #[serde(rename = "Id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "PositionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY")]
    pub position_y: f64,
    #[serde(rename = "Size", deserialize_with = "deser_quoted_int")]
    pub size: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlVariables {
    #[serde(rename = "Variable")]
    pub variable: Vec<XmlVariable>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlVariable {
    #[serde(rename = "Id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ContainerId", deserialize_with = "deser_quoted_int")]
    pub container_id: u32,
    #[serde(rename = "Type")]
    pub r#type: VariableType,
    #[serde(rename = "RangeFrom", deserialize_with = "deser_quoted_int")]
    pub range_from: u32,
    #[serde(rename = "RangeTo", deserialize_with = "deser_quoted_int")]
    pub range_to: u32,
    #[serde(rename = "Formula")]
    pub formula: String,
    #[serde(rename = "PositionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY")]
    pub position_y: f64,
    #[serde(rename = "CellX", deserialize_with = "deser_quoted_int")]
    pub cell_x: u32,
    #[serde(rename = "CellY", deserialize_with = "deser_quoted_int")]
    pub cell_y: u32,
    #[serde(rename = "Angle")]
    pub angle: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlRelationships {
    #[serde(rename = "Relationship")]
    pub relationship: Vec<XmlRelationship>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlRelationship {
    #[serde(rename = "Id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(rename = "ContainerId", deserialize_with = "deser_quoted_int")]
    pub container_id: u32,
    #[serde(rename = "FromVariableId", deserialize_with = "deser_quoted_int")]
    pub from_variable_id: u32,
    #[serde(rename = "ToVariableId", deserialize_with = "deser_quoted_int")]
    pub to_variable_id: u32,
    #[serde(rename = "Type")]
    pub r#type: RelationshipType,
}
