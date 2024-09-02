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
