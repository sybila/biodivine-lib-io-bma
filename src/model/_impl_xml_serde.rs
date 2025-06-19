use crate::serde::xml::XmlVariable;
use crate::serde::xml_model::*;
use crate::{
    BmaLayout, BmaLayoutContainer, BmaLayoutVariable, BmaModel, BmaNetwork, BmaRelationship,
    BmaVariable,
};
use num_rational::Rational64;
use num_traits::FromPrimitive;
use std::collections::HashMap;

impl BmaModel {
    /// Create a new BMA model from a model string in XML format.
    /// Internally, we use serde_xml_rs serialization into an intermediate `XmlBmaModel` structure.
    pub fn from_xml_str(xml_str: &str) -> Result<Self, String> {
        let xml_model: XmlBmaModel = serde_xml_rs::from_str(xml_str).map_err(|e| e.to_string())?;
        BmaModel::try_from(xml_model)
    }
}

impl BmaModel {
    /// Convert XmlVariable instance into a BmaLayoutVariable.
    fn convert_xml_layout_var(xml_var: XmlVariable) -> BmaLayoutVariable {
        let cell = if let (Some(x), Some(y)) = (xml_var.cell_x, xml_var.cell_y) {
            Some((u32::from(x), u32::from(y)))
        } else {
            None
        };
        BmaLayoutVariable {
            id: xml_var.id.into(),
            name: Some(xml_var.name),
            r#type: xml_var.r#type,
            container_id: xml_var.container_id.map(|it| it.into()),
            position: (
                Rational64::from_f64(xml_var.position_x.unwrap_or_default()).unwrap(),
                Rational64::from_f64(xml_var.position_y.unwrap_or_default()).unwrap(),
            ),
            cell,
            angle: Rational64::from_f64(xml_var.angle.unwrap_or_default()).unwrap(),
            description: None,
        }
    }

    /// Convert an XmlContainer instance into a BmaContainer.
    fn convert_xml_container(xml_container: XmlContainer) -> BmaLayoutContainer {
        BmaLayoutContainer {
            id: xml_container.id.into(),
            name: Some(xml_container.name),
            size: xml_container.size.into(),
            position: (
                Rational64::from_f64(xml_container.position_x).unwrap(),
                Rational64::from_f64(xml_container.position_y).unwrap(),
            ),
        }
    }
}

impl TryFrom<XmlBmaModel> for BmaModel {
    type Error = String;

    /// Convert JsonBmaModel instance into a proper BmaModel instance.
    ///
    /// Returns error if the update function has an incorrect format.
    fn try_from(xml_model: XmlBmaModel) -> Result<BmaModel, String> {
        // Convert the network
        let model = BmaNetwork {
            variables: xml_model
                .variables
                .variable
                .clone()
                .into_iter()
                .map(|var| BmaVariable::try_from((&xml_model, &var)))
                .collect::<Result<Vec<BmaVariable>, anyhow::Error>>()
                .map_err(|e| e.to_string())?,
            relationships: xml_model
                .relationships
                .relationship
                .into_iter()
                .map(BmaRelationship::from)
                .collect(),
            name: Some(xml_model.name),
        };

        // Convert the layout
        let (zoom_level, pan) = if let Some(layout) = xml_model.layout.as_ref() {
            let zoom_level = Rational64::from_f64(layout.zoom_level).unwrap();
            let pan_x = Rational64::from_f64(layout.pan_x).unwrap();
            let pan_y = Rational64::from_f64(layout.pan_y).unwrap();
            (Some(zoom_level), Some((pan_x, pan_y)))
        } else {
            (None, None)
        };
        let layout = BmaLayout {
            variables: xml_model
                .variables
                .variable
                .into_iter()
                .map(Self::convert_xml_layout_var)
                .collect(),
            containers: xml_model
                .containers
                .unwrap_or(XmlContainers {
                    container: Vec::new(),
                })
                .container
                .into_iter()
                .map(Self::convert_xml_container)
                .collect(),
            description: Some(xml_model.description),
            zoom_level,
            pan,
        };

        // Metadata can be constructed from various XML fields
        let mut metadata = HashMap::new();
        if xml_model.biocheck_version.is_some() {
            metadata.insert(
                "biocheck_version".to_string(),
                xml_model.biocheck_version.unwrap(),
            );
        }
        if xml_model.created_date.is_some() {
            metadata.insert("created_date".to_string(), xml_model.created_date.unwrap());
        }
        if xml_model.modified_date.is_some() {
            metadata.insert(
                "modified_date".to_string(),
                xml_model.modified_date.unwrap(),
            );
        }

        Ok(BmaModel::new(model, layout, metadata))
    }
}
