use crate::{BmaRelationship, BmaVariable};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Named model with several `variables` that have various `relationships`.
/// This is the main part of the BMA model, and it is always required.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BmaNetwork {
    pub name: String,
    pub variables: Vec<BmaVariable>,
    pub relationships: Vec<BmaRelationship>,
}
