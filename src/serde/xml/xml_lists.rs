//! A bunch of "list wrappers" that are used to correctly de-serialize
//! XML BMA in a type safe manner.

use crate::serde::xml::{XmlContainer, XmlRelationship, XmlVariable};
use serde::{Deserialize, Serialize};

/// Structure to deserialize XML info about container list. Just a wrapper
/// for actual containers list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct XmlContainers {
    #[serde(default, rename = "Container")]
    pub container: Vec<XmlContainer>,
}

/// Structure to deserialize XML info about variables list. Just a wrapper
/// for actual variables list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct XmlVariables {
    #[serde(default, rename = "Variable")]
    pub variable: Vec<XmlVariable>,
}

/// Structure to deserialize XML info about relationships list. Just a wrapper
/// for actual relationships list needed due to the weird xml structure...
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct XmlRelationships {
    #[serde(default, rename = "Relationship")]
    pub relationship: Vec<XmlRelationship>,
}
