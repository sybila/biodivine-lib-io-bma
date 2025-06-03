use crate::enums::VariableType;
use crate::model::{BmaLayout, BmaNetwork};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

/// Main structure with all the important parts of BMA model.
/// We distinguish between three parts tracked in the BMA format:
/// - the functional part with all the variables and relationships (`model`)
/// - the optional layout with positions of variables and containers (`layout`)
/// - the additional optional data like version and so on (`metadata`)
///
/// `BmaModel` instances can be created from JSON or XML versions of the BMA format.
/// You can use `from_json_str`, `from_xml_str` to create a model from a string.
/// For serialization to JSON, use custom methods `to_json_str`, or `to_pretty_json_str`.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaModel {
    /// Main data with variables and relationships.
    pub model: BmaNetwork,
    /// Layout information (variable positions, containers, ...).
    /// Laout can be empty, but it is recommended to provide it.
    pub layout: BmaLayout,
    /// Stores additional metadata like biocheck_version that are sometimes present in XML.
    /// Metadata are usually empty.
    #[serde(flatten)]
    pub metadata: HashMap<String, String>,
}

/// Additional layout information regarding a model variable.
///
/// If some information is not provided, it cab be set to default values (like
/// position and angle values 0, default type, empty description, ...).
/// Other missing information is set to `None` (like cell or container ID).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaLayoutVariable {
    pub id: u32,
    pub name: String, // duplicated with Variable.name, but that's what JSON BMA does
    #[serde(rename = "Type")]
    pub variable_type: VariableType, // Corresponds to "Type" in JSON/XML
    pub position_x: f64,
    pub position_y: f64,
    pub angle: f64,
    pub description: String, // can be empty (by default if not provided)
    pub container_id: Option<u32>,
    pub cell_x: Option<u32>,
    pub cell_y: Option<u32>,
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
    /// Container ID is optional, and can be set to `None`.
    ///
    /// Default position is (0, 0), angle is 0.0, and cell/description is empty.
    /// Cell values are set to `None`.
    pub fn new_default(id: u32, name: String, container_id: Option<u32>) -> Self {
        BmaLayoutVariable {
            id,
            name,
            variable_type: VariableType::Default,
            position_x: 0.0,
            position_y: 0.0,
            angle: 0.0,
            container_id,
            cell_x: None,
            cell_y: None,
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
