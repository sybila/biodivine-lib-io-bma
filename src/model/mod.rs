/// Implementation of utilities, conversion from/to BooleanNetwork...
mod _impl_bma_model;
/// Implementation of JSON/XML serde traits (serialization implemented using intermediate structs).
mod _impl_serde;
/// Definition of the main struct `BmaModel` and its components.
mod bma_model;

pub use bma_model::BmaModel;
