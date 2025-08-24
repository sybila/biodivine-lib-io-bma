use crate::{BmaModel, BmaVariable};
use anyhow::anyhow;
use biodivine_lib_bdd::{BddPartialValuation, BddVariable, BddVariableSet};
use biodivine_lib_param_bn::{FnUpdate, VariableId};
use std::collections::{BTreeMap, HashMap};

impl BmaModel {
    /// Convert the function of the given `target_var` into AEON update function.
    ///
    /// **The function must be defined and all of its inputs must be Boolean variables.**
    pub(crate) fn convert_function_to_aeon<'a>(
        &'a self,
        target_var: &'a BmaVariable,
        bma_id_to_aeon_id: &HashMap<u32, VariableId>,
    ) -> anyhow::Result<FnUpdate> {
        fn binarize(value: u32) -> anyhow::Result<bool> {
            if value == 0 {
                Ok(false)
            } else if value == 1 {
                Ok(true)
            } else {
                Err(anyhow!("Value is not binary"))
            }
        }

        // Step 1: Construct the function table and the other related helper structures.

        let Some(function) = &target_var.formula else {
            return Err(anyhow!("No update function found for `{}`", target_var.id));
        };

        let function = function.as_ref().map_err(|e| anyhow!(e.to_string()))?;

        let mut regulators_map = BTreeMap::new();
        for id in function.collect_variables() {
            let var = self
                .network
                .find_variable(id)
                .ok_or_else(|| anyhow!("Regulator variable `{id}` not found"))?;
            regulators_map.insert(id, var);
        }

        let table = target_var.build_function_table(function, &regulators_map)?;

        // Step 2: Build a symbolic context for representing the update function and a "nice",
        // optimized DNF function.

        let regulator_count = u16::try_from(regulators_map.len())
            .expect("Invariant violation: Regulator count exceeds 16 bits.");
        let bdd_ctx = BddVariableSet::new_anonymous(regulator_count);

        // Since the keys are sorted, we can assume they implicitly correspond to BDD variables.
        let mut bdd_id_to_aeon_id = HashMap::new();
        for (i, bma_id) in regulators_map.keys().enumerate() {
            let aeon_id = bma_id_to_aeon_id
                .get(bma_id)
                .ok_or_else(|| anyhow!("Missing AEON variable for BMA id `{}`", bma_id))?;
            let bdd_id = BddVariable::from_index(i);
            bdd_id_to_aeon_id.insert(bdd_id, *aeon_id);
        }

        let mut dnf_clauses = Vec::new();
        for (input, output) in table {
            if binarize(output)? {
                let mut clause = BddPartialValuation::empty();
                for (i, (_var, value)) in input.iter().enumerate() {
                    let bdd_var = BddVariable::from_index(i);
                    clause.set_value(bdd_var, binarize(*value)?);
                }
                dnf_clauses.push(clause);
            }
        }

        let dnf_bdd = bdd_ctx.mk_dnf(&dnf_clauses);
        let optimized_dnf = dnf_bdd.to_optimized_dnf();

        // Step 3: Convert the optimized DNF into an AEON update function

        let mut aeon_clauses = Vec::new();
        for bdd_clause in optimized_dnf {
            let mut aeon_clause = Vec::new();
            for (bdd_var, value) in bdd_clause.to_values() {
                let aeon_var = bdd_id_to_aeon_id
                    .get(&bdd_var)
                    .expect("Invariant violation: BDD variable does not map to AEON variable");
                let var_fn = FnUpdate::mk_var(*aeon_var);
                if value {
                    aeon_clause.push(var_fn);
                } else {
                    aeon_clause.push(FnUpdate::mk_not(var_fn));
                }
            }
            aeon_clauses.push(FnUpdate::mk_conjunction(&aeon_clause));
        }

        Ok(FnUpdate::mk_disjunction(&aeon_clauses))
    }
}

#[cfg(test)]
mod tests {
    use crate::update_function::tests::{and_model, complex_model};
    use biodivine_lib_param_bn::{BooleanNetwork, VariableId};
    use std::collections::HashMap;

    #[test]
    fn test_to_update_fn_boolean_binary() {
        let model = and_model();

        let var = model.network.find_variable(1).unwrap();

        let id_map = HashMap::from([
            (1, VariableId::from_index(0)),
            (2, VariableId::from_index(1)),
        ]);

        let result_fn = model.convert_function_to_aeon(var, &id_map).unwrap();

        let expected_bn = BooleanNetwork::try_from_bnet(
            r#"
            a, a & b
            b, 0
        "#,
        )
        .unwrap();

        assert_eq!(
            result_fn,
            expected_bn
                .get_update_function(VariableId::from_index(0))
                .clone()
                .unwrap()
        );
    }

    #[test]
    fn test_to_update_fn_boolean_ternary() {
        let model = complex_model();

        let var = model.network.find_variable(1).unwrap();

        let id_map = HashMap::from([
            (1, VariableId::from_index(0)),
            (2, VariableId::from_index(1)),
            (3, VariableId::from_index(2)),
        ]);

        let result_fn = model.convert_function_to_aeon(var, &id_map).unwrap();

        // expected function values are [1, 0, 0, 0, 1, 1, 1, 1]

        let expected_bn = BooleanNetwork::try_from_bnet(
            r#"
            a, a | (!b & !c)
            b, 0
            c, 0
        "#,
        )
        .unwrap();

        assert_eq!(
            result_fn,
            expected_bn
                .get_update_function(VariableId::from_index(0))
                .clone()
                .unwrap()
        );
    }
}
