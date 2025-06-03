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
