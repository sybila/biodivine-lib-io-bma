use crate::BmaVariable;
use crate::data::json_model::JsonBmaModel;
use crate::data::quote_num::QuoteNum;
use crate::update_fn::bma_fn_update::BmaFnUpdate;
use crate::utils::take_if_not_blank;
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
            name: value.name.unwrap_or_default(),
            range_from: value.range.0.into(),
            range_to: value.range.1.into(),
            formula: value.formula.map(|it| it.to_string()).unwrap_or_default(),
        }
    }
}

impl TryFrom<(&JsonBmaModel, &JsonVariable)> for BmaVariable {
    type Error = String;

    fn try_from(value: (&JsonBmaModel, &JsonVariable)) -> Result<BmaVariable, String> {
        let (model, variable) = value;

        let variables = model.variable_name_map();
        let formula = if let Some(formula) = take_if_not_blank(variable.formula.as_str()) {
            Some(BmaFnUpdate::parse_from_str(formula.as_str(), &variables)?)
        } else {
            None
        };

        Ok(BmaVariable {
            id: variable.id.into(),
            name: take_if_not_blank(variable.name.as_str()),
            range: (variable.range_from.into(), variable.range_to.into()),
            formula,
        })
    }
}
