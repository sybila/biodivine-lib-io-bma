use crate::update_function::BmaUpdateFunction;
use crate::update_function::expression_enums::ArithOp;
use ArithOp::{Minus, Mult, Plus};
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

        Ok(Self::try_from_fn_update_rec(fn_update))
    }

    /// Recursively converts the [`FnUpdate`] Boolean formula into a corresponding [`BmaFnUpdate`]
    /// real-number expression.
    ///
    /// Precondition: The `fn_update` object cannot contain parameters.
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

                match op {
                    // AND: map A && B to A * B
                    BinaryOp::And => {
                        BmaUpdateFunction::mk_arithmetic(Mult, &left_expr, &right_expr)
                    }
                    // OR: map A || B to A + B - A * B
                    BinaryOp::Or => {
                        let sum_expr =
                            BmaUpdateFunction::mk_arithmetic(Plus, &left_expr, &right_expr);
                        let prod_expr =
                            BmaUpdateFunction::mk_arithmetic(Mult, &left_expr, &right_expr);
                        BmaUpdateFunction::mk_arithmetic(Minus, &sum_expr, &prod_expr)
                    }
                    // XOR: map A ^ B to A + B - 2 * (A * B)
                    BinaryOp::Xor => {
                        let sum_expr =
                            BmaUpdateFunction::mk_arithmetic(Plus, &left_expr, &right_expr);
                        let prod_expr =
                            BmaUpdateFunction::mk_arithmetic(Mult, &left_expr, &right_expr);
                        let two = BmaUpdateFunction::mk_constant(2);
                        let two_prod_expr =
                            BmaUpdateFunction::mk_arithmetic(Mult, &two, &prod_expr);
                        BmaUpdateFunction::mk_arithmetic(Minus, &sum_expr, &two_prod_expr)
                    }
                    // IFF: map A <-> B to 1 - (A ^ B)
                    BinaryOp::Iff => {
                        // This is not very efficient, but IFF is a very rare operator...
                        let xor = FnUpdate::Binary(BinaryOp::Xor, left.clone(), right.clone());
                        BmaUpdateFunction::try_from_fn_update_rec(&FnUpdate::mk_not(xor))
                    }
                    // IMP: map A -> B to 1 - A + A * B
                    BinaryOp::Imp => {
                        let not_a = FnUpdate::Not(left.clone());
                        let not_left_expr = BmaUpdateFunction::try_from_fn_update_rec(&not_a);
                        let prod_expr =
                            BmaUpdateFunction::mk_arithmetic(Mult, &left_expr, &right_expr);
                        BmaUpdateFunction::mk_arithmetic(Plus, &not_left_expr, &prod_expr)
                    }
                }
            }
            FnUpdate::Param(_, _) => {
                panic!("Precondition violated: `FnUpdate` cannot contain parameters.")
            }
        }
    }
}
