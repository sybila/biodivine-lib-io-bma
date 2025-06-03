use crate::update_fn::bma_fn_update::BmaFnUpdate;
use serde::{Deserialize, Serialize, Serializer};

/// A discrete variable with ID and name, range of possible values, and an update expression
/// that dictates how the variable evolves. Name string can be empty.
///
/// Additional not-functional information like variable's position, description, or type are
/// present as part of the layout (as is usual in BMA JSON format).
///
/// The update expression is optional. The `None` variant is used when empty update expression
/// is provided. Update expressions are serialized using custom `serialize_update_fn` function.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaVariable {
    pub id: u32,
    pub name: String,
    pub range_from: u32,
    pub range_to: u32,
    #[serde(serialize_with = "serialize_update_fn")]
    pub formula: Option<BmaFnUpdate>,
}

/// A utility to serialize update function by calling a custom parser.
fn serialize_update_fn<S>(update_fn: &Option<BmaFnUpdate>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(update_fn_str) = update_fn {
        s.serialize_str(update_fn_str.as_str())
    } else {
        s.serialize_str("")
    }
}
