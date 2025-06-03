use crate::enums::VariableType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Additional layout information regarding a model variable.
///
/// If some information is not provided, it can be set to default values (like
/// position and angle values 0, default type, empty description, ...).
/// Other missing information is set to `None` (like cell or container ID).
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl BmaLayoutVariable {
    /// Create a default layout for a variable with a given name and ID.
    /// Container ID is optional and can be set to `None`.
    ///
    /// The default position is `(0, 0)`, the angle is `0.0`, and the cell / description is empty.
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
