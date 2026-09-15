use super::utils::{get_string_arg, send_error, send_feedback, HologramStorageSuggestions};
use pumpkin_plugin_api::{
    command::{ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
    Server,
};

pub struct DeleteCommand;

impl CommandHandler for DeleteCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo delete <id>");
                return Ok(0);
            }
        };

        let lock = crate::get_manager();
        let mut mgr = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        match mgr.delete_hologram(&id) {
            Ok(_) => {
                send_feedback(
                    &sender,
                    &format!("&aHologram '&e{id}&a' has been deleted and despawned."),
                );
                Ok(1)
            }
            Err(e) => {
                send_error(&sender, &format!("Failed to delete hologram: {e}"));
                Ok(0)
            }
        }
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("delete")
        .execute(DeleteCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(DeleteCommand),
        )
}

pub fn build_remove_node() -> CommandNode {
    CommandNode::literal("remove")
        .execute(DeleteCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(DeleteCommand),
        )
}
