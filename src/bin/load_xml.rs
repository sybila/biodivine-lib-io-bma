use biodivine_lib_bma_data::xml_model::XmlBmaModel;
use std::fs::read_to_string;

fn main() {
    let xml_data = read_to_string("models/xml/VerySmallTestCase.xml").expect("Unable to read file");
    let xml_model: XmlBmaModel =
        serde_xml_rs::from_str(&xml_data).expect("XML was not well-formatted");
    println!("{:?}", xml_model);

    let xml_data = read_to_string("models/xml/Skin1D.xml").expect("Unable to read file");
    let xml_model: XmlBmaModel =
        serde_xml_rs::from_str(&xml_data).expect("XML was not well-formatted");
    println!("{:?}", xml_model);
}
