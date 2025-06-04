use crate::BmaLayoutContainer;
use crate::data::quote_num::QuoteNum;
use crate::utils::take_if_not_blank;
use num_rational::Rational64;
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};

/// Structure to deserialize JSON info about layout container.
///
/// All details must be provided, except for the name.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonContainer {
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

impl From<BmaLayoutContainer> for JsonContainer {
    fn from(value: BmaLayoutContainer) -> Self {
        JsonContainer {
            id: value.id.into(),
            name: value.name.unwrap_or_default(),
            size: value.size.into(),
            position_x: value.position.0.to_f64().unwrap_or_default(),
            position_y: value.position.1.to_f64().unwrap_or_default(),
        }
    }
}

impl From<JsonContainer> for BmaLayoutContainer {
    fn from(value: JsonContainer) -> Self {
        BmaLayoutContainer {
            id: value.id.into(),
            name: take_if_not_blank(value.name.as_str()),
            size: value.size.into(),
            position: (
                Rational64::from_f64(value.position_x).unwrap_or_default(),
                Rational64::from_f64(value.position_y).unwrap_or_default(),
            ),
        }
    }
}
