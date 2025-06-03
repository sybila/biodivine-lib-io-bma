use num_rational::Rational64;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Additional layout information regarding a [`crate::BmaVariable`].
///
/// Expected invariants (checked during validation):
///  - The `id` must be unique within the layout variable IDs, but is unrelated to the ID of
///    the associated [`crate::BmaVariable`].
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
    /// Create a default layout for a variable with a given name and ID.
    /// Container ID is optional and can be set to `None`.
    ///
    /// The default position is `(0, 0)`, the angle is `0.0`, and the cell / description is empty.
    /// Cell values are set to `None`.
    pub fn new_default(id: u32, name: String, container_id: Option<u32>) -> Self {
        BmaLayoutVariable {
            id,
            container_id,
            name: Some(name),
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
