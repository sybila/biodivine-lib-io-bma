use crate::BmaLayoutContainer;
use crate::serde::quote_num::QuoteNum;
use crate::utils::{f64_or_default, rational_or_default};
use serde::{Deserialize, Serialize};

/// Structure to deserialize JSON info about layout container.
///
/// All details must be provided, except for the name.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonLayoutContainer {
    #[serde(rename = "Id", alias = "id")]
    pub id: QuoteNum,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "Size", alias = "size")]
    pub size: QuoteNum,
    #[serde(rename = "PositionX", alias = "positionX")]
    pub position_x: f64,
    #[serde(rename = "PositionY", alias = "positionY")]
    pub position_y: f64,
}

impl From<BmaLayoutContainer> for JsonLayoutContainer {
    fn from(value: BmaLayoutContainer) -> Self {
        JsonLayoutContainer {
            id: value.id.into(),
            name: value.name.clone(),
            size: value.size.into(),
            position_x: f64_or_default(value.position.0),
            position_y: f64_or_default(value.position.1),
        }
    }
}

impl From<JsonLayoutContainer> for BmaLayoutContainer {
    fn from(value: JsonLayoutContainer) -> Self {
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
