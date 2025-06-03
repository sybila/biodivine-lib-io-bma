use crate::{BmaLayoutContainer, BmaLayoutVariable};
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A layout describing positions and types of variables and containers.
/// Most fields are optional, as the layout contains mostly complementary information.
///
/// Set of variables here should be a subset of the variables in the model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BmaLayout {
    pub variables: Vec<BmaLayoutVariable>,
    pub containers: Vec<BmaLayoutContainer>,
    pub description: Option<String>,
    pub zoom_level: Option<Rational64>,
    pub pan: Option<(Rational64, Rational64)>,
}

impl BmaLayout {
    /// Find an instances of [`BmaLayoutVariable`] stored in this layout, assuming it exists.
    pub fn find_variable(&self, id: u32) -> Option<&BmaLayoutVariable> {
        self.variables.iter().find(|v| v.id == id)
    }

    /// Find an instances of [`BmaLayoutContainer`] stored in this layout, assuming it exists.
    pub fn find_container(&self, id: u32) -> Option<&BmaLayoutContainer> {
        self.containers.iter().find(|v| v.id == id)
    }
}
