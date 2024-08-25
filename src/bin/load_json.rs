use biodivine_lib_bma_data::json_model::JsonBmaModel;
use std::fs::read_to_string;

fn main() {
    let json_data =
        read_to_string("models/json/SimpleBifurcation.json").expect("Unable to read file");
    let resting_neuron: JsonBmaModel =
        serde_json::from_str(&json_data).expect("JSON was not well-formatted");
    println!("{:?}", resting_neuron);

    let json_data = read_to_string("models/json/RestingNeuron.json").expect("Unable to read file");
    let resting_neuron: JsonBmaModel =
        serde_json::from_str(&json_data).expect("JSON was not well-formatted");
    println!("{:?}", resting_neuron);
}
