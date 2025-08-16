/// Basic BMA model methods, including converting from BN instances.
mod _impl_bma_model;
/// Converting a BMA model into a regulatory graph and BN.
mod _impl_to_bn;
pub(crate) mod bma_model;
pub(crate) mod bma_network;
pub(crate) mod bma_relationship;
pub(crate) mod bma_variable;
pub(crate) mod layout;

#[cfg(test)]
mod tests {
    use crate::{
        BmaLayout, BmaLayoutContainer, BmaLayoutVariable, BmaNetwork, BmaRelationship, BmaVariable,
    };
    use num_rational::Rational64;

    pub fn simple_network() -> BmaNetwork {
        BmaNetwork {
            name: "Some network".to_string(),
            variables: vec![
                BmaVariable::new_boolean(3, "var_B", None),
                BmaVariable::new(0, "var_A", (1, 3), None),
            ],
            relationships: vec![
                BmaRelationship::new_activator(0, 0, 3),
                BmaRelationship::new_inhibitor(1, 3, 3),
            ],
            ..Default::default()
        }
    }

    pub fn simple_layout() -> BmaLayout {
        BmaLayout {
            variables: vec![
                BmaLayoutVariable::new(0, "l_var_A", None),
                BmaLayoutVariable::new(3, "l_var_B", Some(13)),
            ],
            containers: vec![BmaLayoutContainer::new(13, "Test container")],
            description: "Lorem ipsum".to_string(),
            zoom_level: Some(Rational64::new(1, 3)),
            pan: Some((Rational64::from(3), Rational64::from(10))),
        }
    }
}
