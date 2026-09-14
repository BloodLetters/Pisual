pub mod create;
pub mod delete;
pub mod help;
pub mod lines;
pub mod list;
pub mod load;
pub mod save;
pub mod teleport;
pub mod utils;

use pumpkin_plugin_api::{command::Command, Context};

/// Permission node required for administering Pisual holograms.
pub const PERMISSION_NODE: &str = "pisual.admin";

/// Builds the root command tree and registers it with the plugin context.
pub fn register_commands(context: &Context) {
    let cmd = Command::new(
        &[
            "pisual".to_string(),
            "holo".to_string(),
            "hologram".to_string(),
        ],
        "Manage Pisual holograms",
    )
    .execute(help::HelpCommand)
    .then(create::build_node())
    .then(delete::build_node())
    .then(delete::build_remove_node())
    .then(load::build_node())
    .then(load::build_reload_node())
    .then(list::build_node())
    .then(lines::build_addline_node())
    .then(lines::build_setline_node())
    .then(lines::build_removeline_node())
    .then(teleport::build_tp_node())
    .then(teleport::build_movehere_node())
    .then(teleport::build_tphere_node())
    .then(save::build_node())
    .then(help::build_node());

    context.register_command(cmd, PERMISSION_NODE);
    crate::logger::info("Registered /pisual (/holo, /hologram) command hierarchy.");
}
