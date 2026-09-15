use crate::hologram::BillboardType;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum IpcRequest {
    Create {
        id: String,
        world: String,
        position: (f64, f64, f64),
        lines: Vec<String>,
        #[serde(default)]
        ram: bool,
        #[serde(default)]
        billboard: Option<BillboardType>,
        #[serde(default)]
        shadow: Option<bool>,
        #[serde(default)]
        see_through: Option<bool>,
        #[serde(default)]
        scale: Option<(f32, f32, f32)>,
    },
    CreateRam {
        id: String,
        world: String,
        position: (f64, f64, f64),
        lines: Vec<String>,
        #[serde(default)]
        billboard: Option<BillboardType>,
        #[serde(default)]
        shadow: Option<bool>,
        #[serde(default)]
        see_through: Option<bool>,
        #[serde(default)]
        scale: Option<(f32, f32, f32)>,
    },
    Edit {
        id: String,
        #[serde(default)]
        lines: Option<Vec<String>>,
        #[serde(default)]
        billboard: Option<BillboardType>,
        #[serde(default)]
        shadow: Option<bool>,
        #[serde(default)]
        see_through: Option<bool>,
        #[serde(default)]
        scale: Option<(f32, f32, f32)>,
    },
    Move {
        id: String,
        position: (f64, f64, f64),
        #[serde(default)]
        world: Option<String>,
    },
    Delete {
        id: String,
    },
    Get {
        id: String,
    },
    List,
}
