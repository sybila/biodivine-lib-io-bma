use crate::utils::is_unique_id;
use crate::{BmaLayout, ContextualValidation, ErrorReporter};
use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Layout information about a container.
///
/// Expected invariants (checked during validation):
///  - The `id` must be unique within the containers of this [`BmaLayout`].
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BmaLayoutContainer {
    pub id: u32,
    pub name: String,
    pub size: u32,
    pub position: (Rational64, Rational64),
}

impl Default for BmaLayoutContainer {
    fn default() -> Self {
        BmaLayoutContainer {
            id: 0,
            name: String::default(),
            size: 1,
            position: (Default::default(), Default::default()),
        }
    }
}

impl BmaLayoutContainer {
    /// Create a new container using the given `id` and `name`. Remaining values use
    /// default values.
    pub fn new(id: u32, name: &str) -> Self {
        BmaLayoutContainer {
            id,
            name: name.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaLayoutContainerError {
    #[error("(Container: `{id}`) Id must be unique within `BmaLayout`")]
    IdNotUnique { id: u32 },
}

impl ContextualValidation<BmaLayout> for BmaLayoutContainer {
    type Error = BmaLayoutContainerError;

    fn validate_all<R: ErrorReporter<Self::Error>>(&self, context: &BmaLayout, reporter: &mut R) {
        // Ensure that the container id is unique within the enclosing BmaLayout.
        let Ok(is_unique) = is_unique_id(&context.containers, self, |x| x.id) else {
            panic!("Validation called on a container that is not part of the BmaLayout")
        };

        if !is_unique {
            reporter.report(BmaLayoutContainerError::IdNotUnique { id: self.id });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BmaLayout, BmaLayoutContainer, BmaLayoutContainerError, ContextualValidation};

    fn make_layout_for_container(container: &BmaLayoutContainer) -> BmaLayout {
        BmaLayout {
            containers: vec![container.clone()],
            ..Default::default()
        }
    }

    #[test]
    fn default_is_valid() {
        let container = BmaLayoutContainer::default();
        let layout = make_layout_for_container(&container);
        assert!(container.validate(&layout).is_ok());
    }

    #[test]
    fn blank_name() {
        let container = BmaLayoutContainer {
            name: "".to_string(),
            ..Default::default()
        };
        let layout = make_layout_for_container(&container);
        assert!(container.validate(&layout).is_ok());
    }

    #[test]
    fn unique_id() {
        let container = BmaLayoutContainer::default();
        let mut layout = make_layout_for_container(&container);
        layout.containers.push(container.clone());
        let issues = container.validate(&layout).unwrap_err();
        assert_eq!(issues, vec![BmaLayoutContainerError::IdNotUnique { id: 0 }]);
    }

    #[test]
    #[should_panic]
    fn missing_container() {
        let container = BmaLayoutContainer {
            name: "".to_string(),
            ..Default::default()
        };
        let mut layout = make_layout_for_container(&container);
        layout.containers.clear();
        container.validate(&layout).unwrap();
    }
}
