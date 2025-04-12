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
    pub model: BmaNetwork,
    /// Layout information (variable positions, containers, ...).
    pub layout: BmaLayout,
    /// Stores additional metadata like biocheck_version that are sometimes present in XML.
    #[serde(flatten)]
    pub metadata: HashMap<String, String>,
}

/// Named model with several `variables` that have various `relationships`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaNetwork {
    pub name: String,
    pub variables: Vec<BmaVariable>,
    pub relationships: Vec<BmaRelationship>,
}

/// A discrete variable with ID and name, range of possible values, and an update expression
/// that dictates how the variable evolves.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaVariable {
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
pub struct BmaRelationship {
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
pub struct BmaLayout {
    pub variables: Vec<BmaLayoutVariable>,
    pub containers: Vec<BmaContainer>,
    pub description: String, // can be empty (by default if not provided)
    pub zoom_level: Option<f32>,
    pub pan_x: Option<i32>,
    pub pan_y: Option<i32>,
}

/// A layout information regarding a model variable.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaLayoutVariable {
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
pub struct BmaContainer {
    pub id: u32,
    pub name: String, // can be empty if not provided
    pub size: u32,
    pub position_x: f64,
    pub position_y: f64,
}

impl BmaLayoutVariable {
    /// Create a default layout for a variable with given name and ID.
    /// Default position is (0, 0), angle is 0.0, and cell/description is empty.
    pub fn new_default(id: u32, name: String) -> Self {
        BmaLayoutVariable {
            id,
            name,
            variable_type: VariableType::Default,
            container_id: 0,
            position_x: 0.0,
            position_y: 0.0,
            cell_x: None,
            cell_y: None,
            angle: 0.0,
            description: "".to_string(),
        }
    }
}

impl BmaContainer {
    /// Create a default empty container. Default position is (0, 0), and size is 1.
    pub fn new_default(id: u32, name: String) -> Self {
        BmaContainer {
            id,
            name,
            size: 1,
            position_x: 0.0,
            position_y: 0.0,
        }
    }
}

impl Default for BmaLayout {
    /// Create a default empty layout with no variables or containers.
    fn default() -> Self {
        BmaLayout {
            variables: Vec::new(),
            containers: Vec::new(),
            description: String::default(),
            zoom_level: None,
            pan_x: None,
            pan_y: None,
        }
    }
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
