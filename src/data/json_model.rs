use crate::RelationshipType;
use crate::VariableType;
use crate::data::quote_num::QuoteNum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(rename = "Id", alias = "id")]
    pub id: QuoteNum,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "RangeFrom", alias = "rangeFrom")]
    pub range_from: QuoteNum,
    #[serde(rename = "RangeTo", alias = "rangeTo")]
    pub range_to: QuoteNum,
    #[serde(rename = "Formula", alias = "formula")]
    pub formula: String,
}

/// Structure to deserialize JSON info about an individual relationship.
///
/// All relationships must have its own ID, type, and IDs of both interacting
/// variables.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonRelationship {
    #[serde(rename = "Id", alias = "id")]
    pub id: QuoteNum,
    #[serde(
        rename = "FromVariable",
        alias = "fromVariable",
        alias = "fromVariableId",
        alias = "FromVariableId"
    )]
    pub from_variable: QuoteNum,
    #[serde(
        rename = "ToVariable",
        alias = "toVariable",
        alias = "toVariableId",
        alias = "ToVariableId"
    )]
    pub to_variable: QuoteNum,
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
    #[serde(rename = "Id", alias = "id")]
    pub id: QuoteNum,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(default, rename = "Type", alias = "type")]
    pub r#type: VariableType,
    #[serde(default, rename = "PositionX", alias = "positionX")]
    pub position_x: f64,
    #[serde(default, rename = "PositionY", alias = "positionY")]
    pub position_y: f64,
    #[serde(default, rename = "Angle", alias = "angle")]
    pub angle: f64,
    #[serde(default, rename = "Description", alias = "description")]
    pub description: String,
    #[serde(rename = "ContainerId", alias = "containerId", default)]
    pub container_id: Option<QuoteNum>,
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
    #[serde(rename = "Id", alias = "id")]
    pub id: QuoteNum,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "Size", alias = "size")]
    pub size: QuoteNum,
    #[serde(rename = "PositionX", alias = "positionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY", alias = "positionY")]
    pub position_y: f64,
}

impl JsonBmaModel {
    /// Collects set of all variables in the model, creating ID-name mapping.
    /// First collects all variables from the model. For those that have empty
    /// names, it tries to find a name in the layout.
    pub fn collect_all_variables(&self) -> HashMap<u32, String> {
        let mut model_vars = self
            .model
            .variables
            .iter()
            .map(|var| (var.id.into(), var.name.clone()))
            .collect::<HashMap<u32, String>>();

        let layout_named_vars = self.collect_named_layout_variables();
        for (id, name_in_layout) in layout_named_vars {
            if let Some(name_in_model) = model_vars.get(&id) {
                if name_in_model.is_empty() {
                    model_vars.insert(id, name_in_layout);
                }
            }
        }

        model_vars
    }

    /// Collects set of all named variables from the layout, creating ID-name mapping.
    /// Variables without names (i.e., with empty name string) are ignored.
    pub fn collect_named_layout_variables(&self) -> HashMap<u32, String> {
        match &self.layout {
            None => HashMap::new(),
            Some(layout) => layout
                .variables
                .iter()
                .filter(|layout_var| !layout_var.name.is_empty())
                .map(|layout_var| (layout_var.id.into(), layout_var.name.clone()))
                .collect(),
        }
    }

    /// Collects set of variables that regulate given variable.
    pub fn get_regulators(&self, variable_id: u32) -> Vec<u32> {
        self.model
            .relationships
            .iter()
            .filter(|rel| u32::from(rel.to_variable) == variable_id)
            .map(|rel| rel.from_variable.into())
            .collect()
    }
}
