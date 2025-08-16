use crate::{
    BmaLayoutContainer, BmaLayoutContainerError, BmaLayoutVariable, BmaLayoutVariableError,
    BmaModel, ContextualValidation, ErrorReporter,
};
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

/// A layout describing positions and types of variables and containers.
/// Most fields are optional, as the layout contains mostly complementary information.
///
/// Set of variables here should be a subset of the variables in the model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BmaLayout {
    pub variables: Vec<BmaLayoutVariable>,
    pub containers: Vec<BmaLayoutContainer>,
    pub description: String,
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

/// Possible validation errors for [`BmaLayout`].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaLayoutError {
    #[error(transparent)]
    Variable(#[from] BmaLayoutVariableError),
    #[error(transparent)]
    Container(#[from] BmaLayoutContainerError),
}

impl ContextualValidation<BmaModel> for BmaLayout {
    type Error = BmaLayoutError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, context: &BmaModel, reporter: &mut R) {
        for var in &self.variables {
            var.validate_all(context, &mut reporter.wrap());
        }

        for container in &self.containers {
            container.validate_all(self, &mut reporter.wrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::tests::{simple_layout, simple_network};
    use crate::{BmaLayout, BmaModel, BmaNetwork, ContextualValidation};

    #[test]
    fn default_layout_is_valid() {
        let layout = BmaLayout::default();
        let model = BmaModel {
            network: BmaNetwork::default(),
            layout: layout.clone(),
            metadata: Default::default(),
        };
        assert!(layout.validate(&model).is_ok());
    }

    #[test]
    fn simple_layout_is_valid() {
        let network = simple_network();
        let layout = simple_layout();
        let model = BmaModel {
            network,
            layout: layout.clone(),
            metadata: Default::default(),
        };
        assert!(layout.validate(&model).is_ok());
    }

    #[test]
    fn description_empty() {
        let layout = BmaLayout {
            description: String::default(),
            ..BmaLayout::default()
        };
        let model = BmaModel {
            network: simple_network(),
            layout: layout.clone(),
            metadata: Default::default(),
        };
        assert!(layout.validate(&model).is_ok());
    }
}
