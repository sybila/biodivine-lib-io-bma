use crate::update_function::{BmaUpdateFunction, FunctionTable, InvalidBmaExpression};
use crate::utils::is_unique_id;
use crate::{BmaNetwork, ContextualValidation, ErrorReporter, RelationshipType};
use BmaVariableError::{
    CannotBuildFunctionTable, ConstantWithRegulators, ConstantWithUpdateFunction,
    UpdateFunctionRegulatorInvalid,
};
use RelationshipType::{Activator, Inhibitor};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::cmp::Ordering;
use std::collections::BTreeMap;
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
/// else will be reported as validation error.
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

    /// True if the domain is exactly `[0,1]`.
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        self.range == (0, 1)
    }

    /// Returns a reference to the update function of this variable, assuming the function is
    /// set and was parsed successfully.
    #[must_use]
    pub fn try_get_update_function(&self) -> Option<&BmaUpdateFunction> {
        self.formula.as_ref().and_then(|it| it.as_ref().ok())
    }

    /// Create a string identifier that contains the variable ID, variable name (if set) and
    /// given level in a human-readable format.
    ///
    /// # Panics
    /// The given `level` must be valid in the range of this variable.
    #[must_use]
    pub(crate) fn mk_level_identifier(&self, level: u32) -> String {
        assert!(level >= self.range.0 && level <= self.range.1);
        if self.name.is_empty() {
            format!("{}[{}]", self.id, level)
        } else {
            format!("{}_{}[{}]", self.id, self.name, level)
        }
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
    #[error(
        "(Variable id: `{id}`) Variable appears to be a constant (`{value}`), but has update function `{expression}`"
    )]
    ConstantWithUpdateFunction {
        id: u32,
        value: u32,
        expression: String,
    },
    #[error(
        "(Variable id: `{id}`) Variable appears to be a constant (`{value}`), but has regulators `{regulators:?}`"
    )]
    ConstantWithRegulators {
        id: u32,
        value: u32,
        regulators: Vec<u32>,
    },
    #[error("(Variable id: `{id}`) {source}")]
    UpdateFunctionExpressionInvalid {
        id: u32,
        #[source]
        source: InvalidBmaExpression,
    },
    #[error("(Variable id: `{id}`) Regulator `{regulator}` is invalid: {source}")]
    UpdateFunctionRegulatorInvalid {
        id: u32,
        regulator: u32,
        #[source]
        source: RegulatorErrorType,
    },
    #[error("(Variable id: `{id}`) Cannot build function table: {error}")]
    CannotBuildFunctionTable { id: u32, error: String },
}

/// Possible validation error type for [`BmaVariable`] concerning function regulators.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RegulatorErrorType {
    #[error("Variable does not exist")]
    MissingVariable,
    #[error("Variable not declared as regulator")]
    MissingRelationship,
    #[error("Variable does not influence function output")]
    UnusedRelationship,
    #[error("Declared monotonicity is `{declared:?}`, but observed monotonicity is `{observed:?}`")]
    BadMonotonicity {
        declared: Vec<RelationshipType>,
        observed: Vec<RelationshipType>,
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

        let mut regulators = Vec::from_iter(context.get_regulators(self.id, &None));
        regulators.sort_unstable();

        if self.has_constant_range() {
            validate_constant_variable_update(self, &regulators, reporter);
        } else {
            validate_dynamic_variable_update(context, self, &regulators, reporter);
        }
    }
}

fn validate_dynamic_variable_update<R: ErrorReporter<BmaVariableError>>(
    context: &BmaNetwork,
    variable: &BmaVariable,
    regulators: &[u32],
    reporter: &mut R,
) {
    // For non-constant variables, we need to make sure they use valid regulators
    // and have a "reasonable" update function.

    // Note: If a variable from regulators does not exist, this is already reported as
    //       relationship validation error, so we don't need to test for it here.

    // 1. All used variables exist and are regulators.
    if let Some(formula) = variable.try_get_update_function() {
        let syntactic_regulators = formula.collect_variables();

        for reg_var in syntactic_regulators {
            if context.find_variable(reg_var).is_none() {
                reporter.report(UpdateFunctionRegulatorInvalid {
                    id: variable.id,
                    regulator: reg_var,
                    source: RegulatorErrorType::MissingVariable,
                });
            }
            if !regulators.contains(&reg_var) {
                reporter.report(UpdateFunctionRegulatorInvalid {
                    id: variable.id,
                    regulator: reg_var,
                    source: RegulatorErrorType::MissingRelationship,
                });
            }
        }
    }

    // 2. All declared regulations have valid monotonicity and essentiality.

    let function_table = context.build_function_table(variable.id);
    match function_table {
        Err(error) => reporter.report(CannotBuildFunctionTable {
            id: variable.id,
            error: error.to_string(),
        }),
        Ok(mut function_table) => {
            let declared_activators = context.get_regulators(variable.id, &Some(Activator));
            let declared_inhibitors = context.get_regulators(variable.id, &Some(Inhibitor));

            for reg_var in regulators {
                let observed = infer_relationship_type(&mut function_table, *reg_var);
                if observed.is_empty() {
                    reporter.report(UpdateFunctionRegulatorInvalid {
                        id: variable.id,
                        regulator: *reg_var,
                        source: RegulatorErrorType::UnusedRelationship,
                    });
                } else {
                    let mut declared = Vec::new();
                    if declared_activators.contains(reg_var) {
                        declared.push(Activator);
                    }
                    if declared_inhibitors.contains(reg_var) {
                        declared.push(Inhibitor);
                    }
                    if declared != observed {
                        reporter.report(UpdateFunctionRegulatorInvalid {
                            id: variable.id,
                            regulator: *reg_var,
                            source: RegulatorErrorType::BadMonotonicity { declared, observed },
                        });
                    }
                }
            }
        }
    }
}

