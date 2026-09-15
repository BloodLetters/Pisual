use super::utils::{send_error, send_feedback};
use pumpkin_plugin_api::{
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
};

pub struct ReloadCommand;

impl CommandHandler for ReloadCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        send_feedback(&sender, "&eReloading holograms from disk storage...");

        let lock = crate::get_manager();
        let mut mgr = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        match mgr.load_and_spawn(|name| server.get_world_by_name(name)) {
            Ok(_) => {
                send_feedback(
                    &sender,
                    &format!(
                        "&aSuccessfully reloaded &e{}&a hologram(s) and spawned them in their worlds!",
                        mgr.count()
                    ),
                );
                Ok(1)
            }
            Err(e) => {
                send_error(&sender, &format!("Failed to reload holograms from disk: {e}"));
                Ok(0)
            }
        }
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("reload").execute(ReloadCommand)
}
