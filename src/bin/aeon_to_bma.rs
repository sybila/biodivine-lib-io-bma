use biodivine_lib_io_bma::BmaModel;
use biodivine_lib_param_bn::BooleanNetwork;
use std::io::{self, Read};

fn main() {
    // Read from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");

    // Parse AEON format into BooleanNetwork
    let bn = BooleanNetwork::try_from(input.as_str()).expect("Failed to parse AEON format");

    // Convert BooleanNetwork to BmaModel
    let bma_model = BmaModel::try_from(&bn).expect("Failed to convert BooleanNetwork to BmaModel");

    // Output as BMA JSON format
    let output = bma_model
        .to_json_string()
        .expect("Failed to serialize BMA model to JSON");

    println!("{}", output);
}
