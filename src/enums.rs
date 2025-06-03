use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq)]
pub enum VariableType {
    #[default]
    Default,
    Constant,
    MembraneReceptor,
}
