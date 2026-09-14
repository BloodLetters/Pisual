pub mod commands;
pub mod hologram;
pub mod logger;
pub mod manager;
pub mod storage;

pub use hologram::{BillboardType, Hologram, HologramData};
pub use manager::HologramManager;

use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};
use std::sync::{OnceLock, RwLock};

static MANAGER: OnceLock<RwLock<HologramManager>> = OnceLock::new();

pub fn get_manager() -> &'static RwLock<HologramManager> {
    MANAGER.get().expect("HologramManager has not been initialized")
}

struct Pisual;

impl Plugin for Pisual {
    fn new() -> Self {
        Pisual
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Pisual".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["SirAshesh".into()],
            description: "Modern Hologram plugin for Pumpkin MC powered by TextDisplay entities".into(),
            permissions: vec![
                pumpkin_plugin_api::permissions::FS_READ_DATA.into(),
                pumpkin_plugin_api::permissions::FS_WRITE_DATA.into(),
            ],
            dependencies: vec![],
        }
    }

    fn on_load(&self, context: Context) -> pumpkin_plugin_api::Result<()> {
        logger::info("Starting Pisual...");

        let data_folder = context.get_data_folder();
        logger::info(&format!("Pisual data folder: {}", data_folder));

        let manager = RwLock::new(HologramManager::new(data_folder));
        let _ = MANAGER.set(manager);

        let server = context.get_server();
        if let Ok(mut mgr) = get_manager().write() {
            if let Err(e) = mgr.load_and_spawn(|name| server.get_world_by_name(name)) {
                logger::warn(&format!("Notice while loading hologram data: {}", e));
            }
            logger::info(&format!(
                "Pisual enabled! Total {} hologram(s) registered on server.",
                mgr.count()
            ));
        }

        commands::register_commands(&context);

        Ok(())
    }

    fn on_unload(&self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        logger::info("Disabling Pisual plugin...");

        if let Some(lock) = MANAGER.get() {
            if let Ok(mut mgr) = lock.write() {
                mgr.despawn_all();
                logger::info("All hologram entities have been cleared from worlds.");
            }
        }

        logger::info("Pisual plugin unloaded. Goodbye!");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(Pisual);