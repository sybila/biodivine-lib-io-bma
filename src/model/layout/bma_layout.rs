use crate::{BmaLayoutContainer, BmaLayoutVariable};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A layout describing positions and types of variables and containers.
/// Most fields are optional, as the layout contains mostly complementary information.
///
/// Set of variables here should be a subset of the variables in the model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BmaLayout {
    pub variables: Vec<BmaLayoutVariable>,
    pub containers: Vec<BmaLayoutContainer>,
    pub description: String, // can be empty (by default if not provided)
    pub zoom_level: Option<f64>,
    pub pan_x: Option<f64>,
    pub pan_y: Option<f64>,
}
