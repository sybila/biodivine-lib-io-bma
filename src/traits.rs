use serde::{Deserialize, Serialize};

/// Trait that provides methods to serialize and deserialize objects into (from) JSON,
/// utilizing [serde].
///
/// All of the structs implementing `JsonSerde` must implement traits `Serialize` and `Deserialize`.
pub trait JsonSerDe<'de>: Sized + Serialize + Deserialize<'de> {
    /// Wrapper for json serialization.
    fn to_json_str(&self) -> String;

    /// Wrapper for *pretty* json serialization with indentation.
    fn to_pretty_json_str(&self) -> String;

    /// Wrapper for json de-serialization.
    fn from_json_str(s: &'de str) -> Result<Self, String>;
}

/// Trait that provides method to deserialize objects from XML utilizing [serde].
///
/// All of the structs implementing `JsonSerde` must implement trait `Deserialize`.
pub trait XmlDe<'de>: Sized + Deserialize<'de> {
    /// Wrapper for xml de-serialization.
    fn from_xml_str(s: &'de str) -> Result<Self, String>;
}
