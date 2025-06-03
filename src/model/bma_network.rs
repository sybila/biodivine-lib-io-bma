use crate::model::bma_relationship::BmaRelationshipError;
use crate::utils::is_blank;
use crate::{
    BmaRelationship, BmaVariable, BmaVariableError, ContextualValidation, ErrorReporter, Validation,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

/// Named model with several [`BmaVariable`] objects that are connected through various
/// [`BmaRelationship`] objects. The model name is optional.
///
/// This is the main part of [`crate::BmaModel`], and it is always required.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BmaNetwork {
    pub name: Option<String>,
    pub variables: Vec<BmaVariable>,
    pub relationships: Vec<BmaRelationship>,
}

impl BmaNetwork {
    /// Create a new [`BmaNetwork`] from the provided data.
    pub fn new(variables: Vec<BmaVariable>, relationships: Vec<BmaRelationship>) -> Self {
        BmaNetwork {
            name: None,
            variables,
            relationships,
        }
    }

    /// Find an instances of [`BmaVariable`] stored in this network, assuming it exists.
    pub fn find_variable(&self, id: u32) -> Option<&BmaVariable> {
        self.variables.iter().find(|v| v.id == id)
    }

    /// Get the current model name, or a default string if the name is not set.
    pub fn name_or_default(&self) -> String {
        self.name.clone().unwrap_or_else(|| "BMA Model".to_string())
    }
}

/// Possible validation errors for [`BmaNetwork`].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaNetworkError {
    #[error("Name of the `BmaNetwork` cannot be empty; use `None` instead")]
    NameEmpty,
    #[error(transparent)]
    Variable(#[from] BmaVariableError),
    #[error(transparent)]
    Relationship(#[from] BmaRelationshipError),
}

impl Validation for BmaNetwork {
    type Error = BmaNetworkError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, reporter: &mut R) {
        // Ensure that the name is not empty.
        if is_blank(&self.name) {
            reporter.report(BmaNetworkError::NameEmpty);
        }

        // Check all variables.
        for var in &self.variables {
            var.validate_all(self, &mut reporter.wrap());
        }

        // Check all relationships.
        for relationship in &self.relationships {
            relationship.validate_all(self, &mut reporter.wrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::tests::simple_network;
    use crate::{BmaNetwork, BmaNetworkError, Validation};

    #[test]
    fn default_network_is_valid() {
        let network = BmaNetwork::default();
        assert!(network.validate().is_ok());
    }

    #[test]
    fn simple_network_is_valid() {
        let network = simple_network();
        assert!(network.validate().is_ok());
    }

    #[test]
    fn empty_name() {
        let network = BmaNetwork {
            name: Some("".to_string()),
            ..Default::default()
        };
        let issues = network.validate().unwrap_err();
        assert_eq!(issues, vec![BmaNetworkError::NameEmpty]);
    }
}
