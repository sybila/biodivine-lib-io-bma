use biodivine_lib_param_bn::Monotonicity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum VariableType {
    Default,
    Constant,
    MembraneReceptor,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RelationshipType {
    Activator,
    Inhibitor,
}

impl From<RelationshipType> for Monotonicity {
    fn from(val: RelationshipType) -> Self {
        match val {
            RelationshipType::Activator => Monotonicity::Activation,
            RelationshipType::Inhibitor => Monotonicity::Inhibition,
        }
    }
}
