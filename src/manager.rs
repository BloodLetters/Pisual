use crate::hologram::{Hologram, HologramData};
use crate::storage;
use pumpkin_plugin_api::world::World;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct HologramManager {
    holograms: HashMap<String, Hologram>,
    data_folder: PathBuf,
}

impl HologramManager {
    pub fn new(data_folder: impl Into<PathBuf>) -> Self {
        Self {
            holograms: HashMap::new(),
            data_folder: data_folder.into(),
        }
    }

    pub fn data_folder(&self) -> &Path {
        &self.data_folder
    }

    pub fn contains(&self, id: &str) -> bool {
        self.holograms.contains_key(id)
    }

    pub fn get(&self, id: &str) -> Option<&Hologram> {
        self.holograms.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Hologram> {
        self.holograms.get_mut(id)
    }

    pub fn list(&self) -> Vec<&Hologram> {
        self.holograms.values().collect()
    }

    pub fn count(&self) -> usize {
        self.holograms.len()
    }

    pub fn create_hologram(
        &mut self,
        data: HologramData,
        world: Option<&World>,
    ) -> Result<(), String> {
        let id = data.id.clone();
        if !storage::is_valid_id(&id) {
            return Err(format!(
                "Invalid hologram ID '{id}'! ID must be 1-64 characters using letters, numbers, underscores, and hyphens."
            ));
        }

        if self.holograms.contains_key(&id) {
            return Err(format!("Hologram with ID '{id}' already exists!"));
        }

        let is_ram = data.is_ram;
        if !is_ram {
            storage::save_hologram(&self.data_folder, &data)?;
        }

        let mut hologram = Hologram::new(data);
        if let Some(world) = world {
            hologram.spawn(world);
        }

        self.holograms.insert(id, hologram);
        Ok(())
    }

    pub fn delete_hologram(&mut self, id: &str) -> Result<HologramData, String> {
        if let Some(mut hologram) = self.holograms.remove(id) {
            hologram.despawn();
            if !hologram.data.is_ram {
                let _ = storage::delete_hologram_file(&self.data_folder, id);
            }
            Ok(hologram.data.clone())
        } else {
            Err(format!("Hologram with ID '{id}' was not found!"))
        }
    }

    pub fn save_hologram(&self, id: &str) -> Result<(), String> {
        if let Some(hologram) = self.holograms.get(id) {
            if !hologram.data.is_ram {
                storage::save_hologram(&self.data_folder, &hologram.data)?;
            }
            Ok(())
        } else {
            Err(format!("Hologram with ID '{id}' was not found!"))
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let all_data: Vec<HologramData> = self
            .holograms
            .values()
            .filter(|hologram| !hologram.data.is_ram)
            .map(|hologram| hologram.data.clone())
            .collect();
        storage::save_to_disk(&self.data_folder, &all_data)
    }

    pub fn load_and_spawn<F>(&mut self, mut resolve_world: F) -> Result<(), String>
    where
        F: FnMut(&str) -> Option<World>,
    {
        self.despawn_all();
        self.holograms.clear();

        let saved_data = storage::load_from_disk(&self.data_folder)?;
        let count = saved_data.len();

        for data in saved_data {
            let world_name = data.world_name.clone();
            let id = data.id.clone();
            let mut hologram = Hologram::new(data);

            if let Some(world) = resolve_world(&world_name) {
                if hologram.spawn(&world) {
                    crate::logger::info(&format!("Hologram '{}' spawned in world '{}'", id, world_name));
                } else {
                    crate::logger::warn(&format!("Failed to spawn entity for hologram '{}'", id));
                }
            } else {
                crate::logger::warn(&format!(
                    "World '{}' for hologram '{}' is not loaded yet. Hologram retained in memory.",
                    world_name,
                    id
                ));
            }

            self.holograms.insert(id, hologram);
        }

        crate::logger::info(&format!("Finished loading {} holograms.", count));
        Ok(())
    }

    pub fn despawn_all(&mut self) {
        for hologram in self.holograms.values_mut() {
            hologram.despawn();
        }
    }

    pub fn respawn_all<F>(&mut self, mut resolve_world: F)
    where
        F: FnMut(&str) -> Option<World>,
    {
        for hologram in self.holograms.values_mut() {
            if let Some(world) = resolve_world(&hologram.data.world_name) {
                hologram.spawn(&world);
            }
        }
    }

    pub fn set_refresh_interval(&mut self, id: &str, interval: Option<u64>) -> Result<(), String> {
        if let Some(hologram) = self.holograms.get_mut(id) {
            hologram.set_refresh_interval(interval);
            if !hologram.data.is_ram {
                storage::save_hologram(&self.data_folder, &hologram.data)?;
            }
            Ok(())
        } else {
            Err(format!("Hologram with ID '{id}' was not found!"))
        }
    }

    pub fn tick(&mut self, server: &pumpkin_plugin_api::Server, tick_step: u64) {
        for hologram in self.holograms.values_mut() {
            if !hologram.is_spawned() {
                continue;
            }

            let has_placeholders = hologram.has_placeholders();
            let interval = match hologram.data.refresh_interval {
                Some(0) => continue,
                Some(n) => n,
                None => {
                    if has_placeholders {
                        20
                    } else {
                        continue;
                    }
                }
            };

            hologram.tick_counter += tick_step;
            if hologram.tick_counter < interval {
                continue;
            }
            hologram.tick_counter = 0;

            let world = server.get_world_by_name(&hologram.data.world_name);
            hologram.refresh_text(world.as_ref());
        }
    }
}
