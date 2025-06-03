use crate::update_fn::bma_fn_update::BmaFnUpdate;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
#[cfg(test)]
mod tests {
    use crate::{BmaNetwork, BmaVariable, ContextualValidation};

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
}
