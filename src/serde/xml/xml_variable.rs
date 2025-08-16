use crate::serde::quote_num::QuoteNum;
use crate::serde::xml::XmlBmaModel;
use crate::update_fn::bma_fn_update::BmaFnUpdate;
use crate::utils::{f64_or_default, rational_or_default, take_if_not_blank};
use crate::{BmaLayoutVariable, BmaVariable, VariableType};
use serde::{Deserialize, Serialize};

/// Structure to deserialize XML info about a variable. BMA XML format mixes
/// functional and layout information for variables (unlike JSON),
/// which makes this a bit messy.
///
/// All variables must have ID, range of possible values, and an update formula.
/// The formula can be empty string. If the name is missing, we set it to an empty string.
/// If the type is missing, we set it to the default value.
///
/// All other layout details are optional. If not provided, we set them to `None` here,
/// and some are set to default values later as needed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct XmlVariable {
    // By default, ID and name are attributes, but they can be also present as child tags.
    #[serde(rename = "@Id", alias = "Id")]
    pub id: QuoteNum,
    #[serde(default, rename = "@Name", alias = "Name")]
    pub name: String,
    #[serde(rename = "RangeFrom")]
    pub range_from: QuoteNum,
    #[serde(rename = "RangeTo")]
    pub range_to: QuoteNum,
    #[serde(rename = "Formula", alias = "Function")]
    pub formula: String,

    #[serde(default, rename = "Type")]
    pub r#type: VariableType,
    #[serde(default, rename = "PositionX")]
    pub position_x: f64,
    #[serde(default, rename = "PositionY")]
    pub position_y: f64,
    #[serde(default, rename = "Angle")]
    pub angle: f64,
    #[serde(default, rename = "ContainerId")]
    pub container_id: Option<QuoteNum>,
    #[serde(default, rename = "CellX")]
    pub cell_x: Option<QuoteNum>,
    #[serde(default, rename = "CellY")]
    pub cell_y: Option<QuoteNum>,
}

impl From<BmaVariable> for XmlVariable {
    fn from(value: BmaVariable) -> Self {
        XmlVariable {
            id: value.id.into(),
            name: value.name.clone(),
            range_from: value.range.0.into(),
            range_to: value.range.1.into(),
            formula: value.formula.map(|it| it.to_string()).unwrap_or_default(),
            r#type: Default::default(),
            position_x: 0.0,
            position_y: 0.0,
            angle: 0.0,
            container_id: None,
            cell_x: None,
            cell_y: None,
        }
    }
}

impl From<(BmaVariable, BmaLayoutVariable)> for XmlVariable {
    fn from(value: (BmaVariable, BmaLayoutVariable)) -> Self {
        let (variable, layout) = value;
        let mut variable = XmlVariable::from(variable);
        variable.r#type = layout.r#type;
        variable.position_x = f64_or_default(layout.position.0);
        variable.position_y = f64_or_default(layout.position.1);
        variable.angle = f64_or_default(layout.angle);
        variable.container_id = layout.container_id.map(|it| it.into());
        if let Some((x, y)) = layout.cell {
            variable.cell_x = Some(x.into());
            variable.cell_y = Some(y.into());
        }
        variable
    }
}

impl TryFrom<(&XmlBmaModel, &XmlVariable)> for BmaVariable {
    type Error = anyhow::Error; // TODO: Replace with type safe error.

    fn try_from(value: (&XmlBmaModel, &XmlVariable)) -> Result<Self, Self::Error> {
        let (model, variable) = value;

        let variables = model.collect_all_variables();
        // TODO: Refactor code duplicate.
        let formula = if let Some(formula) = take_if_not_blank(variable.formula.as_str()) {
            Some(
                BmaFnUpdate::parse_from_str(formula.as_str(), &variables)
                    .map_err(anyhow::Error::msg)?,
            )
        } else {
            None
        };

        Ok(BmaVariable {
            id: variable.id.into(),
            name: variable.name.clone(),
            range: (variable.range_from.into(), variable.range_to.into()),
            formula,
        })
    }
}

impl From<XmlVariable> for BmaLayoutVariable {
    fn from(value: XmlVariable) -> Self {
        // In XML, most data about variable layout is stored directly with variables.
        let cell = match (value.cell_x, value.cell_y) {
            (Some(x), Some(y)) => Some((x.into(), y.into())),
            _ => None,
        };
        BmaLayoutVariable {
            id: value.id.into(),
            container_id: value.container_id.map(|it| it.into()),
            r#type: Default::default(),
            name: value.name.clone(),
            description: String::default(),
            position: (
                rational_or_default(value.position_x),
                rational_or_default(value.position_y),
            ),
            angle: rational_or_default(value.angle),
            cell,
        }
    }
}
