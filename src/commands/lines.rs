use super::utils::{
    HologramStorageSuggestions, get_int_arg, get_string_arg, send_error, send_feedback,
};
use pumpkin_plugin_api::{
    Server,
    command::{ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
};

pub struct AddLineCommand;

impl CommandHandler for AddLineCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo addline <id> <text>");
                return Ok(0);
            }
        };

        let text = match get_string_arg(&args, "text") {
            Some(text) => text,
            None => {
                send_error(
                    &sender,
                    "Text cannot be empty! Usage: /holo addline <id> <text>",
                );
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

        let holo = match mgr.get_mut(&id) {
            Some(h) => h,
            None => {
                send_error(&sender, &format!("Hologram '&e{id}&c' was not found!"));
                return Ok(0);
            }
        };

        holo.add_line(text);
        let total_lines = holo.data.lines.len();
        let _ = mgr.save();

        send_feedback(
            &sender,
            &format!("&aAdded line &e#{total_lines}&a to hologram '&e{id}&a'!"),
        );
        Ok(1)
    }
}

pub struct SetLineCommand;

impl CommandHandler for SetLineCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo setline <id> <index> <text>");
                return Ok(0);
            }
        };

        let index_1based = match get_int_arg(&args, "index") {
            Some(n) if n >= 1 => n as usize,
            _ => {
                send_error(
                    &sender,
                    "Line index must be a positive integer starting at 1!",
                );
                return Ok(0);
            }
        };

        let text = match get_string_arg(&args, "text") {
            Some(text) => text,
            None => {
                send_error(
                    &sender,
                    "Text cannot be empty! Usage: /holo setline <id> <index> <text>",
                );
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

        let holo = match mgr.get_mut(&id) {
            Some(h) => h,
            None => {
                send_error(&sender, &format!("Hologram '&e{id}&c' was not found!"));
                return Ok(0);
            }
        };

        let idx_0based = index_1based - 1;
        match holo.set_line(idx_0based, text) {
            Ok(_) => {
                let _ = mgr.save();
                send_feedback(
                    &sender,
                    &format!("&aUpdated line &e#{index_1based}&a of hologram '&e{id}&a'!"),
                );
                Ok(1)
            }
            Err(e) => {
                send_error(&sender, &e);
                Ok(0)
            }
        }
    }
}

pub struct RemoveLineCommand;

impl CommandHandler for RemoveLineCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo removeline <id> <index>");
                return Ok(0);
            }
        };

        let index_1based = match get_int_arg(&args, "index") {
            Some(n) if n >= 1 => n as usize,
            _ => {
                send_error(
                    &sender,
                    "Line index must be a positive integer starting at 1!",
                );
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

        let holo = match mgr.get_mut(&id) {
            Some(h) => h,
            None => {
                send_error(&sender, &format!("Hologram '&e{id}&c' was not found!"));
                return Ok(0);
            }
        };

        let idx_0based = index_1based - 1;
        match holo.remove_line(idx_0based) {
            Ok(removed) => {
                let _ = mgr.save();
                send_feedback(
                    &sender,
                    &format!(
                        "&aRemoved line &e#{index_1based} &7(\"{removed}&7\") &afrom hologram '&e{id}&a'!"
                    ),
                );
                Ok(1)
            }
            Err(e) => {
                send_error(&sender, &e);
                Ok(0)
            }
        }
    }
}

pub fn build_addline_node() -> CommandNode {
    CommandNode::literal("addline")
        .execute(AddLineCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(AddLineCommand)
                .then(
                    CommandNode::argument("text", &ArgumentType::String(StringType::Greedy))
                        .execute(AddLineCommand),
                ),
        )
}

pub fn build_setline_node() -> CommandNode {
    CommandNode::literal("setline")
        .execute(SetLineCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(SetLineCommand)
                .then(
                    CommandNode::argument("index", &ArgumentType::Integer((Some(1), None)))
                        .execute(SetLineCommand)
                        .then(
                            CommandNode::argument(
                                "text",
                                &ArgumentType::String(StringType::Greedy),
                            )
                            .execute(SetLineCommand),
                        ),
                ),
        )
}

pub fn build_removeline_node() -> CommandNode {
    CommandNode::literal("removeline")
        .execute(RemoveLineCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(RemoveLineCommand)
                .then(
                    CommandNode::argument("index", &ArgumentType::Integer((Some(1), None)))
                        .execute(RemoveLineCommand),
                ),
        )
}
