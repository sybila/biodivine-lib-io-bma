use crate::{BmaLayout, BmaNetwork};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

/// Main structure with all the important parts of a BMA model.
/// We distinguish between three parts tracked in the BMA format:
/// - the functional part with all the variables and relationships (`model`)
/// - the optional layout with positions of variables and containers (`layout`)
/// - the additional optional data like a version and so on (`metadata`)
///
/// `BmaModel` instances can be created from JSON or XML versions of the BMA format.
/// You can use `from_json_str`, `from_xml_str` to create a model from a string.
/// For serialization to JSON, use custom methods `to_json_str`, or `to_pretty_json_str`.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BmaModel {
    /// Main data with variables and relationships.
    pub model: BmaNetwork,
    /// Layout information (variable positions, containers, ...).
    /// Layout can be empty, but it is recommended to provide it.
    pub layout: BmaLayout,
    /// Stores additional metadata like `biocheck_version` that are sometimes present in XML.
    /// Metadata is usually empty.
    #[serde(flatten)]
    pub metadata: HashMap<String, String>,
}
