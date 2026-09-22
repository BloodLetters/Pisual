pub mod entity;
pub mod model;

pub use entity::*;
pub use model::*;

use pumpkin_plugin_api::world::{Entity, World};

pub struct ElementHandle {
    pub element_id: String,
    pub visual_entity: Option<Entity>,
    pub interaction_entity: Option<Entity>,
}

pub struct Hologram {
    pub data: HologramData,
    pub entity: Option<Entity>,
    pub visual_entity: Option<Entity>,
    pub interaction_entity: Option<Entity>,
    pub element_handles: Vec<ElementHandle>,
    pub last_rendered_text: Option<String>,
    pub tick_counter: u64,
}

impl Hologram {
    pub fn new(data: HologramData) -> Self {
        Self {
            data,
            entity: None,
            visual_entity: None,
            interaction_entity: None,
            element_handles: Vec::new(),
            last_rendered_text: None,
            tick_counter: 0,
        }
    }

    pub fn is_spawned(&self) -> bool {
        self.entity.is_some()
            || self.visual_entity.is_some()
            || self.interaction_entity.is_some()
            || !self.element_handles.is_empty()
    }

    pub fn spawn(&mut self, world: &World) -> bool {
        self.despawn();

        let mut spawned = false;

        if let Some(text_entity) = entity::spawn_display(world, &self.data) {
            self.last_rendered_text = Some(entity::render_lines(&self.data, Some(world)));
            self.entity = Some(text_entity);
            spawned = true;
        }

        if let Some(visual_entity) = entity::spawn_visual(world, &self.data) {
            self.visual_entity = Some(visual_entity);
            spawned = true;
        }

        if !self.data.actions.is_empty()
            && let Some(interaction_entity) = entity::spawn_interaction(world, &self.data)
        {
            self.interaction_entity = Some(interaction_entity);
            spawned = true;
        }

        for element in &self.data.elements {
            let visual_entity = entity::spawn_element_visual(world, &self.data, element);
            let interaction_entity = if !element.actions.is_empty() {
                entity::spawn_element_interaction(world, &self.data, element)
            } else {
                None
            };
            if visual_entity.is_some() || interaction_entity.is_some() {
                spawned = true;
            }
            self.element_handles.push(ElementHandle {
                element_id: element.id.clone(),
                visual_entity,
                interaction_entity,
            });
        }

        spawned
    }

