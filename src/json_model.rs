use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Model {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Variables")]
    variables: Vec<Variable>,
    #[serde(rename = "Relationships")]
    relationships: Vec<Relationship>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Variable {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "RangeFrom")]
    range_from: f64,
    #[serde(rename = "RangeTo")]
    range_to: f64,
    #[serde(rename = "Formula")]
    formula: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Relationship {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "FromVariable")]
    from_variable: u32,
    #[serde(rename = "ToVariable")]
    to_variable: u32,
    #[serde(rename = "Type")]
    r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Layout {
    #[serde(rename = "Variables")]
    variables: Vec<LayoutVariable>,
    #[serde(rename = "Containers")]
    containers: Vec<Container>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LayoutVariable {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Type")]
    r#type: String,
    #[serde(rename = "ContainerId")]
    container_id: u32,
    #[serde(rename = "PositionX")]
    position_x: f64,
    #[serde(rename = "PositionY")]
    position_y: f64,
    #[serde(rename = "CellX")]
    cell_x: Option<u32>,
    #[serde(rename = "CellY")]
    cell_y: Option<u32>,
    #[serde(rename = "Angle")]
    angle: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Container {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Size")]
    size: u32,
    #[serde(rename = "PositionX")]
    position_x: f64,
    #[serde(rename = "PositionY")]
    position_y: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonBmaModel {
    #[serde(rename = "Model")]
    model: Model,
    #[serde(rename = "Layout")]
    layout: Option<Layout>,
}
