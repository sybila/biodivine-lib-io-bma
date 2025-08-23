pub(crate) mod from_bn;
pub(crate) mod into_bn;

use crate::serde::json::JsonBmaModel;
use crate::serde::xml::XmlBmaModel;
use crate::{
    BmaLayout, BmaLayoutError, BmaNetwork, BmaNetworkError, ContextualValidation, ErrorReporter,
    RelationshipType, Validation,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Main structure with all the important parts of a BMA model.
/// We distinguish between three parts tracked in the BMA format:
/// - the functional part with all the variables and relationships ([`BmaNetwork`])
/// - the optional layout with positions of variables and containers ([`BmaLayout`])
/// - the additional optional data like a version and so on (`metadata`)
///
/// `BmaModel` instances can be created from JSON or XML versions of the BMA format.
/// You can use [`BmaModel::from_json_string`], [`BmaModel::from_xml_string`] to create a model
/// from a string. For serialization to JSON or XML, use custom methods
/// [`BmaModel::to_json_string`], [`BmaModel::to_json_string_pretty`], or
/// [`BmaModel::to_xml_string`].
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct BmaModel {
    /// Main data with variables and relationships.
    pub network: BmaNetwork,
    /// Layout information (variable positions, containers, ...).
    /// Layout can be empty, but it is recommended to provide it.
    pub layout: BmaLayout,
    /// Stores additional metadata like `biocheck_version` that is sometimes present in the XML.
    /// Metadata is usually empty.
    #[serde(flatten)]
    pub metadata: HashMap<String, String>,
}

impl BmaModel {
    /// Convert the `BmaModel` into a BMA compatible JSON string.
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&JsonBmaModel::from(self.clone()))
    }

    /// Same as [`BmaModel::to_json_string`], but using a human-readable JSON formatting.
    pub fn to_json_string_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&JsonBmaModel::from(self.clone()))
    }

    /// Create a new BMA model from a model string in the BMA JSON format.
    pub fn from_json_string(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<JsonBmaModel>(json_str).map(BmaModel::from)
    }

    /// Create a new BMA model from a model string in XML format.
    /// Internally, we use `serde_xml_rs` serialization into an intermediate `XmlBmaModel` structure.
    pub fn from_xml_string(xml_str: &str) -> Result<Self, serde_xml_rs::Error> {
        serde_xml_rs::from_str::<XmlBmaModel>(xml_str).map(BmaModel::from)
    }

    /// Convert the `BmaModel` into a BMA compatible XML string.
    pub fn to_xml_string(&self) -> Result<String, serde_xml_rs::Error> {
        serde_xml_rs::to_string(&XmlBmaModel::from(self.clone()))
    }

    /// Create a new BMA model with a given network, layout, and metadata.
    /// This is just a constructor wrapper, it does not check the validity of the model.
    #[must_use]
    pub fn new(network: BmaNetwork, layout: BmaLayout, metadata: HashMap<String, String>) -> Self {
        BmaModel {
            network,
            layout,
            metadata,
        }
    }

    /// Check if all variables in the model are Boolean (max level is 1).
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        self.get_max_var_level() <= 1
    }

    /// Get the maximum level across all variables in the BMA model.
    #[must_use]
    pub fn get_max_var_level(&self) -> u32 {
        let mut max_level = 0;
        self.network.variables.iter().for_each(|v| {
            // just in case, lets check both `range_from` and `range_to`
            max_level = max(max_level, max(v.min_level(), v.max_level()));
        });
        max_level
    }

    /// Get regulators of a particular variable, optionally filtered by regulator type.
    /// The regulators are represented by their IDs.
    ///
    /// If network validation passed successfully, you can assume that there is no
    /// [`RelationshipType::Unknown`] (i.e. every relationship is either an activator,
    /// or an inhibitor).
    #[must_use]
    pub fn get_regulators(
        &self,
        target_var: u32,
        relationship: &Option<RelationshipType>,
    ) -> HashSet<u32> {
        self.network
            .relationships
            .iter()
            .filter(|r| r.to_variable == target_var)
            .filter(|r| relationship.as_ref().is_none_or(|x| *x == r.r#type))
            .map(|r| r.from_variable)
            .collect()
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BmaModelError {
    #[error(transparent)]
    Network(#[from] BmaNetworkError),
    #[error(transparent)]
    Layout(#[from] BmaLayoutError),
}

impl Validation for BmaModel {
    type Error = BmaModelError;
    fn validate_all<R: ErrorReporter<Self::Error>>(&self, reporter: &mut R) {
        self.network.validate_all(&mut reporter.wrap());
        self.layout.validate_all(self, &mut reporter.wrap());
    }
}

#[cfg(test)]
mod tests {
    use crate::model::tests::{simple_layout, simple_network};
    use crate::{
        BmaLayout, BmaLayoutContainer, BmaLayoutContainerError, BmaLayoutError, BmaLayoutVariable,
        BmaLayoutVariableError, BmaModel, BmaModelError, BmaNetwork, BmaNetworkError,
        BmaRelationship, BmaRelationshipError, BmaVariable, BmaVariableError, RelationshipType,
        Validation,
    };
    use num_rational::Rational64;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn default_model_is_valid() {
        let model = BmaModel::default();
        assert!(model.validate().is_ok());
    }

    #[test]
    fn simple_model_is_valid() {
        let model = BmaModel {
            network: simple_network(),
            layout: simple_layout(),
            metadata: HashMap::default(),
        };
        model.validate().unwrap();
        assert!(!model.is_boolean());
        assert_eq!(model.get_max_var_level(), 3);
    }

    #[test]
    fn complex_error_example() {
        let model = BmaModel {
            network: BmaNetwork {
                name: String::default(),
                variables: vec![
                    BmaVariable::new_boolean(2, "var_A", None),
                    BmaVariable::new(3, "var_A", (3, 2), None),
                ],
                relationships: vec![
                    BmaRelationship::new_activator(4, 2, 3),
                    BmaRelationship::new_inhibitor(5, 2, 3),
                    BmaRelationship::new_inhibitor(5, 3, 2),
                    BmaRelationship::new_inhibitor(6, 3, 4),
                ],
            },
            layout: BmaLayout {
                variables: vec![
                    BmaLayoutVariable::new(2, "var_A", Some(7)),
                    BmaLayoutVariable::new(3, "var_B", Some(4)),
                ],
                containers: vec![
                    BmaLayoutContainer::new(4, "comp1"),
                    BmaLayoutContainer::new(4, "comp2"),
                ],
                description: "Lorem ipsum".to_string(),
                zoom_level: Some(Rational64::new(10, 3)),
                pan: None,
            },
            metadata: HashMap::default(),
        };

        let expected = vec![
            BmaModelError::Network(BmaNetworkError::Variable(BmaVariableError::RangeInvalid {
                id: 3,
                range: (3, 2),
            })),
            BmaModelError::Network(BmaNetworkError::Relationship(
                BmaRelationshipError::IdNotUnique { id: 5 },
            )),
            BmaModelError::Network(BmaNetworkError::Relationship(
                BmaRelationshipError::IdNotUnique { id: 5 },
            )),
            BmaModelError::Network(BmaNetworkError::Relationship(
                BmaRelationshipError::TargetVariableNotFound {
                    id: 6,
                    to_variable: 4,
                },
            )),
            BmaModelError::Layout(BmaLayoutError::Variable(
                BmaLayoutVariableError::ContainerNotFound {
                    id: 2,
                    container_id: 7,
                },
            )),
            BmaModelError::Layout(BmaLayoutError::Container(
                BmaLayoutContainerError::IdNotUnique { id: 4 },
            )),
            BmaModelError::Layout(BmaLayoutError::Container(
                BmaLayoutContainerError::IdNotUnique { id: 4 },
            )),
        ];

        let issues = model.validate().unwrap_err();
        assert_eq!(issues, expected);
    }

    #[test]
    fn get_regulators_returns_source_variable_ids() {
        let mut network = BmaNetwork::default();
        network
            .variables
            .push(BmaVariable::new_boolean(1, "var_A", None));
        network
            .variables
            .push(BmaVariable::new_boolean(2, "var_B", None));
        network
            .variables
            .push(BmaVariable::new_boolean(3, "var_C", None));
        // Suppose variable 1 inhibits variable 2, and variable 3 activates variable 2
        network
            .relationships
            .push(BmaRelationship::new_inhibitor(10, 1, 2));
        network
            .relationships
            .push(BmaRelationship::new_activator(10, 3, 2));
        let model = BmaModel {
            network,
            layout: Default::default(),
            metadata: Default::default(),
        };

        let regulators = model.get_regulators(2, &Some(RelationshipType::Activator));
        assert_eq!(regulators, HashSet::from_iter(vec![3]));
        let regulators = model.get_regulators(2, &Some(RelationshipType::Inhibitor));
        assert_eq!(regulators, HashSet::from_iter(vec![1]));
        let regulators = model.get_regulators(2, &None);
        assert_eq!(regulators, HashSet::from_iter(vec![1, 3]));
    }
}
