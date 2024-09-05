use crate::enums::{RelationshipType, VariableType};
use crate::update_fn::bma_fn_tree::BmaFnNode;
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
    pub formula: BmaFnNode,
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
    pub id: u32,
    pub name: Option<String>, // Optional, as some containers may not have a name
    pub size: u32,
    pub position_x: f64,
    pub position_y: f64,
}
