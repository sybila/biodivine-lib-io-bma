mod bma_update_function;
mod expression_enums;
mod expression_node_data;

mod bma_expression_error;
mod bma_update_function_evaluation;
mod expression_default_builder;
mod expression_parser;
mod expression_token;
mod from_aeon;

pub use bma_update_function::BmaUpdateFunction;
pub use expression_enums::{AggregateFn, ArithOp, Literal, UnaryFn};
pub use expression_node_data::BmaExpressionNodeData;

pub use bma_expression_error::InvalidBmaExpression;
pub(crate) use bma_expression_error::ParserError;
pub(crate) use expression_default_builder::create_default_update_fn;

pub use bma_update_function_evaluation::FunctionTable;

#[cfg(test)]
mod tests {
    use crate::update_function::BmaUpdateFunction;
    use crate::{BmaModel, BmaNetwork, BmaRelationship, BmaVariable};

    /// Build a model with two variables, each having update function "a * b" (boolean AND on {0,1}).
    pub fn and_model() -> BmaModel {
        let expression = BmaUpdateFunction::try_from("var(1) * var(2)").unwrap();

        BmaModel {
            network: BmaNetwork {
                name: "".to_string(),
                variables: vec![
                    BmaVariable::new(1, "a", (0, 1), Some(expression.clone())),
                    BmaVariable::new(2, "b", (0, 1), Some(expression)),
                ],
                relationships: vec![
                    BmaRelationship::new_activator(100, 1, 1),
                    BmaRelationship::new_activator(101, 2, 1),
                    BmaRelationship::new_activator(102, 1, 2),
                    BmaRelationship::new_activator(103, 2, 2),
                ],
            },
            layout: Default::default(),
            metadata: Default::default(),
        }
    }

    /// Build a model with three variables, the first having a complex update function.
    pub fn complex_model() -> BmaModel {
        let expression =
            BmaUpdateFunction::try_from("var(1) + (1 - min((var(2) + var(3)), 1))").unwrap();

        BmaModel {
            network: BmaNetwork {
                name: "".to_string(),
                variables: vec![
                    BmaVariable::new(1, "a", (0, 1), Some(expression)),
                    BmaVariable::new(2, "b", (0, 1), None),
                    BmaVariable::new(3, "c", (0, 1), None),
                ],
                relationships: vec![
                    BmaRelationship::new_activator(100, 1, 1),
                    BmaRelationship::new_activator(101, 2, 1),
                    BmaRelationship::new_activator(102, 3, 1),
                ],
            },
            layout: Default::default(),
            metadata: Default::default(),
        }
    }
}
