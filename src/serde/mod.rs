pub(crate) mod json;
pub(crate) mod xml;

pub(crate) mod quote_num;

#[cfg(test)]
mod tests {
    use crate::{BmaModel, Validation};
    use std::collections::HashMap;

    fn xml_model_error_count() -> HashMap<&'static str, usize> {
        // For the most part, we have manually validated that these errors are "legit".
        HashMap::from_iter([
            ("./models/xml-repo/SSkin1D_TF.xml", 28),
            ("./models/xml-repo/SSkin2D_3cells_2layers.xml", 30),
            ("./models/xml-repo/SSkin1D.xml", 25),
            ("./models/xml-repo/Skin1D_TF_analysis_hang.xml", 31),
            ("./models/xml-repo/Skin2D_3cells_2layers.xml", 33),
            ("./models/xml-repo/Skin2D_5X2_TF.xml", 103),
            (
                "./models/xml-repo/Skin2D_3cells_2layers_TF_analysis_crash.xml",
                45,
            ),
            ("./models/xml-repo/2var_unstable.xml", 0),
            ("./models/xml-repo/VPC_lin15ko.xml", 9),
            ("./models/xml-repo/BooleanLoop.xml", 1),
            ("./models/xml-repo/VerySmallTestCase.xml", 1),
            ("./models/xml-repo/Skin1D.xml", 25),
            ("./models/xml-repo/Skin2D_5X2.xml", 103),
            ("./models/xml-repo/SSkin1D_analysis_hang.xml", 22),
            ("./models/xml-repo/NoLoopFound.xml", 1),
            ("./models/xml-repo/SmallTestCase.xml", 1),
            ("./models/xml-repo/Skin2D_3cells_2layers_TF.xml", 36),
            ("./models/xml-trap-mvn/Skin2D_5X2_TFAnalysisInput.xml", 65),
            ("./models/xml-trap-mvn/VPC_lin15koAnalysisInput.xml", 9),
            ("./models/xml-trap-mvn/MCP Array AnalysisInput.xml", 0),
            ("./models/xml-trap-mvn/XOR_Stable.xml", 2),
            (
                "./models/xml-trap-mvn/SSkin2D_3cells_2layersAnalysisInput.xml",
                6,
            ),
            ("./models/xml-trap-mvn/Skin1D_TFAnalysisInput.xml", 8),
            ("./models/xml-trap-mvn/Skin2D_5X2AnalysisInput.xml", 65),
            ("./models/xml-trap-mvn/Skin1DAnalysisInput.xml", 5),
            ("./models/xml-trap-mvn/SSkin1DAnalysisInput.xml", 5),
            (
                "./models/xml-trap-mvn/SSkin1D_analysis_hangAnalysisInput.xml",
                2,
            ),
            ("./models/xml-trap-mvn/2var_unstableAnalysisInput.xml", 0),
            ("./models/xml-trap-mvn/SSkin1D_TFAnalysisInput.xml", 8),
            (
                "./models/xml-trap-mvn/Skin2D_3cells_2layersAnalysisInput.xml",
                9,
            ),
            ("./models/xml-trap-mvn/BooleanLoopAnalysisInput.xml", 1),
            (
                "./models/xml-trap-mvn/Skin2D_3cells_2layers_TF_analysis_crashAnalysisInput.xml",
                25,
            ),
            (
                "./models/xml-trap-mvn/Skin1D_TF_analysis_hangAnalysisInput.xml",
                11,
            ),
            (
                "./models/xml-trap-mvn/Skin2D_3cells_2layers_TFAnalysisInput.xml",
                14,
            ),
            ("./models/xml-trap-mvn/E coli Tarfunc AnalysisInput.xml", 2),
        ])
    }

    fn json_model_error_count() -> HashMap<&'static str, usize> {
        // For the most part, we have manually validated that these errors are "legit".
        HashMap::from_iter([
            ("./models/json-repo/Skin1D.json", 5),
            ("./models/json-repo/ToyModelUnstable.json", 0),
            ("./models/json-repo/RestingNeuron.json", 25),
            ("./models/json-repo/SkinModel.json", 5),
            ("./models/json-repo/Race.json", 4),
            ("./models/json-repo/SimpleBifurcation.json", 0),
            ("./models/json-repo/ionChannel.json", 0),
            ("./models/json-repo/ToyModelStable.json", 1),
            ("./models/json-repo/ceilFunc.json", 5),
            (
                "./models/json-export-from-repo/Skin1D_TF_analysis_hang.json",
                25,
            ),
            ("./models/json-export-from-repo/Skin1D.json", 25),
            ("./models/json-export-from-repo/VerySmallTestCase.json", 1),
            ("./models/json-export-from-repo/VPC_lin15ko.json", 1),
            ("./models/json-export-from-repo/Resting Neuron.json", 25),
            (
                "./models/json-export-from-repo/SSkin2D_3cells_2layers.json",
                30,
            ),
            (
                "./models/json-export-from-repo/Skin2D_3cells_2layers_TF.json",
                30,
            ),
            ("./models/json-export-from-repo/SSkin1D_TF.json", 25),
            ("./models/json-export-from-repo/ToyModelUnstable.json", 0),
            ("./models/json-export-from-repo/SSkin1D (1).json", 25),
            ("./models/json-export-from-repo/SmallTestCase.json", 1),
            ("./models/json-export-from-repo/ion channel.json", 0),
            ("./models/json-export-from-repo/model 1.json", 4),
            ("./models/json-export-from-repo/E coli Tarfunc.json", 5),
            ("./models/json-export-from-repo/Default Model.json", 1),
            ("./models/json-export-from-repo/Skin1D (2).json", 25),
            ("./models/json-export-from-repo/2var_unstable (1).json", 0),
            ("./models/json-export-from-repo/Skin2D_5X2_TF.json", 103),
            ("./models/json-export-from-repo/SSkin1D.json", 25),
            (
                "./models/json-export-from-repo/Skin2D_3cells_2layers_TF_analysis_crash.json",
                33,
            ),
            ("./models/json-export-from-repo/model 1 (1).json", 0),
            ("./models/json-export-from-repo/New Model.json", 1),
            ("./models/json-export-from-repo/Skin2D_5X2.json", 103),
            (
                "./models/json-export-from-repo/SSkin1D_analysis_hang.json",
                22,
            ),
            ("./models/json-export-from-repo/Skin1D (1).json", 25),
            (
                "./models/json-export-from-repo/Skin2D_3cells_2layers.json",
                33,
            ),
            ("./models/json-export-from-repo/ToyModelStable.json", 1),
            (
                "./models/json-export-from-tool/Oscillatory negative feedback.json",
                0,
            ),
            ("./models/json-export-from-tool/Metabolism demo.json", 76),
            ("./models/json-export-from-tool/CancerSignalling.json", 11),
            ("./models/json-export-from-tool/Sigmoidal.json", 0),
            ("./models/json-export-from-tool/ToyModelUnstable.json", 0),
            (
                "./models/json-export-from-tool/Activator-Inhibitor Oscillation.json",
                0,
            ),
            ("./models/json-export-from-tool/Perfect Adaptation.json", 0),
            ("./models/json-export-from-tool/SkinModel.json", 25),
            (
                "./models/json-export-from-tool/Substrate depletion oscillations.json",
                1,
            ),
            ("./models/json-export-from-tool/Mutual Inhibition.json", 0),
            ("./models/json-export-from-tool/VPC.json", 1),
            ("./models/json-export-from-tool/Linear.json", 0),
            ("./models/json-export-from-tool/Leukaemia.json", 20),
            ("./models/json-export-from-tool/Homeostasis.json", 0),
            ("./models/json-export-from-tool/Hyperbolic.json", 0),
            ("./models/json-export-from-tool/ToyModelStable.json", 1),
        ])
    }

    #[test]
    fn test_xml_models_have_no_errors() {
        // Go through all files in `xml-repo` and `xml-trap-mvn`, try to read them
        // as XML files and check that they deserialize without errors.

        let expected = xml_model_error_count();
        for folder in &["./models/xml-repo", "./models/xml-trap-mvn"] {
            for file in std::fs::read_dir(folder).unwrap() {
                let file = file.unwrap();
                let file_name = file.file_name().to_str().unwrap().to_owned();
                if !file_name.ends_with(".xml") {
                    continue;
                }
                println!("File: {}/{}", folder, file_name);

                // XML Models have a lot of validation issues. So we are fine with the
                // validation failing, as long as the errors do look reasonable based on
                // manual inspection.
                let xml_data = std::fs::read_to_string(file.path()).unwrap();
                let model = BmaModel::from_xml_string(xml_data.as_str()).unwrap();
                let path = format!("{}/{}", folder, file_name);
                validate_model(path.as_str(), &model, &expected);
            }
        }
    }

    #[test]
    fn test_json_models_have_no_errors() {
        let folders = [
            "./models/json-repo",
            "./models/json-export-from-repo",
            "./models/json-export-from-tool",
        ];

        let expected = json_model_error_count();
        for folder in &folders {
            for file in std::fs::read_dir(folder).unwrap() {
                let file = file.unwrap();
                let file_name = file.file_name().to_str().unwrap().to_owned();
                if !file_name.ends_with(".json") {
                    continue;
                }
                println!("File: {}/{}", folder, file_name);

                // JSON Models have fewer validation issues, but still have a bunch of problems
                // related to duplicate or missing IDs.
                let json_data = std::fs::read_to_string(file.path()).unwrap();
                let model = BmaModel::from_json_string(json_data.as_str()).unwrap();
                let path = format!("{}/{}", folder, file_name);
                validate_model(path.as_str(), &model, &expected);
            }
        }
    }

    fn validate_model(path: &str, model: &BmaModel, expected: &HashMap<&'static str, usize>) {
        let errors = if let Err(errors) = model.validate() {
            println!("\tValidation errors: {}", errors.len());
            errors
        } else {
            println!("\tModel ok.");
            vec![]
        };

        let mut is_ok = false;
        for (p, c) in expected {
            if *p == path {
                if *c != errors.len() {
                    for e in &errors {
                        println!("\tError: {}", e);
                    }
                }
                assert_eq!(*c, errors.len());
                is_ok = true;
            }
        }
        if !is_ok {
            for e in &errors {
                println!("\tError: {}", e);
            }
        }
        assert!(is_ok);
    }
}
