use biodivine_lib_bma_data::bma_model::BmaModel;
use biodivine_lib_bma_data::traits::JsonSerDe;
use std::fs::{read_dir, read_to_string};

/// Iterate through all models and see if they are parse without error.
/// Results are printed, one line per model.
fn test_parse_all_models_in_dir(models_dir: &str) {
    let model_paths = read_dir(models_dir)
        .expect("Unable to read directory")
        .map(|entry| entry.expect("Unable to read entry").path())
        .collect::<Vec<_>>();

    // Iterate over each JSON file and try to parse it
    for model_path in model_paths {
        let model_path_str = model_path.to_str().expect("Invalid path");
        let json_data = read_to_string(&model_path)
            .unwrap_or_else(|_| panic!("Unable to read file: {}", model_path_str));

        let result_model = BmaModel::from_json_str(&json_data);
        match result_model {
            Ok(bma_model) => {
                let result_bn = bma_model.to_boolean_network();
                match result_bn {
                    Ok(_) => {
                        println!("Successfully parsed and converted model: `{model_path_str}`.");
                    }
                    Err(e) => {
                        println!(
                            "Failed to convert model `{}` to BN: {:?}.",
                            model_path_str, e
                        );
                    }
                }

                println!("Successfully parsed and converted model: `{model_path_str}`.");
            }
            Err(e) => {
                println!("Failed to parse JSON file `{}`: {:?}.", model_path_str, e);
            }
        }
    }
    println!();
}

fn main() {
    // 1) first, let's just check the small example and print the internal structure
    let selected_model_paths = vec!["models/json-export-from-tool/ToyModelStable.json"];
    for model_path in selected_model_paths {
        println!("Parsing selected model {:?}:", model_path);
        let json_data = read_to_string(model_path).expect("Unable to read file");
        let model = BmaModel::from_json_str(&json_data).expect("JSON was not well-formatted");
        println!("Internal BmaModel structure:\n{:?}\n", model);
        println!(
            "Exported JSON BmaModel structure:\n{}\n",
            model.to_json_str()
        );
    }

    // 2) now let's iterate through all models and see if they at least parse without error
    test_parse_all_models_in_dir("models/json-repo/");
    test_parse_all_models_in_dir("models/json-export-from-repo/");
    test_parse_all_models_in_dir("models/json-export-from-tool/");
}
