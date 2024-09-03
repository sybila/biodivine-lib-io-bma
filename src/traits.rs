use serde::{Deserialize, Serialize};

/// Trait that provides methods to serialize and deserialize objects into (from) JSON,
/// utilizing [serde].
///
/// All of the structs implementing `JsonSerde` must implement traits `Serialize` and `Deserialize`.
pub trait JsonSerde<'de>: Sized + Serialize + Deserialize<'de> {
    /// Wrapper for json serialization.
    fn to_json_str(&self) -> String;

    /// Wrapper for *pretty* json serialization with indentation.
    fn to_pretty_json_str(&self) -> String;

    /// Wrapper for json de-serialization.
    fn from_json_str(s: &'de str) -> Result<Self, String>;
}

/// Trait that provides methods to serialize and deserialize objects into (from) XML,
/// utilizing [serde].
///
/// All of the structs implementing `JsonSerde` must implement traits `Serialize` and `Deserialize`.
pub trait XmlSerde<'de>: Sized + Serialize + Deserialize<'de> {
    /// Wrapper for xml serialization.
    fn to_xml_str(&self) -> String;

    /// Wrapper for *pretty* xml serialization with indentation.
    fn to_pretty_xml_str(&self) -> String;

    /// Wrapper for xml de-serialization.
    fn from_xml_str(s: &'de str) -> Result<Self, String>;
}
