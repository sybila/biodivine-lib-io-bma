use num_rational::Rational64;
use serde::{Deserialize, Serialize};

/// Layout information about a container.
///
/// Expected invariants (checked during validation):
///  - The `id` must be unique within the containers of this [`crate::BmaLayout`].
///  - If `name` is set, it must not be empty.
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BmaLayoutContainer {
    pub id: u32,
    pub name: Option<String>,
    pub size: u32,
    pub position: (Rational64, Rational64),
}

impl Default for BmaLayoutContainer {
    fn default() -> Self {
        BmaLayoutContainer {
            id: 0,
            name: None,
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
            name: Some(name.to_string()),
            ..Default::default()
        }
    }
}
