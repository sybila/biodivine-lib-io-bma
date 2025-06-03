use crate::update_fn::bma_fn_update::BmaFnUpdate;
use serde::{Deserialize, Serialize};

/// A discrete variable with ID and name, range of possible values, and an update expression
/// that dictates how the variable evolves. Name string can be empty.
///
/// Additional non-functional information like a variable position, description, or type are
/// present as part of the layout (as is usual in BMA JSON format).
///
/// The update expression is optional. The `None` variant is used when an empty update expression
/// is provided. Update expressions are serialized using a custom ` serialize_update_fn ` function.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaVariable {
    pub id: u32,
    pub name: String,
    pub range_from: u32,
    pub range_to: u32,
    pub formula: Option<BmaFnUpdate>,
}
