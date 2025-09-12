use crate::model::bma_relationship::BmaRelationshipError;
use crate::update_function::{BmaUpdateFunction, InvalidBmaExpression, create_default_update_fn};
use crate::{
    BmaRelationship, BmaVariable, BmaVariableError, ContextualValidation, ErrorReporter,
    RelationshipType, Validation,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashSet;
use std::mem::replace;
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

/// Utility methods for dealing with default functions.
impl BmaNetwork {
    /// Build the default update function which is used by BMA if no other function is provided.
    #[must_use]
    pub fn build_default_update_function(&self, var_id: u32) -> BmaUpdateFunction {
        create_default_update_fn(self, var_id)
    }

    /// Modify this BMA model such that the given variable uses the default update function.
    ///
    /// Returns the previous update function.
    ///
    /// See also [`BmaNetwork::build_default_update_function`].
    ///
    /// # Panics
    ///
    /// Panics if the given `var_id` does not reference a network variable.
    pub fn set_default_function(
        &mut self,
        var_id: u32,
    ) -> Option<Result<BmaUpdateFunction, InvalidBmaExpression>> {
        let update = self.build_default_update_function(var_id);
        let variable = self
            .variables
            .iter_mut()
            .find(|v| v.id == var_id)
            .expect("Precondition violated: No variable with given id.");
        replace(&mut variable.formula, Some(Ok(update)))
    }

    /// Add default update functions for all variables where the update function is missing.
    pub fn populate_missing_functions(&mut self) {
        let missing_var_ids = self
            .variables
            .iter()
            .filter(|v| v.formula.is_none())
            .map(|v| v.id)
            .collect::<Vec<_>>();
        for id in missing_var_ids {
            let _ = self.set_default_function(id); // throw away the old function
        }
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
