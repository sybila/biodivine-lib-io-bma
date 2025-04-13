use biodivine_lib_param_bn::Monotonicity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq)]
pub enum VariableType {
    #[default]
    Default,
    Constant,
    MembraneReceptor,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
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

impl From<Monotonicity> for RelationshipType {
    fn from(val: Monotonicity) -> Self {
        match val {
            Monotonicity::Activation => RelationshipType::Activator,
            Monotonicity::Inhibition => RelationshipType::Inhibitor,
        }
    }
}
