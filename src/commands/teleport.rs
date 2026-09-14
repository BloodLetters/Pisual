use super::utils::{get_string_arg, is_player, send_error, send_feedback};
use pumpkin_plugin_api::{
    command::{ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
    Server,
};

pub struct TeleportCommand;

impl CommandHandler for TeleportCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = is_player(&sender)?;

        let id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo tp <id>");
                return Ok(0);
            }
        };

        let lock = crate::get_manager();
        let mgr = match lock.read() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let holo = match mgr.get(&id) {
            Some(h) => h,
            None => {
                send_error(&sender, &format!("Hologram '&e{id}&c' was not found!"));
                return Ok(0);
            }
        };

        let target_pos = holo.data.position;
        let world_name = &holo.data.world_name;

        let world = match server.get_world_by_name(world_name) {
            Some(w) => w,
            None => {
                send_error(
                    &sender,
                    &format!("World '&e{world_name}&c' containing hologram '&e{id}&c' is not loaded!"),
                );
                return Ok(0);
            }
        };

        player.teleport(target_pos, None, None, world);
        send_feedback(&sender, &format!("&aTeleported to hologram '&e{id}&a'!"));

        Ok(1)
    }
}

pub struct MoveHereCommand;

impl CommandHandler for MoveHereCommand {
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
                send_error(&sender, "Usage: /holo movehere <id>");
                return Ok(0);
            }
        };

        let pos = player.get_position();
        let world = player.get_world();
        let world_name = world.get_id();

        let lock = crate::get_manager();
        let mut mgr = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let holo = match mgr.get_mut(&id) {
            Some(h) => h,
            None => {
                send_error(&sender, &format!("Hologram '&e{id}&c' was not found!"));
                return Ok(0);
            }
        };

        let new_pos = (pos.0, pos.1 + 1.2, pos.2);
        holo.teleport(new_pos, &world, world_name);
        let _ = mgr.save();

        send_feedback(
            &sender,
            &format!("&aMoved hologram '&e{id}&a' to your current position!"),
        );

        Ok(1)
    }
}

pub fn build_tp_node() -> CommandNode {
    CommandNode::literal("tp")
        .execute(TeleportCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .execute(TeleportCommand),
        )
}

pub fn build_movehere_node() -> CommandNode {
    CommandNode::literal("movehere")
        .execute(MoveHereCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .execute(MoveHereCommand),
        )
}

pub fn build_tphere_node() -> CommandNode {
    CommandNode::literal("tphere")
        .execute(MoveHereCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .execute(MoveHereCommand),
        )
}
