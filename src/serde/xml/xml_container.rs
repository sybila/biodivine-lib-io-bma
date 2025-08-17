use crate::BmaLayoutContainer;
use crate::utils::{f64_or_default, rational_or_default};
use serde::{Deserialize, Serialize};

/// Structure to deserialize XML info about container.
///
/// All details must be provided, except for the name. If the name is missing,
/// we set it to an empty string.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlContainer {
    #[serde(rename = "@Id", alias = "Id")]
    pub id: u32,
    #[serde(default, rename = "@Name", alias = "Name")]
    pub name: String,
    #[serde(rename = "PositionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY")]
    pub position_y: f64,
    #[serde(rename = "Size")]
    pub size: u32,
}

impl From<BmaLayoutContainer> for XmlContainer {
    fn from(value: BmaLayoutContainer) -> Self {
        XmlContainer {
            id: value.id.into(),
            name: value.name.clone(),
            position_x: f64_or_default(value.position.0),
            position_y: f64_or_default(value.position.1),
            size: value.size.into(),
        }
    }
}

impl From<XmlContainer> for BmaLayoutContainer {
    fn from(value: XmlContainer) -> Self {
        BmaLayoutContainer {
            id: value.id.into(),
            name: value.name.clone(),
            size: value.size.into(),
            position: (
                rational_or_default(value.position_x),
                rational_or_default(value.position_y),
            ),
        }
    }
}
