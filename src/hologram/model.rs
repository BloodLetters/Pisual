use pumpkin_plugin_api::display::BillboardMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VisualType {
    Item { item: String },
    Block { block: String },
    Entity { entity_type: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualConfig {
    pub visual_type: VisualType,
    #[serde(default = "default_offset_x")]
    pub offset_x: f64,
    #[serde(default = "default_offset_y")]
    pub offset_y: f64,
    #[serde(default = "default_offset_z")]
    pub offset_z: f64,
    #[serde(default = "default_scale")]
    pub scale: (f32, f32, f32),
    #[serde(default)]
    pub rotation: Option<(f32, f32)>,
}

impl VisualConfig {
    pub fn item(item: impl Into<String>) -> Self {
        Self {
            visual_type: VisualType::Item { item: item.into() },
            offset_x: default_offset_x(),
            offset_y: default_offset_y(),
            offset_z: default_offset_z(),
            scale: default_scale(),
            rotation: None,
        }
    }

    pub fn block(block: impl Into<String>) -> Self {
        Self {
            visual_type: VisualType::Block {
                block: block.into(),
            },
            offset_x: default_offset_x(),
            offset_y: default_offset_y(),
            offset_z: default_offset_z(),
            scale: default_scale(),
            rotation: None,
        }
    }

    pub fn entity(entity_type: impl Into<String>) -> Self {
        Self {
            visual_type: VisualType::Entity {
                entity_type: entity_type.into(),
            },
            offset_x: default_offset_x(),
            offset_y: default_offset_y(),
            offset_z: default_offset_z(),
            scale: default_scale(),
            rotation: None,
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
    #[serde(default)]
    pub visual: Option<VisualConfig>,
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
    (0.3, 0.3, 0.3)
}

fn default_offset_x() -> f64 {
    -0.1
}

fn default_offset_y() -> f64 {
    0.8
}

fn default_offset_z() -> f64 {
    -0.1
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
            visual: None,
            is_ram: false,
        }
    }
}
