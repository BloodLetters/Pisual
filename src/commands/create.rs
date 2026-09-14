use super::utils::{get_string_arg, is_player, send_error, send_feedback};
use crate::hologram::HologramData;
use pumpkin_plugin_api::{
    command::{ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
    Server,
};

pub struct CreateCommand;

impl CommandHandler for CreateCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = is_player(&sender)?;

        let id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo create <id> [text]");
                return Ok(0);
            }
        };

        let text = get_string_arg(&args, "text").unwrap_or_else(|| "Pisual Hologram".to_string());

        let pos = player.get_position();
        let world = player.get_world();
        let world_name = world.get_id();

        let spawn_pos = (pos.0, pos.1 + 1.2, pos.2);

        let data = HologramData::new(id.clone(), world_name, spawn_pos, vec![text]);

        let lock = crate::get_manager();
        let mut mgr = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        if mgr.contains(&id) {
            send_error(&sender, &format!("Hologram with ID '&e{id}&c' already exists!"));
            return Ok(0);
        }

        match mgr.create_hologram(data, Some(&world)) {
            Ok(_) => {
                send_feedback(
                    &sender,
                    &format!("&aHologram '&e{id}&a' created successfully at your position!"),
                );
                Ok(1)
            }
            Err(e) => {
                send_error(&sender, &format!("Failed to create hologram: {e}"));
                Ok(0)
            }
        }
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("create")
        .execute(CreateCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .execute(CreateCommand)
                .then(
                    CommandNode::argument("text", &ArgumentType::String(StringType::Greedy))
                        .execute(CreateCommand),
                ),
        )
}
