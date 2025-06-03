use crate::{BmaNetwork, BmaVariable, ContextualValidation, ErrorReporter};
use biodivine_lib_param_bn::Monotonicity;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A relationship of a given [`RelationshipType`] between two [`BmaVariable`] objects.
///
/// Expected invariants (checked during validation):
///  - Relationship `id` must be unique within the relationships of the
///    enclosing [`BmaNetwork`].
///  - Variables `from_variable` and `to_variable` must exist in
///    the enclosing [`BmaNetwork`].
///
/// Note that in theory, multiple relationships are allowed between the same pair of variables.
/// If they have the same type, it is equivalent to having a single relationship. If they
/// have different types, it is equivalent to having both an activator and an inhibitor at the
/// same time (i.e., a non-monotonic relationship).
///
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BmaRelationship {
    pub id: u32,
    pub from_variable: u32,
    pub to_variable: u32,
    #[serde(rename = "Type")]
    pub relationship_type: RelationshipType, // Corresponds to "Type" in JSON/XML
}

impl BmaRelationship {
    /// Make a new [`RelationshipType::Activator`] relationship between two variables.
    pub fn new_activator(id: u32, from: u32, to: u32) -> Self {
        BmaRelationship {
            id,
            from_variable: from,
            to_variable: to,
            relationship_type: RelationshipType::Activator,
        }
    }

    /// Make a new [`RelationshipType::Inhibitor`] relationship between two variables.
    pub fn new_inhibitor(id: u32, from: u32, to: u32) -> Self {
        BmaRelationship {
            id,
            from_variable: from,
            to_variable: to,
            relationship_type: RelationshipType::Inhibitor,
        }
    }

    /// Find the regulator variable (`from_variable`) in the enclosing [`BmaNetwork`], assuming
    /// the regulator variable exists.
    pub fn find_regulator_variable<'a>(&self, network: &'a BmaNetwork) -> Option<&'a BmaVariable> {
        network.find_variable(self.from_variable)
    }

    /// Find the target variable (`to_variable`) in the enclosing [`BmaNetwork`], assuming
    /// the target variable exists.
    pub fn find_target_variable<'a>(&self, network: &'a BmaNetwork) -> Option<&'a BmaVariable> {
        network.find_variable(self.to_variable)
    }
}

impl ContextualValidation<BmaNetwork> for BmaRelationship {
    type Error = BmaRelationshipError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, context: &BmaNetwork, reporter: &mut R) {
        // Ensure that regulator and target exist in the enclosing BmaNetwork.

        if self.find_regulator_variable(context).is_none() {
            reporter.report(BmaRelationshipError::RegulatorVariableNotFound {
                id: self.id,
                from_variable: self.from_variable,
            })
        }

        if self.find_target_variable(context).is_none() {
            reporter.report(BmaRelationshipError::TargetVariableNotFound {
                id: self.id,
                to_variable: self.to_variable,
            })
        }

        // Ensure that the relationship id is unique within the enclosing BmaNetwork.

        let mut count = 0;
        let mut found_self = false;
        for relationship in &context.relationships {
            if relationship.id == self.id {
                count += 1;
                if relationship == self {
                    found_self = true;
                }
            }
        }

        assert!(
            found_self,
            "Validation called on a relationship that is not part of the BmaNetwork"
        );

        if count > 1 {
            reporter.report(BmaRelationshipError::IdNotUnique { id: self.id });
        }
    }
}

/// Possible validation errors for [`BmaRelationship`].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaRelationshipError {
    /// Caused by the relationship ID not being unique.
    #[error("(Relationship id: `{id}`) Id must be unique within the enclosing `BmaNetwork`")]
    IdNotUnique { id: u32 },
    #[error(
        "(Relationship id: `{id}`) Regulator (`{from_variable}`) not found in the enclosing `BmaNetwork`"
    )]
    RegulatorVariableNotFound { id: u32, from_variable: u32 },
    #[error(
        "(Relationship id: `{id}`) Target (`{to_variable}`) not found in the enclosing `BmaNetwork`"
    )]
    TargetVariableNotFound { id: u32, to_variable: u32 },
}

/// The type of [`BmaRelationship`] between two variables in a [`BmaNetwork`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RelationshipType {
    #[default]
    Activator,
    Inhibitor,
}

impl From<RelationshipType> for Monotonicity {
    fn from(val: RelationshipType) -> Self {
        match val {
            RelationshipType::Activator => Monotonicity::Activation,
            RelationshipType::Inhibitor => Monotonicity::Inhibition,
        }
    }
}

impl From<Monotonicity> for RelationshipType {
    fn from(val: Monotonicity) -> Self {
        match val {
            Monotonicity::Activation => RelationshipType::Activator,
            Monotonicity::Inhibition => RelationshipType::Inhibitor,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::bma_relationship::BmaRelationshipError;
    use crate::{BmaNetwork, BmaRelationship, BmaVariable, ContextualValidation, RelationshipType};
    use biodivine_lib_param_bn::Monotonicity;

    #[test]
    fn variable_getters() {
        let v1 = BmaVariable::new_boolean(0, "v1", None);
        let v2 = BmaVariable::new_boolean(1, "v2", None);
        let r1 = BmaRelationship::new_activator(0, 0, 1);
        let r2 = BmaRelationship::new_activator(0, 1, 2);

        let network = BmaNetwork::new(vec![v1.clone(), v2.clone()], vec![r1.clone(), r2.clone()]);

        assert_eq!(r1.find_regulator_variable(&network), Some(&v1));
        assert_eq!(r1.find_target_variable(&network), Some(&v2));
        assert_eq!(r2.find_regulator_variable(&network), Some(&v2));
        assert_eq!(r2.find_target_variable(&network), None);
    }

    #[test]
    fn relationship_conversions() {
        assert_eq!(
            Monotonicity::from(RelationshipType::Activator),
            Monotonicity::Activation
        );
        assert_eq!(
            Monotonicity::from(RelationshipType::Inhibitor),
            Monotonicity::Inhibition
        );
        assert_eq!(
            RelationshipType::from(Monotonicity::Activation),
            RelationshipType::Activator
        );
        assert_eq!(
            RelationshipType::from(Monotonicity::Inhibition),
            RelationshipType::Inhibitor
        );
    }

    #[test]
    #[should_panic]
    fn cannot_validate_when_not_in_network() {
        let relationship = BmaRelationship::default();
        let network = BmaNetwork::default();
        relationship.validate(&network).unwrap();
    }

    #[test]
    fn default_relationship_is_valid() {
        let v = BmaVariable::default();
        let r = BmaRelationship::default();
        let network = BmaNetwork::new(vec![v], vec![r.clone()]);
        assert!(r.validate(&network).is_ok());
    }

    #[test]
    fn unknown_regulator_variable() {
        let v = BmaVariable::default();
        let r = BmaRelationship::new_activator(0, 1, 0);
        let network = BmaNetwork::new(vec![v], vec![r.clone()]);

        let issues = r.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![BmaRelationshipError::RegulatorVariableNotFound {
                id: 0,
                from_variable: 1,
            }]
        )
    }

    #[test]
    fn unknown_target_variable() {
        let v = BmaVariable::default();
        let r = BmaRelationship::new_activator(0, 0, 1);
        let network = BmaNetwork::new(vec![v], vec![r.clone()]);

        let issues = r.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![BmaRelationshipError::TargetVariableNotFound {
                id: 0,
                to_variable: 1,
            }]
        )
    }
}
