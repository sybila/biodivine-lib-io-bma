use crate::{BmaNetwork, BmaVariable, ContextualValidation, ErrorReporter};
use biodivine_lib_param_bn::Monotonicity;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
    pub r#type: RelationshipType, // Corresponds to "Type" in JSON/XML
}

impl BmaRelationship {
    /// Make a new [`RelationshipType::Activator`] relationship between two variables.
    pub fn new_activator(id: u32, from: u32, to: u32) -> Self {
        BmaRelationship {
            id,
            from_variable: from,
            to_variable: to,
            r#type: RelationshipType::Activator,
        }
    }

    /// Make a new [`RelationshipType::Inhibitor`] relationship between two variables.
    pub fn new_inhibitor(id: u32, from: u32, to: u32) -> Self {
        BmaRelationship {
            id,
            from_variable: from,
            to_variable: to,
            r#type: RelationshipType::Inhibitor,
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
    #[error("(Relationship: `{id}`) Id must be unique in the `BmaNetwork`")]
    IdNotUnique { id: u32 },
    #[error("(Relationship: `{id}`) Regulator (`{from_variable}`) not found in the `BmaNetwork`")]
    RegulatorVariableNotFound { id: u32, from_variable: u32 },
    #[error("(Relationship: `{id}`) Target (`{to_variable}`) not found in the `BmaNetwork`")]
    TargetVariableNotFound { id: u32, to_variable: u32 },
}

/// The type of [`BmaRelationship`] between two variables in a [`BmaNetwork`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum RelationshipType {
    #[default]
    Activator,
    Inhibitor,
    Unknown(String),
}

/*
   For serialization, we need to override the default behavior, which in XML is to
   serialize/deserialize using tags, not string values.
*/

impl Serialize for RelationshipType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelationshipType::Activator => serializer.serialize_str("Activator"),
            RelationshipType::Inhibitor => serializer.serialize_str("Inhibitor"),
            RelationshipType::Unknown(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for RelationshipType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "activator" => Ok(RelationshipType::Activator),
            "inhibitor" => Ok(RelationshipType::Inhibitor),
            _ => Ok(RelationshipType::Unknown(s)),
        }
    }
}

impl TryFrom<RelationshipType> for Monotonicity {
    type Error = ();

    fn try_from(value: RelationshipType) -> Result<Self, Self::Error> {
        match value {
            RelationshipType::Activator => Ok(Monotonicity::Activation),
            RelationshipType::Inhibitor => Ok(Monotonicity::Inhibition),
            RelationshipType::Unknown(_value) => Err(()),
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
            Monotonicity::try_from(RelationshipType::Activator).unwrap(),
            Monotonicity::Activation
        );
        assert_eq!(
            Monotonicity::try_from(RelationshipType::Inhibitor).unwrap(),
            Monotonicity::Inhibition
        );
        assert_eq!(
            RelationshipType::try_from(Monotonicity::Activation).unwrap(),
            RelationshipType::Activator
        );
        assert_eq!(
            RelationshipType::try_from(Monotonicity::Inhibition).unwrap(),
            RelationshipType::Inhibitor
        );
    }

    #[test]
    fn relationship_serialization() {
        let relationship = BmaRelationship::new_inhibitor(5, 3, 6);
        let serialized = serde_json::to_string(&relationship).unwrap();
        assert_eq!(
            serialized,
            r#"{"id":5,"from_variable":3,"to_variable":6,"type":"Inhibitor"}"#
        );
        let deserialized: BmaRelationship = serde_json::from_str(&serialized).unwrap();
        assert_eq!(relationship, deserialized);
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
        );
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
        );
    }

    #[test]
    fn duplicate_ids() {
        let v = BmaVariable::default();
        let r = BmaRelationship::default();
        let network = BmaNetwork::new(vec![v], vec![r.clone(), r.clone()]);

        let issues = r.validate(&network).unwrap_err();
        assert_eq!(issues, vec![BmaRelationshipError::IdNotUnique { id: 0 }]);
    }
}
