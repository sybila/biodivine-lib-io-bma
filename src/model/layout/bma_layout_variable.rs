use crate::utils::is_unique_id;
use crate::{BmaModel, ContextualValidation, ErrorReporter};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::skip_serializing_none;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Additional layout information regarding a [`crate::BmaVariable`].
///
/// Expected invariants (checked during validation):
///  - The `id` must be unique within the layout variable IDs, and it must correspond to the
///    `id` of one [`crate::BmaVariable`] in the same model.
///  - If `container_id` is set, it must refer to an existing [`crate::BmaLayoutContainer`].
///  - If `description` is set, it must not be empty.
///
/// Note that variable `name` is also stored in [`crate::BmaVariable`]. Typically, these values
/// are the same, but this is not a verified invariant (i.e., in theory, you could use one name
/// for the variable, and another name for its layout counterpart).
///
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BmaLayoutVariable {
    pub id: u32,
    pub container_id: Option<u32>,
    pub r#type: VariableType,
    pub name: String,
    pub description: String,
    pub position: (Decimal, Decimal),
    pub angle: Decimal,
    pub cell: Option<(u32, u32)>,
}

impl BmaLayoutVariable {
    /// Create a layout variable for a [`crate::BmaVariable`] referenced by the given `id`
    /// and `name`, with an optional `container_id`.
    ///
    /// Remaining values are set to default.
    #[must_use]
    pub fn new(id: u32, name: &str, container_id: Option<u32>) -> Self {
        BmaLayoutVariable {
            id,
            container_id,
            name: name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum VariableType {
    #[default]
    Default,
    Constant,
    MembraneReceptor,
    Unknown(String),
}

impl From<&str> for VariableType {
    fn from(value: &str) -> Self {
        match value {
            "Default" => VariableType::Default,
            "Constant" => VariableType::Constant,
            "MembraneReceptor" => VariableType::MembraneReceptor,
            value => VariableType::Unknown(value.to_string()),
        }
    }
}

impl Display for VariableType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableType::Default => f.write_str("Default"),
            VariableType::Constant => write!(f, "Constant"),
            VariableType::MembraneReceptor => write!(f, "MembraneReceptor"),
            VariableType::Unknown(value) => write!(f, "{value}"),
        }
    }
}

impl Serialize for VariableType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for VariableType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = String::deserialize(deserializer)?;
        Ok(VariableType::from(values.as_str()))
    }
}

/// Possible validation errors for [`BmaLayoutVariable`].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaLayoutVariableError {
    #[error("(Layout var.: `{id}`) Id must be unique within `BmaLayout`")]
    IdNotUnique { id: u32 },
    #[error("(Layout var.: `{id}`) Variable not found in `BmaNetwork`")]
    VariableNotFound { id: u32 },
    #[error("(Layout var.: `{id}`) Container not found in `BmaLayout` with id `{container_id}`")]
    ContainerNotFound { id: u32, container_id: u32 },
    #[error("(Layout var.: `{id}`) Unknown variable type `{value}`")]
    UnknownVariableType { id: u32, value: String },
    #[error("(Layout var.: `{id}`) Variable type `{type}` is invalid: {message}")]
    InvalidVariableType {
        id: u32,
        r#type: VariableType,
        message: String,
    },
}

