use super::utils::{send_error, send_feedback};
use pumpkin_plugin_api::{
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
};

pub struct SaveCommand;

impl CommandHandler for SaveCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let lock = crate::get_manager();
        let mgr = match lock.read() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        match mgr.save() {
            Ok(_) => {
                send_feedback(
                    &sender,
                    &format!(
                        "&aSuccessfully saved &e{}&a hologram(s) to disk storage.",
                        mgr.count()
                    ),
                );
                Ok(1)
            }
            Err(e) => {
                send_error(&sender, &format!("Failed to save holograms to disk: {e}"));
                Ok(0)
            }
        }
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("save").execute(SaveCommand)
}
