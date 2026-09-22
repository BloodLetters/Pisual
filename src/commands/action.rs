use super::utils::{
    HologramStorageSuggestions, get_int_arg, get_string_arg, send_error, send_feedback,
};
use crate::hologram::{ClickAction, ClickType};
use pumpkin_plugin_api::{
    Server,
    command::{ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
};

pub struct ActionHelpCommand;

impl CommandHandler for ActionHelpCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        send_error(
            &sender,
            "Usage: /holo action <id> <list|clear|remove|add|size>",
        );
        Ok(0)
    }
}

pub struct ActionListCommand;

impl CommandHandler for ActionListCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo action <id> list");
                return Ok(0);
            }
        };

        let lock = crate::get_manager();
        let manager = match lock.read() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get(&hologram_id) {
            Some(h) => h,
            None => {
                send_error(
                    &sender,
                    &format!("Hologram '&e{hologram_id}&c' was not found!"),
                );
                return Ok(0);
            }
        };

        if hologram.data.actions.is_empty() {
            send_feedback(
                &sender,
                &format!("&7Hologram '&e{hologram_id}&7' has no click actions configured."),
            );
            return Ok(1);
        }

        send_feedback(
            &sender,
            &format!(
                "&6--- Click Actions for '&e{hologram_id}&6' (&e{}&6) ---",
                hologram.data.actions.len()
            ),
        );

        for (index, action) in hologram.data.actions.iter().enumerate() {
            let number = index + 1;
            match action {
                ClickAction::PlayerCommand {
                    command,
                    click_type,
                } => {
                    send_feedback(
                        &sender,
                        &format!("&e{number}. &a[PlayerCmd] &7({click_type:?}): &f{command}"),
                    );
                }
                ClickAction::ConsoleCommand {
                    command,
                    click_type,
                } => {
                    send_feedback(
                        &sender,
                        &format!("&e{number}. &c[ConsoleCmd] &7({click_type:?}): &f{command}"),
                    );
                }
                ClickAction::Message {
                    message,
                    click_type,
                } => {
                    send_feedback(
                        &sender,
                        &format!("&e{number}. &b[Message] &7({click_type:?}): &f{message}"),
                    );
                }
                ClickAction::Sound {
                    sound,
                    volume,
                    pitch,
                    click_type,
                } => {
                    send_feedback(
                        &sender,
                        &format!(
                            "&e{number}. &d[Sound] &7({click_type:?}): &f{sound} &7(vol: {volume}, pitch: {pitch})"
                        ),
                    );
                }
            }
        }
        Ok(1)
    }
}

pub struct ActionClearCommand;

impl CommandHandler for ActionClearCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo action <id> clear");
                return Ok(0);
            }
        };

        let lock = crate::get_manager();
        let mut manager = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get_mut(&hologram_id) {
            Some(h) => h,
            None => {
                send_error(
                    &sender,
                    &format!("Hologram '&e{hologram_id}&c' was not found!"),
                );
                return Ok(0);
            }
        };

        hologram.clear_actions();
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!("&aCleared all click actions from hologram '&e{hologram_id}&a'!"),
        );
        Ok(1)
    }
}

pub struct ActionRemoveCommand;

impl CommandHandler for ActionRemoveCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo action <id> remove <index>");
                return Ok(0);
            }
        };

        let index_1based = match get_int_arg(&args, "index") {
            Some(n) if n >= 1 => n as usize,
            _ => {
                send_error(&sender, "Action index must be an integer >= 1.");
                return Ok(0);
            }
        };

        let lock = crate::get_manager();
        let mut manager = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let world = server.get_world_by_name(
            manager
                .get(&hologram_id)
                .map(|h| h.data.world_name.as_str())
                .unwrap_or(""),
        );

        let hologram = match manager.get_mut(&hologram_id) {
            Some(h) => h,
            None => {
                send_error(
                    &sender,
                    &format!("Hologram '&e{hologram_id}&c' was not found!"),
                );
                return Ok(0);
            }
        };

        match hologram.remove_action(index_1based - 1, world.as_ref()) {
            Ok(_) => {
                let _ = manager.save_hologram(&hologram_id);
                send_feedback(
                    &sender,
                    &format!(
                        "&aRemoved action &e#{index_1based}&a from hologram '&e{hologram_id}&a'!"
                    ),
                );
                Ok(1)
            }
            Err(err) => {
                send_error(&sender, &err);
                Ok(0)
            }
        }
    }
}

pub struct ActionAddCommand;

