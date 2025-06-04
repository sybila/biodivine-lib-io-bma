use crate::RelationshipType;
use crate::VariableType;
use crate::serde::quote_num::QuoteNum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An intermediate structure purely for deserializing XML BMA models.
///
/// We require only the functional parts of the model - variables and relationships.
/// Additional important strings (id, name, description) are set to empty if not provided.
/// Layout information and containers are optional.
/// We also parse some additional metadata items often present, but that is optional as well.
///
/// This structure is intended purely to simplify serialization. It does not provide much of a
/// consistency checking. The serialized instances may contain semantically invalid serde, such as
/// incorrectly formatted update functions, or variables not matching in layout and model.
/// The full correctness of the model is checked when constructing the final `BmaModel` struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Model")]
pub(crate) struct XmlBmaModel {
    #[serde(rename = "Variables")]
    pub variables: XmlVariables,
    #[serde(rename = "Relationships")]
    pub relationships: XmlRelationships,

    #[serde(default, rename = "@Id", alias = "Id")]
    pub id: String,
    #[serde(default, rename = "@Name", alias = "Name", alias = "@ModelName")]
    pub name: String,
    #[serde(default, rename = "Description")]
    pub description: String,
    #[serde(rename = "Layout")]
    pub layout: Option<XmlLayout>,
    #[serde(rename = "Containers")]
    pub containers: Option<XmlContainers>,

    #[serde(rename = "@BioCheckVersion", alias = "BioCheckVersion")]
    pub biocheck_version: Option<String>,
    #[serde(rename = "CreatedDate")]
    pub created_date: Option<String>,
    #[serde(rename = "ModifiedDate")]
    pub modified_date: Option<String>,
}

/// Structure to deserialize XML info about layout. This includes only a few
/// metadata items like zoom level and pan position. Info about variables and
/// containers is stored directly in model in BMA XML (as weird as it is...).
///
/// The zoom and pan values can be missing in the XML. If not provided, default
/// values are used.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlLayout {
    #[serde(rename = "Columns")]
    pub columns: QuoteNum,
    #[serde(rename = "Rows")]
    pub rows: QuoteNum,
    #[serde(default, rename = "ZoomLevel")]
    pub zoom_level: f64,
    #[serde(default, rename = "PanX")]
    pub pan_x: f64,
    #[serde(default, rename = "PanY")]
    pub pan_y: f64,
}

/// Structure to deserialize XML info about container list. Just a wrapper
/// for actual containers list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlContainers {
    #[serde(default, rename = "Container")]
    pub container: Vec<XmlContainer>,
}

/// Structure to deserialize XML info about container.
///
/// All details must be provided, except for the name. If name is missing,
/// we set it to an empty string.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlContainer {
    #[serde(rename = "@Id", alias = "Id")]
    pub id: QuoteNum,
    #[serde(default, rename = "@Name", alias = "Name")]
    pub name: String,
    #[serde(rename = "PositionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY")]
    pub position_y: f64,
    #[serde(rename = "Size")]
    pub size: QuoteNum,
}

/// Structure to deserialize XML info about variables list. Just a wrapper
/// for actual variables list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlVariables {
    #[serde(default, rename = "Variable")]
    pub variable: Vec<XmlVariable>,
}

/// Structure to deserialize XML info about variable. BMA XML format mixes
/// together functional and layout information for variables (unlike JSON),
/// which makes this a bit messsy.
///
/// All variables must have ID, range of possible values, and an update formula.
/// The formula can be empty string. If name is missing, we set it to an empty string.
/// If the type is missing, we set it to default value.
///
/// All other layout details are optional. If not provided, we set them to None here,
/// and some are set to default values later as needed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlVariable {
    #[serde(rename = "@Id", alias = "Id")]
    pub id: QuoteNum,
    #[serde(default, rename = "@Name", alias = "Name")]
    pub name: String,
    #[serde(rename = "RangeFrom")]
    pub range_from: QuoteNum,
    #[serde(rename = "RangeTo")]
    pub range_to: QuoteNum,
    #[serde(rename = "Formula", alias = "Function")]
    pub formula: String,

    #[serde(default, rename = "Type")]
    pub r#type: VariableType,
    #[serde(rename = "PositionX")]
    pub position_x: Option<f64>,
    #[serde(rename = "PositionY")]
    pub position_y: Option<f64>,
    #[serde(rename = "Angle")]
    pub angle: Option<f64>,
    #[serde(default, rename = "ContainerId")]
    pub container_id: Option<QuoteNum>,
    #[serde(default, rename = "CellX")]
    pub cell_x: Option<QuoteNum>,
    #[serde(default, rename = "CellY")]
    pub cell_y: Option<QuoteNum>,
}

/// Structure to deserialize XML info about relationships list. Just a wrapper
/// for actual relationships list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlRelationships {
    #[serde(default, rename = "Relationship")]
    pub relationship: Vec<XmlRelationship>,
}

/// Structure to deserialize JSON info about an individual relationship.
///
/// All relationships must have its own ID, type, and IDs of both interacting
/// variables.
///
/// The container ID is optional, and is set to None if not provided.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlRelationship {
    #[serde(rename = "@Id", alias = "Id")]
    pub id: QuoteNum,
    #[serde(rename = "FromVariableId")]
    pub from_variable_id: QuoteNum,
    #[serde(rename = "ToVariableId")]
    pub to_variable_id: QuoteNum,
    #[serde(rename = "Type")]
    pub r#type: RelationshipType,
    #[serde(default, rename = "ContainerId")]
    pub container_id: Option<QuoteNum>,
}

impl XmlBmaModel {
    /// Collects set of all variables in the model, creating ID-name mapping.
    pub fn collect_all_variables(&self) -> HashMap<u32, String> {
        self.variables
            .variable
            .iter()
            .map(|var| (var.id.into(), var.name.clone()))
            .collect::<HashMap<u32, String>>()
    }

    /// Collects set of variables that regulate given variable.
    pub fn get_regulators(&self, variable_id: u32) -> Vec<u32> {
        self.relationships
            .relationship
            .iter()
            .filter(|rel| u32::from(rel.to_variable_id) == variable_id)
            .map(|rel| rel.from_variable_id.into())
            .collect()
    }
}
