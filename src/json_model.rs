use crate::enums::{RelationshipType, VariableType};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Deserialize)]
#[serde(untagged)]
enum StrOrNum<'a> {
    Str(&'a str),
    Num(u32),
}

/// Custom deserializer function for (potentially) quoted integers (like "42").
///
/// For some reason, this is JSON-specific, and we have to use different variant for XML.
fn deser_quoted_int<'de, D>(deserializer: D) -> Result<u32, D::Error>
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

/// Custom deserializer function for optional (potentially) quoted integers (like "42").
///
/// For some reason, this is JSON-specific, and we have to use different variant for XML.
fn deser_quoted_int_optional<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = StrOrNum::deserialize(deserializer)?;

    match raw {
        StrOrNum::Str(s) => {
            let trimmed = s.trim_matches('"');
            Ok(Some(
                u32::from_str(trimmed).map_err(serde::de::Error::custom)?,
            ))
        }
        StrOrNum::Num(num) => Ok(Some(num)),
    }
}

fn default_position_val() -> f64 {
    0.0
}

fn default_angle_val() -> f64 {
    0.0
}

/// An intermediate structure purely for deserializing JSON BMA models.
///
/// The functional part of the model is stored in `model` field. The additional `layout`
/// information is optional.
///
/// This structure is intended purely to simplify serialization. It does not provide much of a
/// consistency checking. The serialized instances may contain semantically invalid data, such as
/// incorrectly formatted update functions, or variables not matching in layout and model.
/// The full correctness of the model is checked when constructing the final `BmaModel` struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonBmaModel {
    #[serde(rename = "Model", alias = "model")]
    pub model: JsonModel,
    #[serde(rename = "Layout", alias = "layout")]
    pub layout: Option<JsonLayout>,
}

/// Structure to deserialize JSON info about the main model component, with several
/// `variables` that have various `relationships`.
///
/// Variables and relationships are required. The name is optional, and default
/// empty string is used if not provided.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonModel {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "Variables", alias = "variables")]
    pub variables: Vec<JsonVariable>,
    #[serde(rename = "Relationships", alias = "relationships")]
    pub relationships: Vec<JsonRelationship>,
}

/// Structure to deserialize JSON info about individual variable.
///
/// All variables must have ID, range of possible values, and an update formula.
/// The formula can be empty string.
/// Name is optional and set to None is not provided.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonVariable {
    #[serde(rename = "Id", alias = "id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
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

/// Structure to deserialize JSON info about an individual relationship.
///
/// All relationships must have its own ID, type, and IDs of both interacting
/// variables.
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
    pub containers: Vec<JsonContainer>,
    #[serde(default, rename = "Description", alias = "description")]
    pub description: String,
}

/// Structure to deserialize JSON info about variable's layout information.
///
/// We require ID and position to be present in the JSON.
/// If name and description are not provided, we set them to empty strings.
/// If type and angle are not provided, we set it to default values.
/// Container ID and cell coordinates are optional, and set to None if not provided.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonLayoutVariable {
    #[serde(rename = "Id", alias = "id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(default, rename = "Type", alias = "type")]
    pub r#type: VariableType,
    #[serde(
        default = "default_position_val",
        rename = "PositionX",
        alias = "positionX"
    )]
    pub position_x: f64,
    #[serde(
        default = "default_position_val",
        rename = "PositionY",
        alias = "positionY"
    )]
    pub position_y: f64,
    #[serde(default = "default_angle_val", rename = "Angle", alias = "angle")]
    pub angle: f64,
    #[serde(default, rename = "Description", alias = "description")]
    pub description: String,
    #[serde(
        rename = "ContainerId",
        alias = "containerId",
        default,
        deserialize_with = "deser_quoted_int_optional"
    )]
    pub container_id: Option<u32>,
    #[serde(rename = "CellX", alias = "cellX")]
    pub cell_x: Option<u32>,
    #[serde(rename = "CellY", alias = "cellY")]
    pub cell_y: Option<u32>,
}

/// Structure to deserialize JSON info about layout container.
///
/// All details must be provided, except for the name. If name is missing,
/// we set it to an empty string.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonContainer {
    #[serde(rename = "Id", alias = "id", deserialize_with = "deser_quoted_int")]
    pub id: u32,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "Size", alias = "size", deserialize_with = "deser_quoted_int")]
    pub size: u32,
    #[serde(rename = "PositionX", alias = "positionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY", alias = "positionY")]
    pub position_y: f64,
}

impl JsonBmaModel {
    /// Collects set of all named variables from the layout, creating ID-name mapping.
    /// Variables without names (i.e., with empty name string) are ignored.
    pub fn collect_named_layout_variables(&self) -> HashMap<u32, String> {
        match &self.layout {
            None => HashMap::new(),
            Some(layout) => layout
                .variables
                .iter()
                .filter(|layout_var| !layout_var.name.is_empty())
                .map(|layout_var| (layout_var.id, layout_var.name.clone()))
                .collect(),
        }
    }
}
