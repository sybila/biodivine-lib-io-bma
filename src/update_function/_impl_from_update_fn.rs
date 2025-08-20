use crate::update_function::bma_fn_update::BmaUpdateFunction;
use crate::update_function::expression_enums::ArithOp;
use biodivine_lib_param_bn::{BinaryOp, FnUpdate};

impl BmaUpdateFunction {
    /// Try to make a BMA expression from a `FnUpdate` instance (of [biodivine_lib_param_bn]).
    ///
    /// Essentially, this function converts a Boolean formula into a corresponding arithmetic
    /// expression. Constants are converted to 0 or 1, and logical operators are replaced with
    /// their arithmetic equivalents (AND with multiplication, OR with addition, etc.).
    ///
    /// The update function cannot contain any parameters.
    pub fn try_from_fn_update(fn_update: &FnUpdate) -> Result<Self, String> {
        // update functions with function symbols are not allowed
        if !fn_update.collect_parameters().is_empty() {
            return Err("Update function with function symbols can not be translated.".to_string());
        }
        Self::try_from_fn_update_rec(fn_update)
    }

    /// Recursively converts the `FnUpdate` Boolean formula into a corresponding `BmaFnUpdate`
    /// real-number expression.
    ///
    /// At the end, must ensure each result will fall into [0, 1] range. For example, when
    /// using + for logical OR, we need to subtract the product of both operands to avoid
    /// exceeding 1.
    fn try_from_fn_update_rec(fn_update: &FnUpdate) -> Result<BmaUpdateFunction, String> {
        let res = match fn_update {
            FnUpdate::Const(val) => BmaUpdateFunction::mk_constant(if *val { 1 } else { 0 }),
            FnUpdate::Var(var_id) => BmaUpdateFunction::mk_variable(var_id.to_index() as u32),
            FnUpdate::Not(child) => {
                // NOT: map !A to (1 - A)
                let child_expr = Self::try_from_fn_update_rec(child)?;
                let one_node = BmaUpdateFunction::mk_constant(1);
                BmaUpdateFunction::mk_arithmetic(one_node, child_expr, ArithOp::Minus)
            }
            FnUpdate::Binary(op, left, right) => {
                let left_expr = Self::try_from_fn_update_rec(left)?;
                let right_expr = Self::try_from_fn_update_rec(right)?;

                match op {
                    // AND: map A && B to A * B
                    BinaryOp::And => {
                        BmaUpdateFunction::mk_arithmetic(left_expr, right_expr, ArithOp::Mult)
                    }
                    // OR: map A || B to A + B - A * B
                    BinaryOp::Or => {
                        let sum_expr = BmaUpdateFunction::mk_arithmetic(
                            left_expr.clone(),
                            right_expr.clone(),
                            ArithOp::Plus,
                        );
                        let prod_expr =
                            BmaUpdateFunction::mk_arithmetic(left_expr, right_expr, ArithOp::Mult);
                        BmaUpdateFunction::mk_arithmetic(sum_expr, prod_expr, ArithOp::Minus)
                    }
                    // XOR: map A ^ B to A + B - 2 * (A * B)
                    BinaryOp::Xor => {
                        let sum_expr = BmaUpdateFunction::mk_arithmetic(
                            left_expr.clone(),
                            right_expr.clone(),
                            ArithOp::Plus,
                        );
                        let prod_expr =
                            BmaUpdateFunction::mk_arithmetic(left_expr, right_expr, ArithOp::Mult);
                        let two_prod_expr = BmaUpdateFunction::mk_arithmetic(
                            BmaUpdateFunction::mk_constant(2),
                            prod_expr,
                            ArithOp::Mult,
                        );
                        BmaUpdateFunction::mk_arithmetic(sum_expr, two_prod_expr, ArithOp::Minus)
                    }
                    // IFF: map A <-> B to 1 - (A ^ B)
                    BinaryOp::Iff => {
                        let xor_expr = BmaUpdateFunction::try_from_fn_update_rec(
                            &FnUpdate::Binary(BinaryOp::Xor, left.clone(), right.clone()),
                        )?;
                        let one_node = BmaUpdateFunction::mk_constant(1);
                        BmaUpdateFunction::mk_arithmetic(one_node, xor_expr, ArithOp::Minus)
                    }
                    // IMP: map A -> B to 1 - A + A * B
                    BinaryOp::Imp => {
                        let not_left_expr = BmaUpdateFunction::mk_arithmetic(
                            BmaUpdateFunction::mk_constant(1),
                            left_expr.clone(),
                            ArithOp::Minus,
                        );
                        let prod_expr =
                            BmaUpdateFunction::mk_arithmetic(left_expr, right_expr, ArithOp::Mult);
                        BmaUpdateFunction::mk_arithmetic(not_left_expr, prod_expr, ArithOp::Plus)
                    }
                }
            }
            _ => Err("Unsupported operator.")?,
        };
        Ok(res)
    }
}
