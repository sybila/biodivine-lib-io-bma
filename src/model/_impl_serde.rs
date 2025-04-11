use crate::json_model::JsonBmaModel;
use crate::model::bma_model::*;
use crate::update_fn::bma_fn_tree::BmaFnUpdate;
use crate::xml_model::XmlBmaModel;

use std::collections::HashMap;

impl BmaModel {
    /// Convert the `BmaModel` into a JSON string.
    /// Internally, we use serde_json for the conversion.
    pub fn to_json_str(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Convert the `BmaModel` into a pretty formatted JSON string.
    /// Internally, we use serde_json for the conversion.
    pub fn to_pretty_json_str(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    /// Create new BMA model from a model string in JSON format.
    /// Internally, we use json_serde serialization into an intermediate `JsonBmaModel` structure.
    pub fn from_json_str(json_str: &str) -> Result<Self, String> {
        let json_model: JsonBmaModel = serde_json::from_str(json_str).map_err(|e| e.to_string())?;
        BmaModel::try_from(json_model)
    }

    /// Create new BMA model from a model string in XML format.
    /// Internally, we use serde_xml_rs serialization into an intermediate `XmlBmaModel` structure.
    pub fn from_xml_str(xml_str: &str) -> Result<Self, String> {
        let xml_model: XmlBmaModel = serde_xml_rs::from_str(xml_str).map_err(|e| e.to_string())?;
        BmaModel::try_from(xml_model)
    }
}

impl TryFrom<JsonBmaModel> for BmaModel {
    type Error = String;

    fn try_from(json_model: JsonBmaModel) -> Result<BmaModel, String> {
        // Create a mapping from variable IDs to their names from the layout
        let layout_var_names: HashMap<u32, String> = json_model
            .layout
            .as_ref()
            .map(|layout| {
                layout
                    .variables
                    .iter()
                    .filter(|layout_var| layout_var.name.is_some())
                    .map(|layout_var| (layout_var.id, layout_var.name.clone().unwrap()))
                    .collect()
            })
            .unwrap_or_default();

        // Convert the model
        let model = Model {
            name: json_model.model.name,
            variables: json_model
                .model
                .variables
                .into_iter()
                .map(|var| {
                    Ok(Variable {
                        id: var.id,
                        name: var
                            .name
                            .unwrap_or(layout_var_names.get(&var.id).cloned().unwrap_or_default()), // Use the name from layout
                        range_from: var.range_from,
                        range_to: var.range_to,
                        formula: if !var.formula.is_empty() {
                            Some(BmaFnUpdate::parse_from_str(&var.formula)?)
                        } else {
                            None
                        },
                    })
                })
                .collect::<Result<Vec<Variable>, String>>()?,
            relationships: json_model
                .model
                .relationships
                .into_iter()
                .map(|rel| Relationship {
                    id: rel.id,
                    from_variable: rel.from_variable,
                    to_variable: rel.to_variable,
                    relationship_type: rel.r#type,
                })
                .collect(),
        };

        // Convert the layout
        let layout = json_model
            .layout
            .map(|layout| Layout {
                variables: layout
                    .variables
                    .into_iter()
                    .map(|var| LayoutVariable {
                        id: var.id,
                        name: var.name.unwrap_or_default(),
                        container_id: var.container_id,
                        variable_type: var.r#type,
                        position_x: var.position_x,
                        position_y: var.position_y,
                        cell_x: var.cell_x,
                        cell_y: var.cell_y,
                        angle: var.angle,
                        description: var.description.unwrap_or_default(),
                    })
                    .collect(),
                containers: layout
                    .containers
                    .into_iter()
                    .map(|container| Container {
                        id: container.id,
                        name: container.name.unwrap_or_default(),
                        size: container.size,
                        position_x: container.position_x,
                        position_y: container.position_y,
                    })
                    .collect(),
                description: layout.description.unwrap_or_default(),
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            })
            .unwrap_or_else(|| Layout {
                variables: vec![],
                containers: vec![],
                description: String::default(),
                zoom_level: None,
                pan_x: None,
                pan_y: None,
            });

        // metadata not present in JsonBmaModel
        let metadata = HashMap::new();

        let bma_model = BmaModel {
            model,
            layout,
            metadata,
        };
        Ok(bma_model)
    }
}

impl TryFrom<XmlBmaModel> for BmaModel {
    type Error = String;

    fn try_from(xml_model: XmlBmaModel) -> Result<BmaModel, String> {
        // Convert the model
        let model = Model {
            name: xml_model.name,
            variables: xml_model
                .variables
                .variable
                .clone()
                .into_iter()
                .map(|var| {
                    Ok(Variable {
                        id: var.id,
                        name: var.name,
                        range_from: var.range_from,
                        range_to: var.range_to,
                        formula: if !var.formula.is_empty() {
                            Some(BmaFnUpdate::parse_from_str(&var.formula)?)
                        } else {
                            None
                        },
                    })
                })
                .collect::<Result<Vec<Variable>, String>>()?,
            relationships: xml_model
                .relationships
                .relationship
                .into_iter()
                .map(|rel| Relationship {
                    id: rel.id,
                    from_variable: rel.from_variable_id,
                    to_variable: rel.to_variable_id,
                    relationship_type: rel.r#type,
                })
                .collect(),
        };

        // Convert the layout
        let layout = Layout {
            variables: xml_model
                .variables
                .variable
                .into_iter()
                .map(|var| LayoutVariable {
                    id: var.id,
                    name: var.name,
                    variable_type: var.r#type,
                    container_id: var.container_id,
                    position_x: var.position_x,
                    position_y: var.position_y,
                    cell_x: Some(var.cell_x),
                    cell_y: Some(var.cell_y),
                    angle: var.angle,
                    description: String::default(),
                })
                .collect(),
            containers: xml_model
                .containers
                .container
                .into_iter()
                .map(|container| Container {
                    id: container.id,
                    name: container.name,
                    size: container.size,
                    position_x: container.position_x,
                    position_y: container.position_y,
                })
                .collect(),
            description: xml_model.description,
            zoom_level: Some(xml_model.layout.zoom_level),
            pan_x: Some(xml_model.layout.pan_x),
            pan_y: Some(xml_model.layout.pan_y),
        };

        // Metadata can be constructed from various XML fields
        let mut metadata = HashMap::new();
        metadata.insert("biocheck_version".to_string(), xml_model.biocheck_version);
        metadata.insert("created_date".to_string(), xml_model.created_date);
        metadata.insert("modified_date".to_string(), xml_model.modified_date);

        let bma_model = BmaModel {
            model,
            layout,
            metadata,
        };
        Ok(bma_model)
    }
}
