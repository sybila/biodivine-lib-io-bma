use crate::BmaLayout;
use crate::serde::quote_num::QuoteNum;
use crate::serde::xml::XmlBmaModel;
use crate::utils::{clone_into_vec, f64_or_default, rational_or_default};
use serde::{Deserialize, Serialize};

/// Structure to deserialize XML info about layout. This includes only a few
/// metadata items like zoom level and pan position. Info about variables and
/// containers is stored directly in the model object of BMA XML (as weird as it is...).
///
/// The zoom and pan values can be missing in the XML. If not provided, default
/// values are used.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct XmlLayout {
    #[serde(rename = "Columns")]
    pub columns: QuoteNum,
    #[serde(rename = "Rows")]
    pub rows: QuoteNum,
    #[serde(default, rename = "ZoomLevel")]
    pub zoom_level: f64,
    #[serde(default, rename = "PanX")]
    pub pan_x: f64,
    #[serde(default, rename = "PanY")]
    pub pan_y: f64,
}

impl From<BmaLayout> for XmlLayout {
    fn from(value: BmaLayout) -> Self {
        // As far as I can tell, `columns` and `rows` are no longer used...
        let (pan_x, pan_y) = value.pan.unwrap_or_default();
        XmlLayout {
            columns: QuoteNum::from(1),
            rows: QuoteNum::from(1),
            zoom_level: f64_or_default(value.zoom_level.unwrap_or_default()),
            pan_x: f64_or_default(pan_x),
            pan_y: f64_or_default(pan_y),
        }
    }
}

/*
   There is no conversion for `XmlLayout` to `BmaLayout`. Instead, this is built into the
   `XmlModel` conversion, since that's where most of the data stored in `BmaLayout` really is.
*/

impl From<&XmlBmaModel> for BmaLayout {
    fn from(value: &XmlBmaModel) -> Self {
        let (zoom_level, pan) = if let Some(layout) = value.layout.as_ref() {
            let zoom_level = rational_or_default(layout.zoom_level);
            let pan = (
                rational_or_default(layout.pan_x),
                rational_or_default(layout.pan_y),
            );
            (Some(zoom_level), Some(pan))
        } else {
            (None, None)
        };
        BmaLayout {
            variables: clone_into_vec(&value.variables.variable),
            containers: clone_into_vec(&value.containers.clone().unwrap_or_default().container),
            description: value.description.clone(),
            zoom_level,
            pan,
        }
    }
}
