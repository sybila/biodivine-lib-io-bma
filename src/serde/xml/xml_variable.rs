use crate::serde::xml::XmlBmaModel;
use crate::update_function::read_fn_update;
use crate::utils::{f64_or_default, rational_or_default};
use crate::{BmaLayoutVariable, BmaVariable};
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
    pub id: u32,
    #[serde(default, rename = "@Name", alias = "Name")]
    pub name: String,
    #[serde(rename = "RangeFrom")]
    pub range_from: u32,
    #[serde(rename = "RangeTo")]
    pub range_to: u32,
    #[serde(default, rename = "Formula", alias = "Function")]
    pub formula: String,

    #[serde(default, rename = "Type")]
    pub r#type: String,
    #[serde(default, rename = "PositionX")]
    pub position_x: f64,
    #[serde(default, rename = "PositionY")]
    pub position_y: f64,
    #[serde(default, rename = "Angle")]
    pub angle: f64,
    #[serde(default, rename = "ContainerId")]
    pub container_id: Option<u32>,
    #[serde(default, rename = "CellX")]
    pub cell_x: Option<u32>,
    #[serde(default, rename = "CellY")]
    pub cell_y: Option<u32>,
}

impl From<BmaVariable> for XmlVariable {
    fn from(value: BmaVariable) -> Self {
        XmlVariable {
            id: value.id,
            name: value.name.clone(),
            range_from: value.range.0,
            range_to: value.range.1,
            formula: value.formula_string(),
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
        variable.r#type = layout.r#type.to_string();
        variable.position_x = f64_or_default(layout.position.0);
        variable.position_y = f64_or_default(layout.position.1);
        variable.angle = f64_or_default(layout.angle);
        variable.container_id = layout.container_id;
        if let Some((x, y)) = layout.cell {
            variable.cell_x = Some(x);
            variable.cell_y = Some(y);
        }
        variable
    }
}

impl From<(&XmlBmaModel, &XmlVariable)> for BmaVariable {
    fn from(value: (&XmlBmaModel, &XmlVariable)) -> Self {
        let (model, variable) = value;

        let variables = model.regulators(variable.id);

        BmaVariable {
            id: variable.id,
            name: variable.name.clone(),
            range: (variable.range_from, variable.range_to),
            formula: read_fn_update(variable.formula.as_str(), &variables),
        }
    }
}

impl From<XmlVariable> for BmaLayoutVariable {
    fn from(value: XmlVariable) -> Self {
        // In XML, most data about variable layout is stored directly with variables.
        let cell = match (value.cell_x, value.cell_y) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        };
        BmaLayoutVariable {
            id: value.id,
            container_id: value.container_id,
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