/// Validate the update function of a single constant variable.
fn validate_constant_variable_update<R: ErrorReporter<BmaVariableError>>(
    variable: &BmaVariable,
    regulators: &[u32],
    reporter: &mut R,
) {
    // Make sure that a constant has no regulators and has an update function that is
    // compatible with its domain.

    let const_value = variable.min_level();

    // A constant should have no regulators.
    if !regulators.is_empty() {
        reporter.report(ConstantWithRegulators {
            id: variable.id,
            value: const_value,
            regulators: regulators.to_vec(),
        });
    }

    // A constant should have update function that is either empty, or constant/zero
    if let Some(formula) = variable.try_get_update_function() {
        let is_ok = if let Some(const_i32) = formula.as_constant() {
            // If the function is constant, the value must be zero or constant.
            u32::try_from(const_i32).is_ok_and(|it| it == 0 || it == const_value)
        } else {
            // If the function is not constant, this is immediately an error.
            false
        };
        if !is_ok {
            reporter.report(ConstantWithUpdateFunction {
                id: variable.id,
                value: const_value,
                expression: formula.to_string(),
            });
        }
    }
}

/// Infer the type of relationships that are present for the given regulator in the given
/// function table. If the regulator has no impact on the output, result is empty. If the regulator
/// is non-monotonic, the result contains both relationship types (activation, inhibition).
/// Otherwise, only one relationship type is returned.
///
/// The reason why we need a mutable reference to `table` is that we need to sort it. Otherwise,
/// it is not modified.
fn infer_relationship_type(table: &mut FunctionTable, regulator: u32) -> Vec<RelationshipType> {
    // If there is at least one regulator, the table should have at least two entries.
    debug_assert!(table.len() > 1);

    // Gather all other regulators (arbitrary order is fine)
    let mut regulator_ordering = table[0]
        .0
        .keys()
        .copied()
        .filter(|it| *it != regulator)
        .collect::<Vec<_>>();
    // Tested regulator then comes first.
    regulator_ordering.insert(0, regulator);

    // Sort the table so that the "primary key" for the input valuations is the regulator.
    table.sort_by(|(v1, _), (v2, _)| compare_two_inputs(v1, v2, &regulator_ordering));

    // Compute the domain size (first entry should have the lowest and last
    // entry the greatest level)
    let min_level = table[0].0.get(&regulator).copied().unwrap();
    let max_level = table[table.len() - 1].0.get(&regulator).copied().unwrap();
    let domain_size = usize::try_from(max_level - min_level + 1).unwrap();

    // Table length should be divisible by domain size.
    assert_eq!(table.len() % domain_size, 0);

    let skip_by = table.len() / domain_size;

    let mut is_activation = false;
    let mut is_inhibition = false;

    for i in 0..(table.len() - skip_by) {
        let j = i + skip_by;
        let out_i = table[i].1;
        let out_j = table[j].1;
        if out_i < out_j {
            is_activation = true;
        }
        if out_i > out_j {
            is_inhibition = true;
        }
    }

    let mut result = Vec::new();
    if is_activation {
        result.push(Activator);
    }
    if is_inhibition {
        result.push(Inhibitor);
    }

    result
}

