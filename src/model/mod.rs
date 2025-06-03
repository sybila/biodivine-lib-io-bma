/// Basic BMA model methods, including coverting from BN instances.
mod _impl_bma_model;
/// Implementation of (de)serialization from/into JSON format.
mod _impl_json_serde;
/// Converting BMA model into a regulatory graph and BN.
mod _impl_to_bn;
/// Implementation of deserialization from XML format.
mod _impl_xml_serde;
/// Main BMA model struct and its components.
mod bma_model;

mod bma_network;
mod bma_relationship;
mod bma_variable;

pub use bma_model::BmaModel;
pub use bma_network::BmaNetwork;
pub use bma_relationship::BmaRelationship;
pub use bma_variable::BmaVariable;