impl CommandHandler for ActionAddCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo action <id> add <left|right|any> <player_cmd|console_cmd|message|sound> <payload...>",
                );
                return Ok(0);
            }
        };

        let click_type_str = match get_string_arg(&args, "click_type") {
            Some(s) => s.to_lowercase(),
            None => {
                send_error(
                    &sender,
                    "Usage: /holo action <id> add <left|right|any> <player_cmd|console_cmd|message|sound> <payload...>",
                );
                return Ok(0);
            }
        };

        let click_type = match click_type_str.as_str() {
            "left" => ClickType::Left,
            "right" => ClickType::Right,
            "any" => ClickType::Any,
            _ => {
                send_error(&sender, "Invalid click type! Choose: left, right, or any.");
                return Ok(0);
            }
        };

        let action_type_str = match get_string_arg(&args, "action_type") {
            Some(s) => s.to_lowercase(),
            None => {
                send_error(
                    &sender,
                    "Usage: /holo action <id> add <left|right|any> <player_cmd|console_cmd|message|sound> <payload...>",
                );
                return Ok(0);
            }
        };

        let payload = match get_string_arg(&args, "payload") {
            Some(p) if !p.trim().is_empty() => p,
            _ => {
                send_error(&sender, "Action content cannot be empty!");
                return Ok(0);
            }
        };

        let click_action = match action_type_str.as_str() {
            "player_cmd" | "cmd" | "command" => ClickAction::PlayerCommand {
                command: payload,
                click_type,
            },
            "console_cmd" | "console" => ClickAction::ConsoleCommand {
                command: payload,
                click_type,
            },
            "message" | "msg" | "chat" => ClickAction::Message {
                message: payload,
                click_type,
            },
            "sound" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                let sound_name = parts[0].to_string();
                let volume = parts
                    .get(1)
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(1.0);
                let pitch = parts
                    .get(2)
                    .and_then(|p| p.parse::<f32>().ok())
                    .unwrap_or(1.0);
                ClickAction::Sound {
                    sound: sound_name,
                    volume,
                    pitch,
                    click_type,
                }
            }
            _ => {
                send_error(
                    &sender,
                    "Invalid action type! Choose: player_cmd, console_cmd, message, or sound.",
                );
                return Ok(0);
            }
        };

        let lock = crate::get_manager();
        let mut manager = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let world = server.get_world_by_name(
            manager
                .get(&hologram_id)
                .map(|h| h.data.world_name.as_str())
                .unwrap_or(""),
        );

        let hologram = match manager.get_mut(&hologram_id) {
            Some(h) => h,
            None => {
                send_error(
                    &sender,
                    &format!("Hologram '&e{hologram_id}&c' was not found!"),
                );
                return Ok(0);
            }
        };

        hologram.add_action(click_action, world.as_ref());
        let total_actions = hologram.data.actions.len();
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!("&aAdded action &e#{total_actions}&a to hologram '&e{hologram_id}&a'!"),
        );
        Ok(1)
    }
}

pub struct ActionSizeCommand;

impl CommandHandler for ActionSizeCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo action <id> size <width> <height>");
                return Ok(0);
            }
        };

        let width_str = get_string_arg(&args, "width");
        let height_str = get_string_arg(&args, "height");

        let width = match width_str {
            Some(w) => match w.parse::<f32>() {
                Ok(val) if val > 0.0 => Some(val),
                _ => {
                    send_error(&sender, "Width must be a positive number.");
                    return Ok(0);
                }
            },
            None => None,
        };

        let height = match height_str {
            Some(h) => match h.parse::<f32>() {
                Ok(val) if val > 0.0 => Some(val),
                _ => {
                    send_error(&sender, "Height must be a positive number.");
                    return Ok(0);
                }
            },
            None => None,
        };

        let lock = crate::get_manager();
        let mut manager = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let world = server.get_world_by_name(
            manager
                .get(&hologram_id)
                .map(|h| h.data.world_name.as_str())
                .unwrap_or(""),
        );

        let hologram = match manager.get_mut(&hologram_id) {
            Some(h) => h,
            None => {
                send_error(
                    &sender,
                    &format!("Hologram '&e{hologram_id}&c' was not found!"),
                );
                return Ok(0);
            }
        };

        hologram.set_interaction_dimensions(width, height, world.as_ref());
        let actual_width = hologram.data.interaction_width.unwrap_or(1.2);
        let actual_height = hologram
            .data
            .interaction_height
            .unwrap_or_else(|| ((hologram.data.lines.len() as f32) * 0.35).max(0.6));
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!(
                "&aUpdated interaction hitbox for '&e{hologram_id}&a' to width: &e{actual_width}&a, height: &e{actual_height}&a!"
            ),
        );
        Ok(1)
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("action")
        .execute(ActionHelpCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(ActionHelpCommand)
                .then(CommandNode::literal("list").execute(ActionListCommand))
                .then(CommandNode::literal("clear").execute(ActionClearCommand))
                .then(
                    CommandNode::literal("remove")
                        .execute(ActionHelpCommand)
                        .then(
                            CommandNode::argument("index", &ArgumentType::Integer((Some(1), None)))
                                .execute(ActionRemoveCommand),
                        ),
                )
                .then(
                    CommandNode::literal("size")
                        .execute(ActionHelpCommand)
                        .then(
                            CommandNode::argument(
                                "width",
                                &ArgumentType::String(StringType::SingleWord),
                            )
                            .execute(ActionSizeCommand)
                            .then(
                                CommandNode::argument(
                                    "height",
                                    &ArgumentType::String(StringType::SingleWord),
                                )
                                .execute(ActionSizeCommand),
                            ),
                        ),
                )
                .then(
                    CommandNode::literal("add").execute(ActionHelpCommand).then(
                        CommandNode::argument(
                            "click_type",
                            &ArgumentType::String(StringType::SingleWord),
                        )
                        .execute(ActionHelpCommand)
                        .then(
                            CommandNode::argument(
                                "action_type",
                                &ArgumentType::String(StringType::SingleWord),
                            )
                            .execute(ActionHelpCommand)
                            .then(
                                CommandNode::argument(
                                    "payload",
                                    &ArgumentType::String(StringType::Greedy),
                                )
                                .execute(ActionAddCommand),
                            ),
                        ),
                    ),
                ),
        )
}
