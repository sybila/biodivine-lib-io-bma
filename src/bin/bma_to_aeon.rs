use biodivine_lib_io_bma::BmaModel;
use biodivine_lib_param_bn::BooleanNetwork;
use std::io::{self, Read};

fn main() {
    // Read from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");

    // Try to parse as BMA JSON first, then XML
    let bma_model = BmaModel::from_json_string(&input)
        .or_else(|_| BmaModel::from_xml_string(&input))
        .expect("Failed to parse BMA format (tried both JSON and XML)");

    // Convert BmaModel to BooleanNetwork
    let bn = BooleanNetwork::try_from(&bma_model)
        .and_then(|bn| bn.infer_valid_graph().map_err(|e| anyhow::anyhow!("{}", e)))
        .expect("Failed to convert BMA model to BooleanNetwork");

    // Output as AEON format (BooleanNetwork implements Display)
    println!("{}", bn);
}
