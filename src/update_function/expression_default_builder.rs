use crate::update_function::{AggregateFn, ArithOp, BmaUpdateFunction};
use crate::{BmaNetwork, RelationshipType};
use std::collections::HashSet;

/// Create a default update function for a variable in the BMA model with
/// an originally empty formula.
///
/// This function is created the same way as BMA does it, even though that
/// can feel weird at times.
///
/// **WARNING**: Variables with only negative regulators will always evaluate to
/// constant zero due to BMA's averaging logic. This may not match biological
/// intuition but maintains compatibility with BMA.
///
/// The function assumes every regulator relationship is either activation,
/// or inhibition. Unknown relationship types are ignored.
pub(crate) fn create_default_update_fn(model: &BmaNetwork, var_id: u32) -> BmaUpdateFunction {
    fn create_average(variables: &HashSet<u32>) -> BmaUpdateFunction {
        if variables.is_empty() {
            // This makes little sense because it means any variable with only negative
            // regulators is ALWAYS a constant zero. But this is how BMA seems to be doing it, so
            // that's what we are doing as well...
            BmaUpdateFunction::mk_constant(0)
        } else {
            let args = variables
                .iter()
                .map(|x| BmaUpdateFunction::mk_variable(*x))
                .collect::<Vec<_>>();
            BmaUpdateFunction::mk_aggregation(AggregateFn::Avg, &args)
        }
    }

    let positive = model.get_regulators(var_id, &Some(RelationshipType::Activator));
    let negative = model.get_regulators(var_id, &Some(RelationshipType::Inhibitor));
    if positive.is_empty() && negative.is_empty() {
        // This is an undetermined input, in which case we set it to zero,
        // because that's what BMA does.
        return BmaUpdateFunction::mk_constant(0);
    }

    // We build the default function the same way as BMA does.

    // We average the positive and negative regulators
    let p_avr = create_average(&positive);
    let n_avr = create_average(&negative);

    // Finally, we subtract the negative average from the positive average
    BmaUpdateFunction::mk_arithmetic(ArithOp::Minus, &p_avr, &n_avr)
}
