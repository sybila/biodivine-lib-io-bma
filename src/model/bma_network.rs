use crate::{BmaRelationship, BmaVariable};
use serde::{Deserialize, Serialize};

/// Named model with several `variables` that have various `relationships`.
/// This is the main part of the BMA model, and it is always required.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BmaNetwork {
    pub name: String,
    pub variables: Vec<BmaVariable>,
    pub relationships: Vec<BmaRelationship>,
}
