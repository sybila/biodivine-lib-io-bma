use crate::update_function::AggregateFn::{Max, Min};
use crate::update_function::BmaUpdateFunction;
use crate::update_function::expression_enums::ArithOp;
use ArithOp::Minus;
use anyhow::anyhow;
use biodivine_lib_param_bn::{BinaryOp, FnUpdate};

impl BmaUpdateFunction {
    /// Try to make a BMA expression from a [`FnUpdate`] instance.
    ///
    /// *The conversion assumes that variable IDs are identical between the [`FnUpdate`]
    /// and the resulting [`BmaUpdateFunction`].*
    ///
    /// Essentially, this function converts a Boolean formula into a corresponding arithmetic
    /// expression. Constants are converted to 0 or 1, and logical operators are replaced with
    /// their arithmetic equivalents (AND with multiplication, OR with addition, etc.).
    ///
    /// The operators are built such that the result is always guaranteed to be in the `[0,1]`
    ///  range, even though BMA will truncate values for us. This ensures the update logic
    ///  is translated faithfully.
    ///
    /// The update function cannot contain any parameters.
    pub fn try_from_fn_update(fn_update: &FnUpdate) -> anyhow::Result<Self> {
        // Update functions with function symbols are not allowed
        let parameters = fn_update.collect_parameters();
        if !parameters.is_empty() {
            return Err(anyhow!("Found unsupported parameters {parameters:?}"));
        }

        Ok(Self::try_from_fn_update_rec(
            &fn_update.to_and_or_normal_form(),
        ))
    }

    /// Recursively converts the [`FnUpdate`] Boolean formula into a corresponding [`BmaFnUpdate`]
    /// real-number expression.
    ///
    /// Precondition: The `fn_update` object cannot contain parameters and must be AND-OR
    /// normalized.
    pub(crate) fn try_from_fn_update_rec(fn_update: &FnUpdate) -> BmaUpdateFunction {
        match fn_update {
            FnUpdate::Const(val) => BmaUpdateFunction::mk_constant(i32::from(*val)),
            FnUpdate::Var(var_id) => {
                let var_id = u32::try_from(var_id.to_index())
                    .expect("Invariant violation: Variable index must fit into 32-bits.");
                BmaUpdateFunction::mk_variable(var_id)
            }
            FnUpdate::Not(child) => {
                // NOT: map !A to (1 - A)
                let child_expr = Self::try_from_fn_update_rec(child);
                let one_node = BmaUpdateFunction::mk_constant(1);
                BmaUpdateFunction::mk_arithmetic(Minus, &one_node, &child_expr)
            }
            FnUpdate::Binary(op, left, right) => {
                let left_expr = Self::try_from_fn_update_rec(left);
                let right_expr = Self::try_from_fn_update_rec(right);

                /*
                   In some sense, a slightly better translation system would be the following:
                       - A && B to A * B
                       - A || B to A + B - A * B
                       - A ^ B to A + B - 2 * (A * B)
                       - A <-> B to 1 - (A ^ B)
                       - A -> B to 1 - A + A * B

                    However, this creates excessive duplication of expressions, which
                    then bloats complicated models significantly. As such, we are currently
                    relying on the more common min/max translation of AND/OR operators.
                */

                match op {
                    // AND: map A && B to min(A, B)
                    BinaryOp::And => {
                        BmaUpdateFunction::mk_aggregation(Min, &[left_expr, right_expr])
                    }
                    // OR: map A || B to max(A, B)
                    BinaryOp::Or => {
                        BmaUpdateFunction::mk_aggregation(Max, &[left_expr, right_expr])
                    }
                    // XOR: map A ^ B to min(max(A, B), max(1 - A, 1 - B))
                    BinaryOp::Xor | BinaryOp::Iff | BinaryOp::Imp => {
                        panic!(
                            "Precondition violated: `FnUpdate` cannot contain xor/iff/imp operators."
                        )
                    }
                }
            }
            FnUpdate::Param(_, _) => {
                panic!("Precondition violated: `FnUpdate` cannot contain parameters.")
            }
        }
    }
}
