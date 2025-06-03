use crate::model::BmaLayoutContainer;
use crate::model::BmaLayoutVariable;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A layout describing positions and types of variables and containers.
/// Most fields are optional, as the layout contains mostly complementary information.
///
/// Set of variables here should be a subset of the variables in the model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaLayout {
    pub variables: Vec<BmaLayoutVariable>,
    pub containers: Vec<BmaLayoutContainer>,
    pub description: String, // can be empty (by default if not provided)
    pub zoom_level: Option<f64>,
    pub pan_x: Option<f64>,
    pub pan_y: Option<f64>,
}

impl Default for BmaLayout {
    /// Create a default empty layout with no variables or containers.
    fn default() -> Self {
        BmaLayout {
            variables: Vec::new(),
            containers: Vec::new(),
            description: String::default(),
            zoom_level: None,
            pan_x: None,
            pan_y: None,
        }
    }
}