impl ContextualValidation<BmaModel> for BmaLayoutVariable {
    type Error = BmaLayoutVariableError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, context: &BmaModel, reporter: &mut R) {
        if let Some(bma_var) = context.network.find_variable(self.id) {
            // Ensure that constant variables have the correct type.
            let is_const = self.r#type == VariableType::Constant;
            let bma_is_const = bma_var.has_constant_range();
            if is_const && !bma_is_const {
                reporter.report(BmaLayoutVariableError::InvalidVariableType {
                    id: self.id,
                    r#type: self.r#type.clone(),
                    message: "Variable is not actually constant".to_string(),
                });
            }
            if bma_is_const && !is_const {
                reporter.report(BmaLayoutVariableError::InvalidVariableType {
                    id: self.id,
                    r#type: self.r#type.clone(),
                    message: "Variable is not declared as constant".to_string(),
                });
            }
        } else {
            // Ensure corresponding variable exists.
            reporter.report(BmaLayoutVariableError::VariableNotFound { id: self.id });
        }

        if let Some(container_id) = self.container_id
            && context.layout.find_container(container_id).is_none()
        {
            reporter.report(BmaLayoutVariableError::ContainerNotFound {
                id: self.id,
                container_id,
            });
        }

        // Ensure the item has a unique ID.
        let Ok(is_unique) = is_unique_id(&context.layout.variables, self, |x| x.id) else {
            // This is not a validation error; this violates the whole contract of the validation
            // mechanism and is therefore allowed to fail (instead of returning an error).
            panic!("Precondition violation: validated variable is not part of the `BmaLayout`.")
        };

        if !is_unique {
            reporter.report(BmaLayoutVariableError::IdNotUnique { id: self.id });
        }

        if let VariableType::Unknown(value) = &self.r#type {
            reporter.report(BmaLayoutVariableError::UnknownVariableType {
                id: self.id,
                value: value.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BmaLayout, BmaNetwork, BmaVariable};
    use std::collections::HashMap;

    fn make_model_for_variable(l_var: &BmaLayoutVariable) -> BmaModel {
        let n_var = BmaVariable {
            id: l_var.id,
            name: l_var.name.clone(),
            ..Default::default()
        };
        let network = BmaNetwork::new(vec![n_var], vec![]);
        let layout = BmaLayout {
            variables: vec![l_var.clone()],
            ..Default::default()
        };

        BmaModel {
            network,
            layout,
            metadata: HashMap::default(),
        }
    }

    #[test]
    fn default_variable_is_valid() {
        let l_var = BmaLayoutVariable::default();
        let model = make_model_for_variable(&l_var);
        assert!(l_var.validate(&model).is_ok());
    }

    #[test]
    fn blank_name() {
        let l_var = BmaLayoutVariable {
            name: String::default(),
            ..Default::default()
        };
        let model = make_model_for_variable(&l_var);
        assert!(l_var.validate(&model).is_ok());
    }

    #[test]
    fn blank_description() {
        let l_var = BmaLayoutVariable {
            description: String::default(),
            ..Default::default()
        };
        let model = make_model_for_variable(&l_var);
        assert!(l_var.validate(&model).is_ok());
    }

    #[test]
    #[should_panic]
    fn undefined_variable() {
        let mut l_var = BmaLayoutVariable {
            id: 5,
            ..Default::default()
        };
        let model = make_model_for_variable(&l_var);
        l_var.id = 3;
        // Here, validation should actually panic, because we are validating a variable
        // which does not belong to the model.
        let _ = l_var.validate(&model);
    }

    #[test]
    fn unknown_variable() {
        let l_var = BmaLayoutVariable {
            id: 5,
            ..Default::default()
        };
        let mut model = make_model_for_variable(&l_var);
        model.network.variables.clear();
        let issues = l_var.validate(&model).unwrap_err();
        assert_eq!(
            issues,
            vec![BmaLayoutVariableError::VariableNotFound { id: 5 }]
        );
    }

    #[test]
    fn unknown_container() {
        let l_var = BmaLayoutVariable {
            container_id: Some(5),
            ..Default::default()
        };
        let model = make_model_for_variable(&l_var);
        let issues = l_var.validate(&model).unwrap_err();
        assert_eq!(
            issues,
            vec![BmaLayoutVariableError::ContainerNotFound {
                id: 0,
                container_id: 5
            }]
        );
    }

    #[test]
    fn unique_id() {
        let l_var = BmaLayoutVariable {
            id: 5,
            ..Default::default()
        };
        let mut model = make_model_for_variable(&l_var);
        model.layout.variables.push(l_var.clone());
        let issues = l_var.validate(&model).unwrap_err();
        assert_eq!(issues, vec![BmaLayoutVariableError::IdNotUnique { id: 5 }]);
    }

    #[test]
    fn non_constant_variable_declaration() {
        let l_var = BmaLayoutVariable {
            id: 5,
            r#type: VariableType::Constant,
            ..Default::default()
        };
        let mut model = make_model_for_variable(&l_var);
        model.network.variables[0].range = (0, 4);
        let issues = l_var.validate(&model).unwrap_err();
        assert_eq!(
            issues,
            vec![BmaLayoutVariableError::InvalidVariableType {
                id: 5,
                r#type: VariableType::Constant,
                message: "Variable is not actually constant".to_string(),
            }]
        );
    }

    #[test]
    fn constant_variable_non_declaration() {
        let l_var = BmaLayoutVariable {
            id: 5,
            r#type: VariableType::MembraneReceptor,
            ..Default::default()
        };
        let mut model = make_model_for_variable(&l_var);
        model.network.variables[0].range = (4, 4);
        let issues = l_var.validate(&model).unwrap_err();
        assert_eq!(
            issues,
            vec![BmaLayoutVariableError::InvalidVariableType {
                id: 5,
                r#type: VariableType::MembraneReceptor,
                message: "Variable is not declared as constant".to_string(),
            }]
        );
    }
}
