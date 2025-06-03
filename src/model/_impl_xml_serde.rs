use crate::update_fn::bma_fn_update::BmaFnUpdate;
use crate::xml_model::*;
use crate::{
    BmaLayout, BmaLayoutContainer, BmaLayoutVariable, BmaModel, BmaNetwork, BmaRelationship,
    BmaVariable,
};
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
    /// Convert an XmlVariable instance into a proper BmaVariable.
    ///
    /// If the update function has an incorrect format, we return an error.
    fn convert_xml_variable(
        xml_var: XmlVariable,
        xml_model: &XmlBmaModel,
        all_vars: &HashMap<u32, String>,
    ) -> Result<BmaVariable, String> {
        // Get a set of regulators for the variable that we'll pass to update fn parser
        let regulators = xml_model.get_regulators(xml_var.id);
        let named_regulators = all_vars
            .clone()
            .into_iter()
            .filter(|(id, _)| regulators.contains(id))
            .collect::<HashMap<u32, String>>();

        // Try to parse the update function from the JSON variable
        let formula = if !xml_var.formula.is_empty() {
            Some(BmaFnUpdate::parse_from_str(
                &xml_var.formula,
                &named_regulators,
            )?)
        } else {
            None
        };

        Ok(BmaVariable {
            id: xml_var.id,
            name: xml_var.name,
            range_from: xml_var.range_from,
            range_to: xml_var.range_to,
            formula,
        })
    }

    /// Convert XmlRelationship instance into a proper BmaRelationship.
    fn convert_xml_relationship(xml_rel: XmlRelationship) -> BmaRelationship {
        BmaRelationship {
            id: xml_rel.id,
            from_variable: xml_rel.from_variable_id,
            to_variable: xml_rel.to_variable_id,
            relationship_type: xml_rel.r#type,
        }
    }

    /// Convert XmlVariable instance into a BmaLayoutVariable.
    fn convert_xml_layout_var(xml_var: XmlVariable) -> BmaLayoutVariable {
        BmaLayoutVariable {
            id: xml_var.id,
            name: xml_var.name,
            variable_type: xml_var.r#type,
            container_id: xml_var.container_id,
            position_x: xml_var.position_x.unwrap_or_default(),
            position_y: xml_var.position_y.unwrap_or_default(),
            cell_x: xml_var.cell_x,
            cell_y: xml_var.cell_y,
            angle: xml_var.angle.unwrap_or_default(),
            description: String::default(),
        }
    }

    /// Convert an XmlContainer instance into a BmaContainer.
    fn convert_xml_container(xml_container: XmlContainer) -> BmaLayoutContainer {
        BmaLayoutContainer {
            id: xml_container.id,
            name: xml_container.name,
            size: xml_container.size,
            position_x: xml_container.position_x,
            position_y: xml_container.position_y,
        }
    }
}

impl TryFrom<XmlBmaModel> for BmaModel {
    type Error = String;

    /// Convert JsonBmaModel instance into a proper BmaModel instance.
    ///
    /// Returns error if the update function has an incorrect format.
    fn try_from(xml_model: XmlBmaModel) -> Result<BmaModel, String> {
        // Precompute ID-name mapping of all variables
        let all_variables: HashMap<u32, String> = xml_model.collect_all_variables();

        // Convert the network
        let model = BmaNetwork {
            variables: xml_model
                .variables
                .variable
                .clone()
                .into_iter()
                .map(|var| Self::convert_xml_variable(var.clone(), &xml_model, &all_variables))
                .collect::<Result<Vec<BmaVariable>, String>>()?,
            relationships: xml_model
                .relationships
                .relationship
                .into_iter()
                .map(Self::convert_xml_relationship)
                .collect(),
            name: xml_model.name,
        };

        // Convert the layout
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
            description: xml_model.description,
            zoom_level: xml_model.layout.as_ref().map(|l| l.zoom_level),
            pan_x: xml_model.layout.as_ref().map(|l| l.pan_x),
            pan_y: xml_model.layout.as_ref().map(|l| l.pan_y),
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
