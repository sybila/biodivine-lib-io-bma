use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Model")]
struct Model {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "BioCheckVersion")]
    biocheck_version: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "CreatedDate")]
    created_date: String,
    #[serde(rename = "ModifiedDate")]
    modified_date: String,
    #[serde(rename = "Layout")]
    layout: Layout,
    #[serde(rename = "Containers")]
    containers: Containers,
    #[serde(rename = "Variables")]
    variables: Variables,
    #[serde(rename = "Relationships")]
    relationships: Relationships,
}

#[derive(Serialize, Deserialize, Debug)]
struct Layout {
    #[serde(rename = "Columns")]
    columns: u32,
    #[serde(rename = "Rows")]
    rows: u32,
    #[serde(rename = "ZoomLevel")]
    zoom_level: u32,
    #[serde(rename = "PanX")]
    pan_x: i32,
    #[serde(rename = "PanY")]
    pan_y: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Containers {
    #[serde(rename = "Container")]
    container: Vec<Container>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Container {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "PositionX")]
    position_x: f64,
    #[serde(rename = "PositionY")]
    position_y: f64,
    #[serde(rename = "Size")]
    size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Variables {
    #[serde(rename = "Variable")]
    variable: Vec<Variable>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Variable {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "ContainerId")]
    container_id: u32,
    #[serde(rename = "Type")]
    r#type: String,
    #[serde(rename = "RangeFrom")]
    range_from: f64,
    #[serde(rename = "RangeTo")]
    range_to: f64,
    #[serde(rename = "Formula")]
    formula: String,
    #[serde(rename = "PositionX")]
    position_x: f64,
    #[serde(rename = "PositionY")]
    position_y: f64,
    #[serde(rename = "CellX")]
    cell_x: u32,
    #[serde(rename = "CellY")]
    cell_y: u32,
    #[serde(rename = "Angle")]
    angle: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Relationships {
    #[serde(rename = "Relationship")]
    relationship: Vec<Relationship>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Relationship {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "ContainerId")]
    container_id: u32,
    #[serde(rename = "FromVariableId")]
    from_variable_id: u32,
    #[serde(rename = "ToVariableId")]
    to_variable_id: u32,
    #[serde(rename = "Type")]
    r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct XmlBmaModel {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "BioCheckVersion")]
    biocheck_version: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "CreatedDate")]
    created_date: String,
    #[serde(rename = "ModifiedDate")]
    modified_date: String,
    #[serde(rename = "Layout")]
    layout: Layout,
    #[serde(rename = "Containers")]
    containers: Containers,
    #[serde(rename = "Variables")]
    variables: Variables,
    #[serde(rename = "Relationships")]
    relationships: Relationships,
}
