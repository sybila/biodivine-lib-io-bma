use crate::serde::quote_num::QuoteNum;
use crate::utils::{f64_or_default, rational_or_default};
use crate::{BmaLayoutVariable, VariableType};
use serde::{Deserialize, Serialize};

/// Structure to deserialize JSON info about variable's layout information.
///
/// We require ID and position to be present in the JSON.
/// If name and description are not provided, we set them to empty strings.
/// If type and angle are not provided, we set it to default values.
/// Container ID and cell coordinates are optional, and set to None if not provided.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonLayoutVariable {
    #[serde(rename = "Id", alias = "id")]
    pub id: QuoteNum,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(default, rename = "Type", alias = "type")]
    pub r#type: VariableType,
    #[serde(default, rename = "PositionX", alias = "positionX")]
    pub position_x: f64,
    #[serde(default, rename = "PositionY", alias = "positionY")]
    pub position_y: f64,
    #[serde(default, rename = "Angle", alias = "angle")]
    pub angle: f64,
    #[serde(default, rename = "Description", alias = "description")]
    pub description: String,
    #[serde(rename = "ContainerId", alias = "containerId", default)]
    pub container_id: Option<QuoteNum>,
    #[serde(rename = "CellX", alias = "cellX", default)]
    pub cell_x: Option<QuoteNum>,
    #[serde(rename = "CellY", alias = "cellY", default)]
    pub cell_y: Option<QuoteNum>,
}

impl From<JsonLayoutVariable> for BmaLayoutVariable {
    fn from(value: JsonLayoutVariable) -> Self {
        let cell = match (value.cell_x, value.cell_y) {
            (Some(x), Some(y)) => Some((x.into(), y.into())),
            _ => None,
        };

        BmaLayoutVariable {
            id: value.id.into(),
            container_id: value.container_id.map(|it| it.into()),
            r#type: value.r#type,
            name: value.name.clone(),
            description: value.description.clone(),
            position: (
                rational_or_default(value.position_x),
                rational_or_default(value.position_y),
            ),
            angle: rational_or_default(value.angle),
            cell,
        }
    }
}

impl From<BmaLayoutVariable> for JsonLayoutVariable {
    fn from(value: BmaLayoutVariable) -> Self {
        let (cell_x, cell_y) = match value.cell {
            Some(cell) => (Some(cell.0.into()), Some(cell.1.into())),
            None => (None, None),
        };

        JsonLayoutVariable {
            id: value.id.into(),
            name: value.name,
            r#type: value.r#type,
            position_x: f64_or_default(value.position.0),
            position_y: f64_or_default(value.position.1),
            angle: f64_or_default(value.angle),
            description: value.description.clone(),
            container_id: value.container_id.map(|it| it.into()),
            cell_x,
            cell_y,
        }
    }
}
