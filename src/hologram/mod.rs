pub mod entity;
pub mod model;

pub use entity::*;
pub use model::*;

use pumpkin_plugin_api::world::{Entity, World};

pub struct Hologram {
    pub data: HologramData,
    pub entity: Option<Entity>,
    pub visual_entity: Option<Entity>,
}

impl Hologram {
    pub fn new(data: HologramData) -> Self {
        Self {
            data,
            entity: None,
            visual_entity: None,
        }
    }

    pub fn is_spawned(&self) -> bool {
        self.entity.is_some() || self.visual_entity.is_some()
    }

    pub fn spawn(&mut self, world: &World) -> bool {
        self.despawn();

        let mut spawned = false;

        if let Some(text_entity) = entity::spawn_display(world, &self.data) {
            self.entity = Some(text_entity);
            spawned = true;
        }

        if let Some(visual_entity) = entity::spawn_visual(world, &self.data) {
            self.visual_entity = Some(visual_entity);
            spawned = true;
        }

        spawned
    }

    pub fn despawn(&mut self) {
        if let Some(text_entity) = self.entity.take() {
            entity::despawn_display(&text_entity);
        }
        if let Some(visual_entity) = self.visual_entity.take() {
            entity::despawn_display(&visual_entity);
        }
    }

    pub fn respawn(&mut self, world: &World) -> bool {
        self.spawn(world)
    }

    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.data.lines = lines;
        self.sync_display();
    }

    pub fn add_line(&mut self, line: String) {
        self.data.lines.push(line);
        self.sync_display();
    }

    pub fn set_line(&mut self, index: usize, line: String) -> Result<(), String> {
        if index < self.data.lines.len() {
            self.data.lines[index] = line;
            self.sync_display();
            Ok(())
        } else {
            Err(format!(
                "Line index {} out of bounds (total lines: {})",
                index,
                self.data.lines.len()
            ))
        }
    }

    pub fn remove_line(&mut self, index: usize) -> Result<String, String> {
        if index < self.data.lines.len() {
            let removed = self.data.lines.remove(index);
            self.sync_display();
            Ok(removed)
        } else {
            Err(format!(
                "Line index {} out of bounds (total lines: {})",
                index,
                self.data.lines.len()
            ))
        }
    }

    pub fn set_visual(&mut self, visual: Option<VisualConfig>, world: Option<&World>) {
        self.data.visual = visual;
        if let Some(existing_visual) = self.visual_entity.take() {
            entity::despawn_display(&existing_visual);
        }
        if let Some(current_world) = world {
            self.visual_entity = entity::spawn_visual(current_world, &self.data);
        }
    }

    pub fn remove_visual(&mut self) {
        self.data.visual = None;
        if let Some(existing_visual) = self.visual_entity.take() {
            entity::despawn_display(&existing_visual);
        }
    }

    pub fn teleport(&mut self, new_pos: (f64, f64, f64), world: &World, world_name: String) {
        self.data.position = new_pos;
        self.data.world_name = world_name;

        if let Some(entity) = &self.entity {
            entity::teleport_display(entity, new_pos, entity.get_world());
        } else {
            self.spawn(world);
            return;
        }

        if let Some(visual_entity) = &self.visual_entity {
            let visual_pos = entity::calculate_visual_position(new_pos, &self.data);
            entity::teleport_display(visual_entity, visual_pos, visual_entity.get_world());
        } else if self.data.visual.is_some() {
            self.visual_entity = entity::spawn_visual(world, &self.data);
        }
    }

    pub fn set_billboard(&mut self, billboard: BillboardType) {
        self.data.billboard = billboard;
        self.sync_display();
    }

    pub fn set_shadow(&mut self, shadow: bool) {
        self.data.shadow = shadow;
        self.sync_display();
    }

    pub fn set_see_through(&mut self, see_through: bool) {
        self.data.see_through = see_through;
        self.sync_display();
    }

    pub fn set_scale(&mut self, scale: (f32, f32, f32)) {
        self.data.scale = scale;
        self.sync_display();
    }

    fn sync_display(&self) {
        if let Some(entity) = &self.entity {
            entity::update_display(entity, &self.data);
        }
    }
}

impl Drop for Hologram {
    fn drop(&mut self) {
        self.despawn();
    }
}
