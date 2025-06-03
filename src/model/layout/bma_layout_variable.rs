use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Additional layout information regarding a model variable.
///
/// If some information is not provided, it can be set to default values (like
/// position and angle values 0, default type, empty description, ...).
/// Other missing information is set to `None` (like cell or container ID).
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BmaLayoutVariable {
    pub id: u32,
    pub container_id: Option<u32>,
    pub r#type: VariableType,        // Corresponds to "Type" in JSON/XML
    pub name: Option<String>,        // duplicated with Variable.name, but that's what JSON BMA does
    pub description: Option<String>, // can be empty (by default if not provided)
    pub position: (f64, f64),
    pub angle: f64,
    pub cell: Option<(u32, u32)>,
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
            container_id,
            name: Some(name),
            ..Default::default()
        }
    }

    /// Clone the variable name or create a default alternative (`v_ID`).
    pub fn name_or_default(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("v_{}", self.id))
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq)]
pub enum VariableType {
    #[default]
    Default,
    Constant,
    MembraneReceptor,
}
