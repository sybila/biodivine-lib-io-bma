use crate::update_fn::bma_fn_update::BmaFnUpdate;
use crate::{BmaNetwork, ContextualValidation, ErrorReporter};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

/// A discrete variable identified by an integer `id`. Each [BmaVariable] consists
/// of a `name` (optional), its value `range` (inclusive), and an [BmaFnUpdate] function
/// formula (optional) which describes its evolution in time.
///
/// Expected invariants (checked during validation):
///  - Variable `id` must be unique within the enclosing [crate::BmaModel].
///  - Variable `name` cannot be empty but is not required to be unique.
///  - Variable `range` must be a valid range. However, a range that only contains a single
///    value is allowed, in which case the variable is considered constant.
///
/// Note that when `formula` is not specified, the typical interpretation is to assign
/// such a variable the "default" update function based on its associated relationships
/// (see also [crate::BmaRelationship] and [crate::BmaModel::create_default_update_fn]).
///
/// Additional non-functional information like the variable position, description, or type are
/// present as part of [crate::BmaLayout].
///
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BmaVariable {
    pub id: u32,
    pub name: Option<String>,
    pub range: (u32, u32),
    pub formula: Option<BmaFnUpdate>,
}

impl BmaVariable {
    /// The minimum value this variable can take.
    pub fn min_level(&self) -> u32 {
        self.range.0
    }

    /// The maximum value this variable can take.
    pub fn max_level(&self) -> u32 {
        self.range.1
    }

    /// Clone the variable name or create a default alternative (`v_ID`).
    pub fn name_or_default(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("v_{}", self.id))
    }
}

/// The default [BmaVariable] is Boolean, with no name or formula.
impl Default for BmaVariable {
    fn default() -> Self {
        BmaVariable {
            id: 0,
            name: None,
            range: (0, 1),
            formula: None,
        }
    }
}

/// Possible validation errors for [BmaVariable].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaVariableError {
    /// Caused by the variable ID not being unique.
    #[error("(Variable id: `{id}`) Id must be unique within the enclosing `BmaNetwork`")]
    IdNotUnique { id: u32 },
    /// Caused by the variable range not being a valid, non-empty interval.
    #[error("(Variable id: `{id}`) Range `{range:?}` is invalid; must be a non-empty interval")]
    RangeInvalid { id: u32, range: (u32, u32) },
    /// Caused by the variable name being empty.
    #[error("(Variable id: `{id}`) Name cannot be empty; use `None` instead")]
    NameEmpty { id: u32 },
}

impl ContextualValidation<BmaNetwork> for BmaVariable {
    type Error = BmaVariableError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, context: &BmaNetwork, reporter: &mut R) {
        // Ensure that the variable name is not empty.
        if let Some(name) = self.name.as_ref() {
            if name.is_empty() {
                reporter.report(BmaVariableError::NameEmpty { id: self.id });
            }
        }

        // Ensure that the variable range is a valid, non-empty interval.
        if self.range.0 > self.range.1 {
            reporter.report(BmaVariableError::RangeInvalid {
                id: self.id,
                range: self.range,
            });
        }

        // Ensure that the variable id is unique within the enclosing BmaNetwork.
        let mut count = 0;
        let mut found_self = false;
        for var in &context.variables {
            if var.id == self.id {
                count += 1;
                if var == self {
                    found_self = true;
                }
            }
        }

        assert!(
            found_self,
            "Validation called on a variable that is not part of the BmaNetwork"
        );

        if count > 1 {
            reporter.report(BmaVariableError::IdNotUnique { id: self.id });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::bma_variable::BmaVariableError;
    use crate::update_fn::bma_fn_update::BmaFnUpdate;
    use crate::{BmaNetwork, BmaVariable, ContextualValidation};
    use std::collections::HashMap;

    fn network_for_variable(variable: &BmaVariable) -> BmaNetwork {
        BmaNetwork {
            variables: vec![variable.clone()],
            ..Default::default()
        }
    }

    #[test]
    fn range_getters() {
        let variable = BmaVariable {
            range: (1, 3),
            ..Default::default()
        };
        assert_eq!(variable.min_level(), 1);
        assert_eq!(variable.max_level(), 3);
    }

    #[test]
    fn name_or_default() {
        let variable = BmaVariable {
            name: Some("foo".to_string()),
            ..Default::default()
        };
        assert_eq!(variable.name_or_default(), "foo");
        let variable = BmaVariable {
            id: 3,
            ..Default::default()
        };
        assert_eq!(variable.name_or_default(), "v_3");
    }

    #[test]
    fn default_serde() {
        let some_function =
            BmaFnUpdate::parse_from_str("var(0) - var(1)", &HashMap::new()).unwrap();
        let variable = BmaVariable {
            id: 5,
            name: Some("foo".to_string()),
            range: (1, 3),
            formula: Some(some_function),
        };
        let serialized = serde_json::to_string(&variable).unwrap();
        assert_eq!(
            serialized,
            r#"{"id":5,"name":"foo","range":[1,3],"formula":"(var(0) - var(1))"}"#
        );
        let deserialized: BmaVariable = serde_json::from_str(&serialized).unwrap();
        assert_eq!(variable, deserialized);
    }

    #[test]
    fn default_variable_is_valid() {
        let variable = BmaVariable::default();
        let network = network_for_variable(&variable);
        assert!(variable.validate(&network).is_ok());
    }

    #[test]
    #[should_panic]
    fn cannot_validate_when_not_in_network() {
        let variable = BmaVariable::default();
        let network = BmaNetwork::default();
        variable.validate(&network).unwrap();
    }

    /// Empty variable names are not allowed.
    #[test]
    fn empty_name() {
        let variable = BmaVariable {
            name: Some("".to_string()),
            ..Default::default()
        };
        let network = network_for_variable(&variable);

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(issues, vec![BmaVariableError::NameEmpty { id: 0 }]);
    }

    /// Empty ranges are allowed (represents a constant variable).
    #[test]
    fn range_empty() {
        let variable = BmaVariable {
            range: (1, 1),
            ..Default::default()
        };
        let network = network_for_variable(&variable);

        assert!(variable.validate(&network).is_ok());
    }

    /// Invalid ranges are not allowed.
    #[test]
    fn range_invalid() {
        let variable = BmaVariable {
            range: (3, 1),
            ..Default::default()
        };
        let network = network_for_variable(&variable);

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![BmaVariableError::RangeInvalid {
                id: 0,
                range: (3, 1)
            }]
        );
    }

    /// Two variables with the same ID are not allowed.
    #[test]
    fn duplicate_ids() {
        let v1 = BmaVariable::default();
        let v2 = BmaVariable::default();
        let network = BmaNetwork {
            variables: vec![v1.clone(), v2.clone()],
            ..Default::default()
        };

        let issues = v1.validate(&network).unwrap_err();
        assert_eq!(issues, vec![BmaVariableError::IdNotUnique { id: 0 }]);
    }
}
