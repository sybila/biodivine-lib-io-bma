use crate::enums::{RelationshipType, VariableType};
use crate::json_model::JsonBmaModel;
use crate::xml_model::XmlBmaModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BmaModel {
    /// Main data with variables and relationships.
    pub model: Model,
    /// Layout information (variable positions, containers, ...).
    pub layout: Layout,
    /// Stores additional metadata like description, biocheck_version, etc.
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Model {
    pub name: String,
    pub variables: Vec<Variable>,
    pub relationships: Vec<Relationship>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variable {
    pub id: u32,
    pub name: String,
    pub variable_type: VariableType, // Corresponds to "Type" in JSON/XML
    pub range_from: u32,
    pub range_to: u32,
    pub formula: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    pub id: u32,
    pub from_variable: u32,
    pub to_variable: u32,
    pub relationship_type: RelationshipType, // Corresponds to "Type" in JSON/XML
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Layout {
    pub variables: Vec<LayoutVariable>,
    pub containers: Vec<Container>,
    pub zoom_level: Option<f32>,
    pub pan_x: Option<i32>,
    pub pan_y: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutVariable {
    pub id: u32,
    pub container_id: u32,
    pub position_x: f64,
    pub position_y: f64,
    pub cell_x: Option<u32>,
    pub cell_y: Option<u32>,
    pub angle: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Container {
    id: u32,
    pub name: Option<String>, // Optional, as some containers may not have a name
    pub size: u32,
    pub position_x: f64,
    pub position_y: f64,
}

impl From<JsonBmaModel> for BmaModel {
    fn from(json_model: JsonBmaModel) -> Self {
        // Create a mapping from variable IDs to their names from the layout
        let layout_var_names: HashMap<u32, String> = json_model
            .layout
            .as_ref()
            .map(|layout| {
                layout
                    .variables
                    .iter()
                    .filter(|layout_var| layout_var.name.is_some())
                    .map(|layout_var| (layout_var.id, layout_var.name.clone().unwrap()))
                    .collect()
            })
            .unwrap_or_default();

        // Convert the model
        let model = Model {
            name: json_model.model.name,
            variables: json_model
                .model
                .variables
                .into_iter()
                .map(|var| Variable {
                    id: var.id,
                    name: var
                        .name
                        .unwrap_or(layout_var_names.get(&var.id).cloned().unwrap_or_default()), // Use the name from layout
                    variable_type: json_model
                        .layout
                        .as_ref()
                        .and_then(|layout| layout.variables.iter().find(|v| v.id == var.id))
                        .map(|v| v.r#type)
                        .unwrap_or(VariableType::Default), // Use the type from layout if available
                    range_from: var.range_from,
                    range_to: var.range_to,
                    formula: var.formula,
                })
                .collect(),
            relationships: json_model
                .model
                .relationships
                .into_iter()
                .map(|rel| Relationship {
                    id: rel.id,
                    from_variable: rel.from_variable,
                    to_variable: rel.to_variable,
                    relationship_type: rel.r#type,
                })
                .collect(),
        };

        // Convert the layout
        let layout = json_model
            .layout
            .map(|layout| Layout {
                variables: layout
                    .variables
                    .into_iter()
                    .map(|var| LayoutVariable {
                        id: var.id,
                        container_id: var.container_id,
                        position_x: var.position_x,
                        position_y: var.position_y,
                        cell_x: var.cell_x,
                        cell_y: var.cell_y,
                        angle: var.angle,
                    })
                    .collect(),
                containers: layout
                    .containers
                    .into_iter()
                    .map(|container| Container {
                        id: container.id,
                        name: container.name,
                        size: container.size,
                        position_x: container.position_x,
                        position_y: container.position_y,
                    })
                    .collect(),
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            })
            .unwrap_or_else(|| Layout {
                variables: vec![],
                containers: vec![],
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            });

        // metadata not present in JsonBmaModel
        let metadata = HashMap::new();

        BmaModel {
            model,
            layout,
            metadata,
        }
    }
}

impl From<XmlBmaModel> for BmaModel {
    fn from(xml_model: XmlBmaModel) -> Self {
        // Convert the model
        let model = Model {
            name: xml_model.name,
            variables: xml_model
                .variables
                .variable
                .clone()
                .into_iter()
                .map(|var| Variable {
                    id: var.id,
                    name: var.name,
                    variable_type: var.r#type,
                    range_from: var.range_from,
                    range_to: var.range_to,
                    formula: var.formula,
                })
                .collect(),
            relationships: xml_model
                .relationships
                .relationship
                .into_iter()
                .map(|rel| Relationship {
                    id: rel.id,
                    from_variable: rel.from_variable_id,
                    to_variable: rel.to_variable_id,
                    relationship_type: rel.r#type,
                })
                .collect(),
        };

        // Convert the layout
        let layout = Layout {
            variables: xml_model
                .variables
                .variable
                .into_iter()
                .map(|var| LayoutVariable {
                    id: var.id,
                    container_id: var.container_id,
                    position_x: var.position_x,
                    position_y: var.position_y,
                    cell_x: Some(var.cell_x),
                    cell_y: Some(var.cell_y),
                    angle: var.angle,
                })
                .collect(),
            containers: xml_model
                .containers
                .container
                .into_iter()
                .map(|container| Container {
                    id: container.id,
                    name: Some(container.name),
                    size: container.size,
                    position_x: container.position_x,
                    position_y: container.position_y,
                })
                .collect(),
            zoom_level: Some(xml_model.layout.zoom_level),
            pan_x: Some(xml_model.layout.pan_x),
            pan_y: Some(xml_model.layout.pan_y),
        };

        // Metadata can be constructed from various XML fields
        let mut metadata = HashMap::new();
        metadata.insert("biocheck_version".to_string(), xml_model.biocheck_version);
        metadata.insert("description".to_string(), xml_model.description);
        metadata.insert("created_date".to_string(), xml_model.created_date);
        metadata.insert("modified_date".to_string(), xml_model.modified_date);

        BmaModel {
            model,
            layout,
            metadata,
        }
    }
}
