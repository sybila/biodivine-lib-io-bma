use crate::serde::json::JsonBmaModel;
use crate::serde::xml::XmlBmaModel;
use crate::{
    BmaLayout, BmaLayoutError, BmaNetwork, BmaNetworkError, ContextualValidation, ErrorReporter,
    Validation,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use thiserror::Error;

/// Main structure with all the important parts of a BMA model.
/// We distinguish between three parts tracked in the BMA format:
/// - the functional part with all the variables and relationships ([`BmaNetwork`])
/// - the optional layout with positions of variables and containers ([`BmaLayout`])
/// - the additional optional data like a version and so on (`metadata`)
///
/// `BmaModel` instances can be created from JSON or XML versions of the BMA format.
/// You can use `from_json_str`, `from_xml_str` to create a model from a string.
/// For serialization to JSON, use custom methods `to_json_str` or `to_pretty_json_str`
/// (serialization into XML is currently not supported).
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
    pub fn to_bma_json(&self) -> anyhow::Result<String> {
        let model = JsonBmaModel::from(self.clone());
        let json = serde_json::to_string(&model)?;
        Ok(json)
    }

    /// Same as [`BmaModel::to_bma_json`], but using a human-readable JSON formatting.
    pub fn to_bma_json_pretty(&self) -> anyhow::Result<String> {
        let model = JsonBmaModel::from(self.clone());
        let json = serde_json::to_string_pretty(&model)?;
        Ok(json)
    }

    /// Create a new BMA model from a model string in the BMA JSON format.
    pub fn from_bma_json(json_str: &str) -> anyhow::Result<Self> {
        let json_model: JsonBmaModel = serde_json::from_str(json_str)?;
        let model = BmaModel::try_from(json_model)?;
        Ok(model)
    }

    /// Create a new BMA model from a model string in XML format.
    /// Internally, we use serde_xml_rs serialization into an intermediate `XmlBmaModel` structure.
    pub fn from_xml_str(xml_str: &str) -> Result<Self, String> {
        let xml_model: XmlBmaModel = serde_xml_rs::from_str(xml_str).map_err(|e| e.to_string())?;
        BmaModel::try_from(xml_model).map_err(|e| e.to_string())
    }

    /// Convert the `BmaModel` into a BMA compatible XML string.
    pub fn to_xml_string(&self) -> anyhow::Result<String> {
        let model = XmlBmaModel::from(self.clone());
        let xml = serde_xml_rs::to_string(&model)?;
        Ok(xml)
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
        BmaRelationship, BmaRelationshipError, BmaVariable, BmaVariableError, Validation,
    };
    use num_rational::Rational64;

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
            metadata: Default::default(),
        };
        assert!(model.validate().is_ok());
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
                description: Some("Lorem ipsum".to_string()),
                zoom_level: Some(Rational64::new(10, 3)),
                pan: None,
            },
            metadata: Default::default(),
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
}
