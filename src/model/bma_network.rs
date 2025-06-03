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
impl BmaNetwork {
    /// Create a new [`BmaNetwork`] from the provided data.
    pub fn new(variables: Vec<BmaVariable>, relationships: Vec<BmaRelationship>) -> Self {
        BmaNetwork {
            name: "".to_string(),
            variables,
            relationships,
        }
    }

    /// Find an instances of [`BmaVariable`] stored in this network, assuming it exists.
    pub fn find_variable(&self, id: u32) -> Option<&BmaVariable> {
        self.variables.iter().find(|v| v.id == id)
    }
}