/// Compare two input valuations using the given variable ordering. Variables not present
/// in the ordering will not be considered in the comparison.
fn compare_two_inputs(
    a: &BTreeMap<u32, u32>,
    b: &BTreeMap<u32, u32>,
    priority: &[u32],
) -> Ordering {
    for var in priority {
        let a_val = a.get(var).unwrap();
        let b_val = b.get(var).unwrap();
        let ord = a_val.cmp(b_val);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

#[cfg(test)]
mod tests {
    use crate::BmaVariableError::CannotBuildFunctionTable;
    use crate::RelationshipType::{Activator, Inhibitor};
    use crate::model::bma_variable::{BmaVariableError, RegulatorErrorType};
    use crate::update_function::BmaUpdateFunction;
    use crate::{BmaNetwork, BmaRelationship, BmaVariable, ContextualValidation};
    use BmaVariableError::{
        ConstantWithRegulators, ConstantWithUpdateFunction, IdNotUnique, RangeInvalid,
        UpdateFunctionRegulatorInvalid,
    };

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
            vec![RangeInvalid {
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
        assert_eq!(issues, vec![IdNotUnique { id: 0 }]);
    }

    #[test]
    fn constant_with_regulators() {
        let variable = BmaVariable::new(0, "v1", (3, 3), None);
        let mut network = network_for_variable(&variable);
        network
            .relationships
            .push(BmaRelationship::new_activator(0, 2, 0));

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![ConstantWithRegulators {
                id: 0,
                value: 3,
                regulators: vec![2],
            }]
        );
    }

    #[test]
    fn constant_with_update_function() {
        let update = BmaUpdateFunction::try_from("var(0) + var(1)").unwrap();
        let variable = BmaVariable::new(0, "v1", (3, 3), Some(update));
        let network = network_for_variable(&variable);

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![ConstantWithUpdateFunction {
                id: 0,
                value: 3,
                expression: "(var(0) + var(1))".to_string(),
            }]
        );
    }

    #[test]
    fn unknown_regulator() {
        let update = BmaUpdateFunction::try_from("var(2)").unwrap();
        let variable = BmaVariable::new(0, "v1", (0, 3), Some(update));
        let mut network = network_for_variable(&variable);
        network
            .relationships
            .push(BmaRelationship::new_activator(0, 2, 0));

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![
                UpdateFunctionRegulatorInvalid {
                    id: 0,
                    regulator: 2,
                    source: RegulatorErrorType::MissingVariable,
                },
                CannotBuildFunctionTable {
                    id: 0,
                    error: "Regulator variable `2` does not exist".to_string(),
                }
            ]
        );
    }

    #[test]
    fn missing_relationship() {
        let update = BmaUpdateFunction::try_from("var(0)").unwrap();
        let variable = BmaVariable::new(0, "v1", (0, 3), Some(update));
        let network = network_for_variable(&variable);

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![
                UpdateFunctionRegulatorInvalid {
                    id: 0,
                    regulator: 0,
                    source: RegulatorErrorType::MissingRelationship,
                },
                CannotBuildFunctionTable {
                    id: 0,
                    error: "Missing input value for variable `0`".to_string(),
                }
            ]
        );
    }

    #[test]
    fn unused_relationship_syntactic() {
        let update = BmaUpdateFunction::try_from("1").unwrap();
        let variable = BmaVariable::new(0, "v1", (0, 3), Some(update));
        let mut network = network_for_variable(&variable);
        network
            .relationships
            .push(BmaRelationship::new_activator(0, 0, 0));

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![UpdateFunctionRegulatorInvalid {
                id: 0,
                regulator: 0,
                source: RegulatorErrorType::UnusedRelationship,
            },]
        );
    }

    #[test]
    fn unused_relationship_semantic() {
        let update = BmaUpdateFunction::try_from("var(0) - var(0)").unwrap();
        let variable = BmaVariable::new(0, "v1", (0, 3), Some(update));
        let mut network = network_for_variable(&variable);
        network
            .relationships
            .push(BmaRelationship::new_activator(0, 0, 0));

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![UpdateFunctionRegulatorInvalid {
                id: 0,
                regulator: 0,
                source: RegulatorErrorType::UnusedRelationship,
            },]
        );
    }

    #[test]
    fn inverted_monotonicity() {
        let update = BmaUpdateFunction::try_from("var(0)").unwrap();
        let variable = BmaVariable::new(0, "v1", (0, 3), Some(update));
        let mut network = network_for_variable(&variable);
        network
            .relationships
            .push(BmaRelationship::new_inhibitor(0, 0, 0));

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![UpdateFunctionRegulatorInvalid {
                id: 0,
                regulator: 0,
                source: RegulatorErrorType::BadMonotonicity {
                    declared: vec![Inhibitor],
                    observed: vec![Activator],
                },
            },]
        );
    }

    #[test]
    fn dual_monotonicity() {
        // Basically an XOR on integer domains:
        let update =
            BmaUpdateFunction::try_from("max(var(0), var(1)) - min(var(0), var(1))").unwrap();
        let variable = BmaVariable::new(0, "v1", (0, 3), Some(update));
        let variable_2 = BmaVariable::new(1, "v2", (0, 3), None);
        let mut network = network_for_variable(&variable);
        network.variables.push(variable_2);
        network
            .relationships
            .push(BmaRelationship::new_inhibitor(0, 0, 0));
        network
            .relationships
            .push(BmaRelationship::new_inhibitor(1, 1, 0));
        network
            .relationships
            .push(BmaRelationship::new_activator(1, 1, 0));

        let issues = variable.validate(&network).unwrap_err();
        assert_eq!(
            issues,
            vec![UpdateFunctionRegulatorInvalid {
                id: 0,
                regulator: 0,
                source: RegulatorErrorType::BadMonotonicity {
                    declared: vec![Inhibitor],
                    observed: vec![Activator, Inhibitor],
                },
            },]
        );
    }
}
