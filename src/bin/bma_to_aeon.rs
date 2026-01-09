use std::collections::{BTreeMap, HashMap};
use biodivine_lib_io_bma::{BmaLayoutVariable, BmaModel, BmaRelationship, BmaVariable, RelationshipType, VariableType};
use biodivine_lib_param_bn::BooleanNetwork;
use std::io::{self, Read};
use biodivine_lib_io_bma::update_function::BmaUpdateFunction;

const INGEST: &str = r#"
FLT-ITD	0
TYK2	0
PDGFRA	0
FGFR1	0
PIM1 	max(3/4*var(FLT-ITD),var(TYK2),3/4*var(PDGFRA),var(FGFR1))
PIM2 	max(var(FLT-ITD),1/2*var(TYK2),1/2*var(PDGFRA))
BAD 	1/2*var(RSK)+1/2*var(PIM1)
EIF4B 	min(3,var(RSK)+1/2*var(PIM1))
EIF3 	min(var(EIF4E),var(EIF4B))
4EBP1 	2/3*var(mTORC1)+1/6*var(EIF3)+1/6*var(PIM2)
EIF4E 	max(1/2*var(4EBP1),var(S6))
S6 	1/8*var(RSK)+3/4*var(mTORC1)+1/8*var(EIF3)
BCR 	max(1, 1/2*var(FLT-ITD))
GRB2-SOS 	max(var(BCR),1/2*var(FGFR1))
RAS 	min(var(GRB2-SOS),var(BCR))
PI3K 	max(var(BCR),var(GRB2-SOS),1/2*var(PDGFRA))
RAF 	avg(var(RAS))
MEK 	avg(var(RAF))
ERK 	max(var(MEK),1/2*var(PDGFRA))
RSK 	avg(var(ERK))
AKT 	max(var(PI3K),var(mTORC2))
mTORC2 	avg(var(PI3K))
mTORC1 	1/2*var(PRAS40)+var(TSC2)
TSC2 	1/2*((var(PIM2)-1)+1/2*var(AKT))
PRAS40 	1/4*var(PIM1)+5/4*var(AKT)
CHK 	max(var(PIM1),var(PIM2))
H3 	avg(var(CHK))
cMYC 	max(3/4*var(FGFR1),max(1,1/4*(max(var(PIM1),var(PIM2)) + var(H3))))
P27 	max(1,var(cMYC)*(var(cMYC)-2)+1/2*max(var(PIM1),var(PIM2)))
Proliferation 	(var(EIF4B)-2)+1/2*var(ERK)+2/3*var(P27)+2/3*var(cMYC)
Apoptosis 	1-max(var(BAD), var(S6), 1/2*var(BAD) + var(cMYC) + var(S6) + 2*var(EIF4E))
"#;

fn main() {

    let mut var_map = BTreeMap::new();
    let mut expr_map = BTreeMap::new();
    let mut next_id = 0u32;
    for line in INGEST.lines() {
        if line.trim().is_empty() {
            continue;
        }

        next_id += 1;
        let mut line = line.split("\t");
        let name = line.next().unwrap().trim();
        let function = line.next().unwrap().trim();
        assert!(line.next().is_none());

        var_map.insert(next_id, name);
        expr_map.insert(next_id, function);
    }

    let var_hint = var_map.iter().map(|(a, b)| (*a, b.to_string())).collect::<Vec<_>>();

    let mut model = BmaModel::default();
    for (id, expr) in &expr_map {
        let parsed = BmaUpdateFunction::parse_with_hint(expr, &var_hint).unwrap();

        let regulators = parsed.collect_variables();

        model.network.variables.push(BmaVariable {
            id: *id,
            name: var_map.get(id).unwrap().to_string(),
            range: (0, 1),
            formula: Some(Ok(parsed)),
        });

        model.layout.variables.push(BmaLayoutVariable {
            id: *id,
            container_id: None,
            r#type: VariableType::Default,
            name: var_map.get(id).unwrap().to_string(),
            description: String::default(),
            position: (Default::default(), Default::default()),
            angle: Default::default(),
            cell: None,
        });

        for reg in regulators {
            next_id += 1;
            model.network.relationships.push(BmaRelationship {
                id: next_id,
                from_variable: reg,
                to_variable: *id,
                r#type: RelationshipType::Activator,
            })
        }
    }

    println!("{}", model.to_json_string_pretty().unwrap());

    panic!();

    // Read from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");

    // Try to parse as BMA JSON first, then XML
    let bma_model = BmaModel::from_json_string(&input)
        //.or_else(|_| BmaModel::from_xml_string(&input))
        .expect("Failed to parse BMA format (tried both JSON and XML)");

    // Convert BmaModel to BooleanNetwork
    let bn = BooleanNetwork::try_from(&bma_model)
        .and_then(|bn| bn.infer_valid_graph().map_err(|e| anyhow::anyhow!("{}", e)))
        .expect("Failed to convert BMA model to BooleanNetwork");

    // Output as AEON format (BooleanNetwork implements Display)
    println!("{}", bn);
}
