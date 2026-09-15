use pumpkin_plugin_api::display::BillboardMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BillboardType {
    #[default]
    Center,
    Vertical,
    Horizontal,
    Fixed,
}

impl From<BillboardType> for BillboardMode {
    fn from(b: BillboardType) -> Self {
        match b {
            BillboardType::Center => BillboardMode::Center,
            BillboardType::Vertical => BillboardMode::Vertical,
            BillboardType::Horizontal => BillboardMode::Horizontal,
            BillboardType::Fixed => BillboardMode::Fixed,
        }
    }
}

impl From<BillboardMode> for BillboardType {
    fn from(b: BillboardMode) -> Self {
        match b {
            BillboardMode::Center => BillboardType::Center,
            BillboardMode::Vertical => BillboardType::Vertical,
            BillboardMode::Horizontal => BillboardType::Horizontal,
            BillboardMode::Fixed => BillboardType::Fixed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HologramData {
    pub id: String,
    pub world_name: String,
    pub position: (f64, f64, f64),
    pub lines: Vec<String>,
    #[serde(default)]
    pub billboard: BillboardType,
    #[serde(default = "default_true")]
    pub shadow: bool,
    #[serde(default)]
    pub see_through: bool,
    #[serde(default)]
    pub background: Option<i32>,
    #[serde(default = "default_range")]
    pub view_range: f32,
    #[serde(default = "default_scale")]
    pub scale: (f32, f32, f32),
    #[serde(default, skip_serializing)]
    pub is_ram: bool,
}

fn default_true() -> bool {
    true
}

fn default_range() -> f32 {
    64.0
}

fn default_scale() -> (f32, f32, f32) {
    (1.0, 1.0, 1.0)
}

impl HologramData {
    pub fn new(
        id: impl Into<String>,
        world_name: impl Into<String>,
        position: (f64, f64, f64),
        lines: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            world_name: world_name.into(),
            position,
            lines,
            billboard: BillboardType::Center,
            shadow: true,
            see_through: false,
            background: None,
            view_range: 64.0,
            scale: (1.0, 1.0, 1.0),
            is_ram: false,
        }
    }
}
