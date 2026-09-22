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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClickType {
    #[default]
    Any,
    Right,
    Left,
}

impl ClickType {
    pub fn matches_click(&self, is_attack: bool) -> bool {
        match self {
            Self::Any => true,
            Self::Left => is_attack,
            Self::Right => !is_attack,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClickAction {
    PlayerCommand {
        command: String,
        #[serde(default)]
        click_type: ClickType,
    },
    ConsoleCommand {
        command: String,
        #[serde(default)]
        click_type: ClickType,
    },
    Message {
        message: String,
        #[serde(default)]
        click_type: ClickType,
    },
    Sound {
        sound: String,
        #[serde(default = "default_sound_volume")]
        volume: f32,
        #[serde(default = "default_sound_pitch")]
        pitch: f32,
        #[serde(default)]
        click_type: ClickType,
    },
}

fn default_sound_volume() -> f32 {
    1.0
}

fn default_sound_pitch() -> f32 {
    1.0
}

impl ClickAction {
    pub fn matches_click(&self, is_attack: bool) -> bool {
        match self {
            Self::PlayerCommand { click_type, .. } => click_type.matches_click(is_attack),
            Self::ConsoleCommand { click_type, .. } => click_type.matches_click(is_attack),
            Self::Message { click_type, .. } => click_type.matches_click(is_attack),
            Self::Sound { click_type, .. } => click_type.matches_click(is_attack),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualElement {
    pub id: String,
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
    #[serde(default)]
    pub actions: Vec<ClickAction>,
    #[serde(default)]
    pub interaction_width: Option<f32>,
    #[serde(default)]
    pub interaction_height: Option<f32>,
}

impl VisualElement {
    pub fn new(id: impl Into<String>, visual_type: VisualType) -> Self {
        Self {
            id: id.into(),
            visual_type,
            offset_x: default_offset_x(),
            offset_y: default_offset_y(),
            offset_z: default_offset_z(),
            scale: default_scale(),
            rotation: None,
            actions: Vec::new(),
            interaction_width: None,
            interaction_height: None,
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
    #[serde(default)]
    pub elements: Vec<VisualElement>,
    #[serde(default)]
    pub refresh_interval: Option<u64>,
    #[serde(default)]
    pub actions: Vec<ClickAction>,
    #[serde(default)]
    pub interaction_width: Option<f32>,
    #[serde(default)]
    pub interaction_height: Option<f32>,
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
            elements: Vec::new(),
            refresh_interval: None,
            actions: Vec::new(),
            interaction_width: None,
            interaction_height: None,
            is_ram: false,
        }
    }

    pub fn get_element(&self, element_id: &str) -> Option<&VisualElement> {
        self.elements
            .iter()
            .find(|element| element.id == element_id)
    }

    pub fn get_element_mut(&mut self, element_id: &str) -> Option<&mut VisualElement> {
        self.elements
            .iter_mut()
            .find(|element| element.id == element_id)
    }

    pub fn add_element(&mut self, element: VisualElement) {
        if let Some(existing) = self.get_element_mut(&element.id) {
            *existing = element;
        } else {
            self.elements.push(element);
        }
    }

    pub fn remove_element(&mut self, element_id: &str) -> Option<VisualElement> {
        if let Some(index) = self
            .elements
            .iter()
            .position(|element| element.id == element_id)
        {
            Some(self.elements.remove(index))
        } else {
            None
        }
    }
}