    pub fn despawn(&mut self) {
        self.last_rendered_text = None;
        self.tick_counter = 0;
        if let Some(text_entity) = self.entity.take() {
            entity::despawn_display(&text_entity);
        }
        if let Some(visual_entity) = self.visual_entity.take() {
            entity::despawn_display(&visual_entity);
        }
        if let Some(interaction_entity) = self.interaction_entity.take() {
            entity::despawn_display(&interaction_entity);
        }
        for handle in self.element_handles.drain(..) {
            if let Some(visual_entity) = handle.visual_entity {
                entity::despawn_display(&visual_entity);
            }
            if let Some(interaction_entity) = handle.interaction_entity {
                entity::despawn_display(&interaction_entity);
            }
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

        if let Some(interaction_entity) = &self.interaction_entity {
            entity::teleport_display(interaction_entity, new_pos, interaction_entity.get_world());
        } else if !self.data.actions.is_empty() {
            self.interaction_entity = entity::spawn_interaction(world, &self.data);
        }

        for element in &self.data.elements {
            let element_pos = entity::calculate_element_position(new_pos, element);
            if let Some(handle) = self
                .element_handles
                .iter_mut()
                .find(|h| h.element_id == element.id)
            {
                if let Some(visual_entity) = &handle.visual_entity {
                    entity::teleport_display(visual_entity, element_pos, visual_entity.get_world());
                } else {
                    handle.visual_entity = entity::spawn_element_visual(world, &self.data, element);
                }
                if let Some(interaction_entity) = &handle.interaction_entity {
                    entity::teleport_display(
                        interaction_entity,
                        element_pos,
                        interaction_entity.get_world(),
                    );
                } else if !element.actions.is_empty() {
                    handle.interaction_entity =
                        entity::spawn_element_interaction(world, &self.data, element);
                }
            } else {
                let visual_entity = entity::spawn_element_visual(world, &self.data, element);
                let interaction_entity = if !element.actions.is_empty() {
                    entity::spawn_element_interaction(world, &self.data, element)
                } else {
                    None
                };
                self.element_handles.push(ElementHandle {
                    element_id: element.id.clone(),
                    visual_entity,
                    interaction_entity,
                });
            }
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

    pub fn has_placeholders(&self) -> bool {
        crate::placeholder::any_has_placeholders(&self.data.lines)
    }

    pub fn set_refresh_interval(&mut self, interval: Option<u64>) {
        self.data.refresh_interval = interval;
    }

    pub fn add_action(&mut self, action: ClickAction, world: Option<&World>) {
        self.data.actions.push(action);
        self.sync_interaction(world);
    }

    pub fn remove_action(
        &mut self,
        index: usize,
        world: Option<&World>,
    ) -> Result<ClickAction, String> {
        if index < self.data.actions.len() {
            let removed = self.data.actions.remove(index);
            self.sync_interaction(world);
            Ok(removed)
        } else {
            Err(format!(
                "Action index {} out of bounds (total actions: {})",
                index,
                self.data.actions.len()
            ))
        }
    }

    pub fn clear_actions(&mut self) {
        self.data.actions.clear();
        if let Some(interaction_entity) = self.interaction_entity.take() {
            entity::despawn_display(&interaction_entity);
        }
    }

    pub fn set_interaction_dimensions(
        &mut self,
        width: Option<f32>,
        height: Option<f32>,
        world: Option<&World>,
    ) {
        self.data.interaction_width = width;
        self.data.interaction_height = height;
        self.sync_interaction(world);
    }

    pub fn sync_interaction(&mut self, world: Option<&World>) {
        if self.data.actions.is_empty() {
            if let Some(interaction_entity) = self.interaction_entity.take() {
                entity::despawn_display(&interaction_entity);
            }
        } else if let Some(interaction_entity) = &self.interaction_entity {
            entity::update_interaction(interaction_entity, &self.data);
        } else if let Some(world) = world {
            self.interaction_entity = entity::spawn_interaction(world, &self.data);
        }
    }

    pub fn refresh_text(&mut self, world: Option<&World>) -> bool {
        let text_entity = match &self.entity {
            Some(e) => e,
            None => return false,
        };

        let rendered = entity::render_lines(&self.data, world);
        if let Some(last) = &self.last_rendered_text
            && last == &rendered
        {
            return false;
        }

        let updated = entity::update_text(text_entity, &rendered);
        if updated {
            self.last_rendered_text = Some(rendered);
        }
        updated
    }

    pub fn sync_elements(&mut self, world: Option<&World>) {
        let mut current_handles = std::mem::take(&mut self.element_handles);
        for element in &self.data.elements {
            let existing_position = current_handles
                .iter()
                .position(|handle| handle.element_id == element.id);

            if let Some(index) = existing_position {
                let mut handle = current_handles.remove(index);
                if let Some(interaction_entity) = &handle.interaction_entity {
                    if element.actions.is_empty() {
                        entity::despawn_display(interaction_entity);
                        handle.interaction_entity = None;
                    } else {
                        entity::update_element_interaction(interaction_entity, element);
                    }
                } else if !element.actions.is_empty()
                    && let Some(current_world) = world
                {
                    handle.interaction_entity =
                        entity::spawn_element_interaction(current_world, &self.data, element);
                }
                self.element_handles.push(handle);
            } else if let Some(current_world) = world {
                let visual_entity =
                    entity::spawn_element_visual(current_world, &self.data, element);
                let interaction_entity = if !element.actions.is_empty() {
                    entity::spawn_element_interaction(current_world, &self.data, element)
                } else {
                    None
                };
                self.element_handles.push(ElementHandle {
                    element_id: element.id.clone(),
                    visual_entity,
                    interaction_entity,
                });
            }
        }

        for orphaned_handle in current_handles {
            if let Some(visual_entity) = orphaned_handle.visual_entity {
                entity::despawn_display(&visual_entity);
            }
            if let Some(interaction_entity) = orphaned_handle.interaction_entity {
                entity::despawn_display(&interaction_entity);
            }
        }
    }

    pub fn add_element(&mut self, element: VisualElement, world: Option<&World>) {
        self.data.add_element(element);
        self.sync_elements(world);
    }

    pub fn remove_element(&mut self, element_id: &str) -> Option<VisualElement> {
        let removed = self.data.remove_element(element_id);
        if let Some(index) = self
            .element_handles
            .iter()
            .position(|handle| handle.element_id == element_id)
        {
            let handle = self.element_handles.remove(index);
            if let Some(visual_entity) = handle.visual_entity {
                entity::despawn_display(&visual_entity);
            }
            if let Some(interaction_entity) = handle.interaction_entity {
                entity::despawn_display(&interaction_entity);
            }
        }
        removed
    }

    fn sync_display(&mut self) {
        let world = crate::get_world(&self.data.world_name);
        if let Some(entity) = &self.entity {
            entity::update_display(entity, &self.data, world.as_ref());
            self.last_rendered_text = Some(entity::render_lines(&self.data, world.as_ref()));
        }
        self.sync_interaction(world.as_ref());
        self.sync_elements(world.as_ref());
    }
}

impl Drop for Hologram {
    fn drop(&mut self) {
        self.despawn();
    }
}
