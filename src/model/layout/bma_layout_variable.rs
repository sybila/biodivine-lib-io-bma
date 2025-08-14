use crate::utils::{is_blank, is_unique_id};
use crate::{BmaModel, ContextualValidation, ErrorReporter};
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use thiserror::Error;

/// Additional layout information regarding a [`crate::BmaVariable`].
///
/// Expected invariants (checked during validation):
///  - The `id` must be unique within the layout variable IDs, and it must correspond to the
///    `id` of one [`crate::BmaVariable`] in the same model.
///  - If `container_id` is set, it must refer to an existing [`crate::BmaLayoutContainer`].
///  - If `name` is set, it must not be empty.
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
    pub name: Option<String>,
    pub description: Option<String>,
    pub position: (Rational64, Rational64),
    pub angle: Rational64,
    pub cell: Option<(u32, u32)>,
}

impl BmaLayoutVariable {
    /// Create a layout variable for a [`crate::BmaVariable`] referenced by the given `id`
    /// and `name`, with an optional `container_id`.
    ///
    /// Remaining values are set to default.
    pub fn new(id: u32, name: &str, container_id: Option<u32>) -> Self {
        BmaLayoutVariable {
            id,
            container_id,
            name: Some(name.to_string()),
            ..Default::default()
        }
    }

    /// Clone the variable name or create a default alternative (`v_ID`).
    pub fn name_or_default(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("v_{}", self.id))
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableType {
    #[default]
    Default,
    Constant,
    MembraneReceptor,
}

/// Possible validation errors for [BmaLayoutVariable].
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaLayoutVariableError {
    #[error("(Layout var.: `{id}`) Id must be unique within `BmaLayout`")]
    IdNotUnique { id: u32 },
    #[error("(Layout var.: `{id}`) Variable not found in `BmaNetwork`")]
    VariableNotFound { id: u32 },
    #[error("(Layout var.: `{id}`) Container not found in `BmaLayout` with id `{container_id}`")]
    ContainerNotFound { id: u32, container_id: u32 },
    #[error("(Layout var.: `{id}`) Name cannot be empty; use `None` instead")]
    NameEmpty { id: u32 },
    #[error("(Layout var.: `{id}`) Description cannot be empty; use `None` instead")]
    DescriptionEmpty { id: u32 },
}

impl ContextualValidation<BmaModel> for BmaLayoutVariable {
    type Error = BmaLayoutVariableError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, context: &BmaModel, reporter: &mut R) {
        // Ensure serde fields are not empty.
        if is_blank(&self.name) {
            reporter.report(BmaLayoutVariableError::NameEmpty { id: self.id });
        }
        if is_blank(&self.description) {
            reporter.report(BmaLayoutVariableError::DescriptionEmpty { id: self.id });
        }

        // Ensure referenced IDs exist.
        if context.network.find_variable(self.id).is_none() {
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
            panic!("Validation called on a variable that is not part of the BmaLayout")
        };

        if !is_unique {
            reporter.report(BmaLayoutVariableError::IdNotUnique { id: self.id });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BmaLayout, BmaNetwork, BmaVariable};

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
            metadata: Default::default(),
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
            name: Some("".to_string()),
            ..Default::default()
        };
        let model = make_model_for_variable(&l_var);
        let issues = l_var.validate(&model).unwrap_err();
        assert_eq!(issues, vec![BmaLayoutVariableError::NameEmpty { id: 0 }]);
    }

    #[test]
    fn blank_description() {
        let l_var = BmaLayoutVariable {
            description: Some("".to_string()),
            ..Default::default()
        };
        let model = make_model_for_variable(&l_var);
        let issues = l_var.validate(&model).unwrap_err();
        assert_eq!(
            issues,
            vec![BmaLayoutVariableError::DescriptionEmpty { id: 0 }]
        );
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
        l_var.validate(&model).unwrap();
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
}
