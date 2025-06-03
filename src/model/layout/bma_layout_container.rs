use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A layout information about a container.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaLayoutContainer {
    pub id: u32,
    pub name: String, // can be empty if not provided
    pub size: u32,
    pub position_x: f64,
    pub position_y: f64,
}

impl BmaLayoutContainer {
    /// Create a default empty container. Default position is (0, 0), and size is 1.
    pub fn new_default(id: u32, name: String) -> Self {
        BmaLayoutContainer {
            id,
            name,
            size: 1,
            position_x: 0.0,
            position_y: 0.0,
        }
    }
}
