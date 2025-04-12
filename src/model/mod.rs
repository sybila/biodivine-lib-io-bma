/// Implementation of utilities, conversion from/to BooleanNetwork...
mod _impl_bma_model;
/// Implementation of (de)serialization from/into JSON format.
mod _impl_json_serde;
/// Implementation of deserialization from XML format.
mod _impl_xml_serde;
/// Definition of the main struct `BmaModel` and its components.
mod bma_model;

pub use bma_model::BmaModel;
