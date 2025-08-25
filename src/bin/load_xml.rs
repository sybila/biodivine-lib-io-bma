use biodivine_lib_io_bma::BmaModel;
use biodivine_lib_param_bn::BooleanNetwork;
use std::fs::{read_dir, read_to_string};

/// Iterate through all models and see if they are parse without error.
/// Results are printed, one line per model.
fn test_parse_all_models_in_dir(models_dir: &str) {
    let model_paths = read_dir(models_dir)
        .expect("Unable to read directory")
        .map(|entry| entry.expect("Unable to read entry").path())
        .collect::<Vec<_>>();

    // Iterate over each XML file and try to parse it
    for model_path in model_paths {
        let model_path_str = model_path.to_str().expect("Invalid path");
        let xml_data = read_to_string(&model_path)
            .unwrap_or_else(|_| panic!("Unable to read file: {}", model_path_str));

        let result_model = BmaModel::from_xml_string(&xml_data);
        match result_model {
            Ok(_) => {
                println!("Successfully parsed model `{model_path_str}`.");
            }
            Err(e) => {
                println!("Failed to parse XML file `{}`: {:?}.", model_path_str, e);
            }
        }
    }
    println!();
}

fn main() {
    // 1) first, let's just check the small example and print the internal structure
    let selected_model_paths = vec!["models/xml-repo/VerySmallTestCase.xml"];
    for model_path in selected_model_paths {
        println!("Parsing selected model {:?}:", model_path);
        let xml_data = read_to_string(model_path).expect("Unable to read file");
        let model = BmaModel::from_xml_string(&xml_data).expect("XML was not well-formatted");
        println!("Internal BmaModel structure:\n{:?}\n", model);
    }

    // 2) now let's iterate through all models and see if they at least parse without error
    test_parse_all_models_in_dir("models/xml-repo/");
    test_parse_all_models_in_dir("models/xml-trap-mvn/");

    // 3) first, let's just check fully converting a small boolean example
    let boolean_model_paths = vec!["models/xml-trap-mvn/BooleanLoopAnalysisInput.xml"];
    for model_path in boolean_model_paths {
        println!("Processing selected boolean model {:?}:", model_path);
        let xml_data = read_to_string(model_path).expect("Unable to read file");
        let bma_model = BmaModel::from_xml_string(&xml_data).expect("XML was not well-formatted");
        let bn = BooleanNetwork::try_from(&bma_model).expect("Failed to convert to BN");
        println!("Resulting BN:\n{bn}");
    }
}
