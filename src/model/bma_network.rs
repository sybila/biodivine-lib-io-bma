use crate::model::bma_relationship::BmaRelationshipError;
use crate::{
    BmaRelationship, BmaVariable, BmaVariableError, ContextualValidation, ErrorReporter,
    RelationshipType, Validation,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashSet;
use thiserror::Error;

/// Named model with several [`BmaVariable`] objects that are connected through various
/// [`BmaRelationship`] objects. The model name can be blank.
///
/// This is the main part of [`crate::BmaModel`], and it is always required.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BmaNetwork {
    pub name: String,
    pub variables: Vec<BmaVariable>,
    pub relationships: Vec<BmaRelationship>,
}

impl BmaNetwork {
    /// Create a new [`BmaNetwork`] from the provided data.
    #[must_use]
    pub fn new(variables: Vec<BmaVariable>, relationships: Vec<BmaRelationship>) -> Self {
        BmaNetwork {
            name: String::default(),
            variables,
            relationships,
        }
    }

    /// Find an instances of [`BmaVariable`] stored in this network, assuming it exists.
    #[must_use]
    pub fn find_variable(&self, id: u32) -> Option<&BmaVariable> {
        self.variables.iter().find(|v| v.id == id)
    }

    /// Get regulators of a particular variable, optionally filtered by regulator type.
    /// The regulators are represented by their IDs.
    ///
    /// If network validation passed successfully, you can assume that there is no
    /// [`RelationshipType::Unknown`] (i.e. every relationship is either an activator,
    /// or an inhibitor).
    #[must_use]
    pub fn get_regulators(
        &self,
        target_var: u32,
        relationship: &Option<RelationshipType>,
    ) -> HashSet<u32> {
        self.relationships
            .iter()
            .filter(|r| r.to_variable == target_var)
            .filter(|r| relationship.as_ref().is_none_or(|x| *x == r.r#type))
            .map(|r| r.from_variable)
            .collect()
    }
}

/// Possible validation errors for [`BmaNetwork`].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaNetworkError {
    #[error(transparent)]
    Variable(#[from] BmaVariableError),
    #[error(transparent)]
    Relationship(#[from] BmaRelationshipError),
}

impl Validation for BmaNetwork {
    type Error = BmaNetworkError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, reporter: &mut R) {
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
    use crate::{BmaNetwork, Validation};

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
}
