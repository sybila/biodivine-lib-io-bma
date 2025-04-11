use crate::enums::{RelationshipType, VariableType};
use crate::update_fn::bma_fn_tree::BmaFnUpdate;
use serde::{Deserialize, Serialize, Serializer};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

/// Main structure with all the important parts of BMA model.
/// We distinguish between three parts tracked in the BMA format
/// - the functional part with all the variables and relationships (`model`)
/// - the layout part positions of variables and containers (`layout`)
/// - the additional (optional) data like version and so on (`metadata`)
///
/// `BmaModel` instances can be parsed from JSON or XML versions of the BMA format.
/// You can use `from_json_str`, `from_xml_str` to create a model from a string.
/// For serialization to JSON, use custom methods `to_json_str`, or `to_pretty_json_str`.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaModel {
    /// Main data with variables and relationships.
    pub model: Model,
    /// Layout information (variable positions, containers, ...).
    pub layout: Layout,
    /// Stores additional metadata like biocheck_version that are sometimes present in XML.
    #[serde(flatten)]
    pub metadata: HashMap<String, String>,
}

/// Named model with several `variables` that have various `relationships`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Model {
    pub name: String,
    pub variables: Vec<Variable>,
    pub relationships: Vec<Relationship>,
}

/// A discrete variable with ID and name, range of possible values, and an update expression
/// that dictates how the variable evolves.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Variable {
    pub id: u32,
    pub name: String,
    pub range_from: u32,
    pub range_to: u32,
    #[serde(serialize_with = "serialize_update_fn")]
    pub formula: Option<BmaFnUpdate>,
}

/// A relationship of a given type between two variables.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Relationship {
    pub id: u32,
    pub from_variable: u32,
    pub to_variable: u32,
    #[serde(rename = "Type")]
    pub relationship_type: RelationshipType, // Corresponds to "Type" in JSON/XML
}

/// A layout describing positions and types of variables and containers.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Layout {
    pub variables: Vec<LayoutVariable>,
    pub containers: Vec<Container>,
    pub description: String, // can be empty (by default if not provided)
    pub zoom_level: Option<f32>,
    pub pan_x: Option<i32>,
    pub pan_y: Option<i32>,
}

/// A layout information regarding a model variable.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LayoutVariable {
    pub id: u32,
    pub name: String, // duplicated with Variable.name, but that's what BMA does
    #[serde(rename = "Type")]
    pub variable_type: VariableType, // Corresponds to "Type" in JSON/XML
    pub container_id: u32,
    pub position_x: f64,
    pub position_y: f64,
    pub cell_x: Option<u32>, // this can be serialized to null
    pub cell_y: Option<u32>, // this can be serialized to null
    pub angle: f64,
    pub description: String, // can be empty (by default if not provided)
}

/// A layout information about a container.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Container {
    pub id: u32,
    pub name: String, // can be empty if not provided
    pub size: u32,
    pub position_x: f64,
    pub position_y: f64,
}

/// A utility to serialize update function by calling a custom parser.
fn serialize_update_fn<S>(update_fn: &Option<BmaFnUpdate>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(update_fn_str) = update_fn {
        s.serialize_str(update_fn_str.as_str())
    } else {
        s.serialize_str("")
    }
}
