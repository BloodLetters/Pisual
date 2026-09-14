use super::utils::{send_error, send_feedback};
use pumpkin_plugin_api::{
    command::{CommandError, CommandNode, CommandSender, ConsumedArgs},
    commands::CommandHandler,
    Server,
};

pub struct ListCommand;

impl CommandHandler for ListCommand {
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

        let holograms = mgr.list();
        if holograms.is_empty() {
            send_feedback(&sender, "&7No holograms are currently registered.");
            return Ok(0);
        }

        send_feedback(
            &sender,
            &format!("&6=== &ePisual Holograms (&f{}&e) &6===", holograms.len()),
        );

        for holo in holograms {
            let data = &holo.data;
            let status = if holo.is_spawned() {
                "&2[Spawned]"
            } else {
                "&c[Despawned]"
            };

            send_feedback(
                &sender,
                &format!(
                    "&e• &b{} {} &7in &f{} &8(&7{:.1}, {:.1}, {:.1}&8) &7- &a{} line(s)",
                    data.id,
                    status,
                    data.world_name,
                    data.position.0,
                    data.position.1,
                    data.position.2,
                    data.lines.len()
                ),
            );
        }

        Ok(0)
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("list").execute(ListCommand)
}
