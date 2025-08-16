use crate::BmaVariable;
use crate::serde::json::JsonBmaModel;
use crate::serde::quote_num::QuoteNum;
use crate::update_fn::read_fn_update;
use serde::{Deserialize, Serialize};

/// Structure to deserialize JSON info about individual variable.
///
/// All variables must have ID, range of possible values, and an update formula.
/// The formula can be empty string.
/// Name is optional and set to None is not provided.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct JsonVariable {
    #[serde(rename = "Id", alias = "id")]
    pub id: QuoteNum,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: String,
    #[serde(rename = "RangeFrom", alias = "rangeFrom")]
    pub range_from: QuoteNum,
    #[serde(rename = "RangeTo", alias = "rangeTo")]
    pub range_to: QuoteNum,
    #[serde(rename = "Formula", alias = "formula")]
    pub formula: String,
}

impl From<BmaVariable> for JsonVariable {
    fn from(value: BmaVariable) -> Self {
        JsonVariable {
            id: value.id.into(),
            name: value.name.clone(),
            range_from: value.range.0.into(),
            range_to: value.range.1.into(),
            formula: value.formula_string(),
        }
    }
}

impl From<(&JsonBmaModel, &JsonVariable)> for BmaVariable {
    fn from(value: (&JsonBmaModel, &JsonVariable)) -> BmaVariable {
        let (model, variable) = value;

        let variables = model.variable_name_map();

        BmaVariable {
            id: variable.id.into(),
            name: variable.name.clone(),
            range: (variable.range_from.into(), variable.range_to.into()),
            formula: read_fn_update(variable.formula.as_str(), &variables),
        }
    }
}
