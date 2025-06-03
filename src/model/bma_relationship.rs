use crate::{BmaNetwork, BmaVariable};
use biodivine_lib_param_bn::Monotonicity;
use serde::{Deserialize, Serialize};

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
    use crate::{BmaNetwork, BmaRelationship, BmaVariable, RelationshipType};
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
}
