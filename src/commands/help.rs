use super::utils::send_feedback;
use pumpkin_plugin_api::{
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
};

pub struct HelpCommand;

impl CommandHandler for HelpCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        send_feedback(&sender, "&6=== &ePisual Hologram Commands &6===");
        send_feedback(&sender, "&e/holo create <id> <text> &8- &7Create hologram at your position");
        send_feedback(&sender, "&e/holo delete <id> &8- &7Delete and despawn a hologram");
        send_feedback(&sender, "&e/holo list &8- &7List all registered holograms");
        send_feedback(&sender, "&e/holo addline <id> <text> &8- &7Add a new line of text");
        send_feedback(&sender, "&e/holo setline <id> <index> <text> &8- &7Edit a specific line (1-indexed)");
        send_feedback(&sender, "&e/holo removeline <id> <index> &8- &7Remove a specific line");
        send_feedback(&sender, "&e/holo tp <id> &8- &7Teleport to a hologram");
        send_feedback(&sender, "&e/holo movehere <id> &8- &7Move a hologram to your position");
        send_feedback(&sender, "&e/holo load &8- &7Reload and respawn all holograms from disk");
        send_feedback(&sender, "&e/holo save &8- &7Save all holograms to disk storage");
        send_feedback(&sender, "&e/holo help &8- &7Display this help message");
        Ok(1)
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("help").execute(HelpCommand)
}
