use crate::enums::{RelationshipType, VariableType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize)]
#[serde(untagged)]
enum StrOrNum<'a> {
    Str(&'a str),
    Num(u32),
}

/// Custom deserializer function for (potentially) quoted integers (like "42").
///
/// For some reason, this is JSON-specific, and we have to use different variant for XMP.
pub fn deser_quoted_int<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = StrOrNum::deserialize(deserializer)?;

    match raw {
        StrOrNum::Str(s) => {
            let trimmed = s.trim_matches('"');
            u32::from_str(trimmed).map_err(serde::de::Error::custom)
        }
        StrOrNum::Num(num) => Ok(num),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonBmaModel {
    #[serde(rename = "Model", alias = "model")]
    pub model: JsonModel,
    #[serde(rename = "Layout", alias = "layout")]
    pub layout: Option<JsonLayout>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonModel {
    #[serde(rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "Variables", alias = "variables")]
    pub variables: Vec<JsonVariable>,
    #[serde(rename = "Relationships", alias = "relationships")]
    pub relationships: Vec<JsonRelationship>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonVariable {
    #[serde(rename = "Id", alias = "id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(rename = "Id", alias = "id")]
    pub name: Option<String>,
    #[serde(
        rename = "RangeFrom",
        alias = "rangeFrom",
        deserialize_with = "deser_quoted_int"
    )]
    pub range_from: u32,
    #[serde(
        rename = "RangeTo",
        alias = "rangeTo",
        deserialize_with = "deser_quoted_int"
    )]
    pub range_to: u32,
    #[serde(rename = "Formula", alias = "formula")]
    pub formula: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonRelationship {
    #[serde(rename = "Id", alias = "id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(
        rename = "FromVariable",
        alias = "fromVariable",
        alias = "fromVariableId",
        alias = "FromVariableId",
        deserialize_with = "deser_quoted_int"
    )]
    pub from_variable: u32,
    #[serde(
        rename = "ToVariable",
        alias = "toVariable",
        alias = "toVariableId",
        alias = "ToVariableId",
        deserialize_with = "deser_quoted_int"
    )]
    pub to_variable: u32,
    #[serde(rename = "Type", alias = "type")]
    pub r#type: RelationshipType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonLayout {
    #[serde(rename = "Variables", alias = "variables")]
    pub variables: Vec<JsonLayoutVariable>,
    #[serde(rename = "Containers", alias = "containers")]
    pub containers: Vec<JsonContainer>,
    #[serde(rename = "Description", alias = "description")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonLayoutVariable {
    #[serde(rename = "Id", alias = "id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(rename = "Type", alias = "type")]
    pub r#type: VariableType,
    #[serde(
        rename = "ContainerId",
        alias = "containerId",
        deserialize_with = "deser_quoted_int"
    )]
    pub container_id: u32,
    #[serde(rename = "PositionX", alias = "positionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY", alias = "positionY")]
    pub position_y: f64,
    #[serde(rename = "CellX", alias = "cellX")]
    pub cell_x: Option<u32>,
    #[serde(rename = "CellY", alias = "cellY")]
    pub cell_y: Option<u32>,
    #[serde(rename = "Angle", alias = "angle")]
    pub angle: f64,
    #[serde(rename = "Description", alias = "description")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonContainer {
    #[serde(rename = "Id", alias = "id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(rename = "Size", alias = "size", deserialize_with = "deser_quoted_int")]
    pub size: u32,
    #[serde(rename = "PositionX", alias = "positionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY", alias = "positionY")]
    pub position_y: f64,
}
