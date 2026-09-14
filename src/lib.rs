use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};
use tracing::*;

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
            description: "Hologram plugin for pumpkin".into(),
            permissions: vec![
                pumpkin_plugin_api::permissions::FS_READ_DATA.into(),
                pumpkin_plugin_api::permissions::FS_WRITE_DATA.into(),
            ],
            dependencies: vec![],
        }
    }

    fn on_load(&self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("Pisual plugin loaded. Hello!");
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("Pisual plugin unloaded. Goodbye!");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(Pisual);