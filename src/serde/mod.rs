pub(crate) mod json;
pub(crate) mod xml;

pub(crate) mod quote_num;

#[cfg(test)]
mod tests {
    use crate::BmaModel;

    #[test]
    fn test_xml_models_have_no_errors() {
        // Go through all files in `xml-repo` and `xml-trap-mvn`, try to read them
        // as XML files and check that they deserialize without errors.
        for folder in &["./models/xml-repo", "./models/xml-trap-mvn"] {
            for file in std::fs::read_dir(folder).unwrap() {
                let file = file.unwrap();
                let file_name = file.file_name().to_str().unwrap().to_owned();
                if !file_name.ends_with(".xml") {
                    continue;
                }
                println!("File: {}/{}", folder, file_name);

                let xml_data = std::fs::read_to_string(file.path()).unwrap();
                BmaModel::from_xml_string(xml_data.as_str()).unwrap();
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
        for folder in &folders {
            for file in std::fs::read_dir(folder).unwrap() {
                let file = file.unwrap();
                let file_name = file.file_name().to_str().unwrap().to_owned();
                if !file_name.ends_with(".json") {
                    continue;
                }
                println!("File: {}/{}", folder, file_name);

                let json_data = std::fs::read_to_string(file.path()).unwrap();
                BmaModel::from_json_string(json_data.as_str()).unwrap();
            }
        }
    }
}
