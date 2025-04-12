use crate::model::bma_model::*;
use crate::update_fn::bma_fn_tree::BmaFnUpdate;
use crate::update_fn::expression_enums::{AggregateFn, ArithOp};
use biodivine_lib_param_bn::{BooleanNetwork, Monotonicity, RegulatoryGraph};
use regex::Regex;
use std::collections::HashMap;

impl BmaModel {
    /// Generate a canonical name for a BMA variable by combining its ID and name.
    /// This canonical name will be used in a BooleanNetwork.
    fn canonical_var_name(var: &BmaVariable) -> String {
        // Regex that matches non-alphanumeric and non-underscore characters
        let re = Regex::new(r"[^0-9a-zA-Z_]").unwrap();
        let sanitized_name = re.replace_all(&var.name, "");
        format!("v_{}_{}", var.id, sanitized_name)
    }

    /// Convert BmaModel into a RegulatoryGraph instance.
    /// The graph will contain the same set of variables and regulations as this model.
    /// See [Self::canonical_var_name] for how we create variable names.
    ///
    // TODO: decide how to handle "doubled" regulations (of the same vs of different type)
    // TODO: for now, we ignore observability (making it `false` for all regulations)
    pub fn to_regulatory_graph(&self) -> Result<RegulatoryGraph, String> {
        let mut variables_map: HashMap<u32, String> = HashMap::new();
        for var in &self.model.variables {
            let inserted = variables_map.insert(var.id, BmaModel::canonical_var_name(var));
            if inserted.is_some() {
                return Err(format!("Variable ID {} is not unique.", var.id));
            }
        }
        let variables = variables_map.clone().into_values().collect();
        let mut graph = RegulatoryGraph::new(variables);

        // add regulations
        // TODO: decide how to handle "doubled" regulations and observability
        self.model
            .relationships
            .iter()
            .try_for_each(|relationship| {
                let regulator_id = relationship.from_variable;
                let target_id = relationship.to_variable;
                let regulator = variables_map
                    .get(&regulator_id)
                    .ok_or(format!("Regulator var {} does not exist.", regulator_id))?;
                let target = variables_map
                    .get(&target_id)
                    .ok_or(format!("Target var {} does not exist.", target_id))?;
                let monotonicity = Some(relationship.relationship_type.into());
                graph.add_regulation(regulator, target, false, monotonicity)
            })?;

        Ok(graph)
    }

    /// Convert BmaModel into a BooleanNetwork instance.
    ///
    /// The network will contain the same set of variables and regulations as this model.
    /// See [Self::canonical_var_name] for how we create variable names.
    /// The update functions are transformed using [BmaFnUpdate::to_update_fn].
    ///
    /// TODO: For now, we do not handle multi-valued models.
    pub fn to_boolean_network(&self) -> Result<BooleanNetwork, String> {
        if !self.is_boolean_model() {
            return Err(
                "Currently, converting multi-valued models into BNs is not supported.".to_string(),
            );
        }

        let graph = self.to_regulatory_graph()?;
        let mut bn = BooleanNetwork::new(graph);

        let mut max_levels = HashMap::new();
        for var in &self.model.variables {
            let var_name = BmaModel::canonical_var_name(var);
            max_levels.insert(var_name, var.range_to);
        }

        // add update functions
        for var in &self.model.variables {
            let var_name = BmaModel::canonical_var_name(var);
            let var_id = bn.as_graph().find_variable(&var_name).unwrap();

            if var.range_to == 0 {
                // We can have zero constants and we must deal with these accordingly.
                bn.add_string_update_function(&var_name, "false").unwrap()
            }

            if let Some(bma_formula) = var.formula.clone() {
                // todo: to_update_fn is not fully finished yet
                let update_fn = bma_formula.to_update_fn(&max_levels);
                bn.set_update_function(var_id, Some(update_fn))?;
            } else {
                // The formula is empty, which means we have to build a default one
                // the same way as BMA is doing this.
                // We then convert this default BMA expression to a logical formula.

                let regulators = bn.regulators(var_id);
                if regulators.is_empty() {
                    // This is an undetermined input, in which case we set it to zero,
                    // because that's what BMA does.
                    bn.add_string_update_function(&var_name, "false").unwrap()
                }

                // We build the default function the same way as BMA does.
                let mut positive = Vec::new();
                let mut negative = Vec::new();
                for regulator in regulators {
                    let regulator_name = bn.get_variable_name(regulator);
                    let reg = bn.as_graph().find_regulation(regulator, var_id).unwrap();
                    // BMA variables must be monotonic
                    match reg.monotonicity.unwrap() {
                        Monotonicity::Activation => positive.push(regulator_name),
                        Monotonicity::Inhibition => negative.push(regulator_name),
                    }
                }

                let p_avr = if !positive.is_empty() {
                    let p_args = positive
                        .iter()
                        .map(|x| BmaFnUpdate::mk_variable(x))
                        .collect();
                    BmaFnUpdate::mk_aggregation(AggregateFn::Avg, p_args)
                } else {
                    // This does not make much sense, because it means any variable with only negative
                    // regulators is ALWAYS a constant zero. But this is how BMA seems to be doing it, so
                    // that's what we are doing as well...
                    BmaFnUpdate::mk_constant(0)
                };
                let n_avr = if !negative.is_empty() {
                    let n_args = negative
                        .iter()
                        .map(|x| BmaFnUpdate::mk_variable(x))
                        .collect();
                    BmaFnUpdate::mk_aggregation(AggregateFn::Avg, n_args)
                } else {
                    BmaFnUpdate::mk_constant(0)
                };
                let default_bma_formula = BmaFnUpdate::mk_arithmetic(p_avr, n_avr, ArithOp::Minus);
                let update_fn = default_bma_formula.to_update_fn(&max_levels);
                bn.set_update_function(var_id, Some(update_fn))?;
            }
        }

        Ok(bn)
    }
}
