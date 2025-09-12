use crate::update_function::{BmaUpdateFunction, FunctionTable, InvalidBmaExpression};
use crate::utils::is_unique_id;
use crate::{BmaNetwork, ContextualValidation, ErrorReporter};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

/// A discrete variable identified by an integer `id`. Each [`BmaVariable`] consists
/// of a `name` (can be blank), its value `range` (inclusive), and an [`BmaUpdateFunction`] function
/// formula (optional) which describes its evolution in time.
///
/// Expected invariants (checked during validation):
///  - Variable `id` must be unique within the variables of the enclosing [`crate::BmaNetwork`].
///  - Variable `name` can be blank and is not required to be unique.
///  - Variable `range` must be a valid range. However, a range that only contains a single
///    value is allowed, in which case the variable is considered constant.
///
/// Note that when `formula` is not specified, the typical interpretation is to assign
/// such a variable the "default" update function based on its associated relationships
/// as `avg(positive_regulators) - avg(negative_regulators)`.
///
/// Additional non-functional information like the variable position, description, or type are
/// present as part of [`crate::BmaLayout`].
///
/// ## Constant variables
///
/// A variable is considered to be a constant if its range only admits a single value `x`. In such
/// cases, we require that (1) the variable has no regulators, and (b) the `formula` is either
/// empty, or set to a constant value that is `0` or `x`. At the same time, if a corresponding
/// [`crate::BmaLayoutVariable`] exists, its `type` should be also set to `Constant`. Everything
/// else will be reported as validation error. However, These conditions are checked by
///
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BmaVariable {
    pub id: u32,
    pub name: String,
    pub range: (u32, u32),
    pub formula: Option<Result<BmaUpdateFunction, InvalidBmaExpression>>,
}

impl BmaVariable {
    /// Create a new *boolean* [`BmaVariable`] with the given `name`.
    #[must_use]
    pub fn new_boolean(id: u32, name: &str, formula: Option<BmaUpdateFunction>) -> Self {
        Self::new(id, name, (0, 1), formula)
    }

    /// Create a new [`BmaVariable`] with the given `name` and `range`.
    pub fn new(
        id: u32,
        name: &str,
        range: (u32, u32),
        formula: Option<BmaUpdateFunction>,
    ) -> BmaVariable {
        BmaVariable {
            id,
            name: name.to_string(),
            range,
            formula: formula.map(Ok),
        }
    }

    /// The minimum value this variable can take.
    #[must_use]
    pub fn min_level(&self) -> u32 {
        self.range.0
    }

    /// The maximum value this variable can take.
    #[must_use]
    pub fn max_level(&self) -> u32 {
        self.range.1
    }

    #[must_use]
    pub fn formula_string(&self) -> String {
        if let Some(formula) = &self.formula {
            match formula {
                Ok(f) => f.to_string(),
                Err(e) => e.expression.clone(),
            }
        } else {
            String::default()
        }
    }

    /// Returns true if the range of this variable is a single number.
    ///
    /// These variables are expected to have a constant update function. Note that there
    /// is also a constant variable type [`crate::VariableType`], but this is not always
    /// set consistently.
    #[must_use]
    pub fn has_constant_range(&self) -> bool {
        self.range.0 == self.range.1
    }

    /// Returns a reference to the update function of this variable, assuming the function is
    /// set and was parsed successfully.
    #[must_use]
    pub fn try_get_update_function(&self) -> Option<&BmaUpdateFunction> {
        self.formula.as_ref().and_then(|it| it.as_ref().ok())
    }
}

/// The default [`BmaVariable`] is Boolean, with no name or formula.
impl Default for BmaVariable {
    fn default() -> Self {
        BmaVariable {
            id: 0,
            name: String::default(),
            range: (0, 1),
            formula: None,
        }
    }
}

/// Possible validation errors for [`BmaVariable`].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaVariableError {
    #[error("(Variable id: `{id}`) Id must be unique in the `BmaNetwork`")]
    IdNotUnique { id: u32 },
    #[error("(Variable id: `{id}`) Range `{range:?}` is invalid; must be an interval")]
    RangeInvalid { id: u32, range: (u32, u32) },
    #[error("(Variable id: `{id}`) {source}")]
    UpdateFunctionInvalid {
        id: u32,
        #[source]
        source: InvalidBmaExpression,
    },
}

impl ContextualValidation<BmaNetwork> for BmaVariable {
    type Error = BmaVariableError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, context: &BmaNetwork, reporter: &mut R) {
        // Ensure that the variable range is a valid interval (start <= end).
        // Single-value ranges are allowed.
        if self.range.0 > self.range.1 {
            reporter.report(BmaVariableError::RangeInvalid {
                id: self.id,
                range: self.range,
            });
        }

        // Ensure that the variable id is unique within the enclosing BmaNetwork.
        let Ok(is_unique) = is_unique_id(&context.variables, self, |x| x.id) else {
            // This is not a validation error; this violates the whole contract of the validation
            // mechanism and is therefore allowed to fail (instead of returning an error).
            panic!("Precondition violation: validated variable is not part of the `BmaNetwork`.")
        };

        if !is_unique {
            reporter.report(BmaVariableError::IdNotUnique { id: self.id });
        }

        if let Some(Err(error)) = &self.formula {
            reporter.report(BmaVariableError::UpdateFunctionExpressionInvalid {
                id: self.id,
                source: error.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::bma_variable::BmaVariableError;
    use crate::update_function::BmaUpdateFunction;
    use crate::{BmaNetwork, BmaVariable, ContextualValidation};

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
            name: "foo".to_string(),
            ..Default::default()
        };
        assert_eq!(variable.name, "foo");
        let variable = BmaVariable {
            id: 3,
            ..Default::default()
        };
        assert_eq!(variable.name, "");
    }

    #[test]
    fn default_serde() {
        let formula = BmaUpdateFunction::try_from("var(0) - var(1)").unwrap();
        let variable = BmaVariable::new(5, "foo", (1, 3), Some(formula));
        let serialized = serde_json::to_string(&variable).unwrap();
        assert_eq!(
            serialized,
            r#"{"id":5,"name":"foo","range":[1,3],"formula":{"Ok":"(var(0) - var(1))"}}"#
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

    #[test]
    fn empty_name() {
        let variable = BmaVariable::new_boolean(0, "", None);
        let network = network_for_variable(&variable);

        assert!(variable.validate(&network).is_ok());
    }

    /// Empty ranges are allowed (represents a constant variable).
    #[test]
    fn range_empty() {
        let variable = BmaVariable::new(0, "v1", (1, 1), None);
        let network = network_for_variable(&variable);

        assert!(variable.validate(&network).is_ok());
    }

    /// Invalid ranges are not allowed.
    #[test]
    fn range_invalid() {
        let variable = BmaVariable::new(0, "v1", (3, 1), None);
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
