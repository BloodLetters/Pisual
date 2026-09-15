use super::utils::{
    calculate_facing_direction, get_string_arg, is_player, send_error, send_feedback,
    HologramStorageSuggestions,
};
use crate::hologram::{parse_entity_type, VisualConfig};
use pumpkin_plugin_api::{
    command::{ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
    Server,
};

fn parse_numeric_argument(raw_value: &str, argument_name: &str) -> Result<f64, String> {
    raw_value
        .parse::<f64>()
        .map_err(|_| format!("{argument_name} must be a valid number, got '{raw_value}'!"))
}

fn parse_single_or_triple_coordinates(
    arguments: &ConsumedArgs,
) -> Result<(f64, f64, f64), String> {
    let first_argument = match get_string_arg(arguments, "x_or_y") {
        Some(value) => value,
        None => return Err("Offset coordinate must be specified.".to_string()),
    };

    let second_argument = get_string_arg(arguments, "y");
    let third_argument = get_string_arg(arguments, "z");

    if let (Some(y_value), Some(z_value)) = (second_argument, third_argument) {
        let x = parse_numeric_argument(&first_argument, "X offset")?;
        let y = parse_numeric_argument(&y_value, "Y offset")?;
        let z = parse_numeric_argument(&z_value, "Z offset")?;
        Ok((x, y, z))
    } else {
        let y = parse_numeric_argument(&first_argument, "Y offset")?;
        Ok((-0.1, y, -0.1))
    }
}

fn parse_single_or_triple_scale(
    arguments: &ConsumedArgs,
) -> Result<(f32, f32, f32), String> {
    let first_argument = match get_string_arg(arguments, "x_or_uniform") {
        Some(value) => value,
        None => return Err("Scale multiplier must be specified.".to_string()),
    };

    let second_argument = get_string_arg(arguments, "y");
    let third_argument = get_string_arg(arguments, "z");

    if let (Some(y_value), Some(z_value)) = (second_argument, third_argument) {
        let scale_x = parse_numeric_argument(&first_argument, "X scale")? as f32;
        let scale_y = parse_numeric_argument(&y_value, "Y scale")? as f32;
        let scale_z = parse_numeric_argument(&z_value, "Z scale")? as f32;
        Ok((scale_x, scale_y, scale_z))
    } else {
        let uniform_scale = parse_numeric_argument(&first_argument, "Uniform scale")? as f32;
        Ok((uniform_scale, uniform_scale, uniform_scale))
    }
}

fn calculate_facing_from_sender(
    sender: &CommandSender,
    hologram_position: (f64, f64, f64),
    visual_offset: (f64, f64, f64),
) -> Option<(f32, f32)> {
    let player = sender.as_player()?;
    let player_position = player.get_position();
    let player_yaw = player.get_yaw();
    let visual_world_position = (
        hologram_position.0 + visual_offset.0,
        hologram_position.1 + visual_offset.1,
        hologram_position.2 + visual_offset.2,
    );
    Some(calculate_facing_direction(
        visual_world_position,
        player_position,
        player_yaw,
    ))
}

pub struct VisualHelpCommand;

impl CommandHandler for VisualHelpCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        send_feedback(&sender, "&6=== &ePisual Visual Commands &6===");
        send_feedback(
            &sender,
            "&e/holo visual <id> entity <type> [offset] &8- &7Attach an entity facing you",
        );
        send_feedback(
            &sender,
            "&e/holo visual <id> item <item> [offset] &8- &7Attach a floating item",
        );
        send_feedback(
            &sender,
            "&e/holo visual <id> block <block> [offset] &8- &7Attach a block display",
        );
        send_feedback(
            &sender,
            "&e/holo visual <id> offset <y> [x y z] &8- &7Adjust visual offset position",
        );
        send_feedback(
            &sender,
            "&e/holo visual <id> scale <scale> [x y z] &8- &7Adjust visual scale",
        );
        send_feedback(
            &sender,
            "&e/holo visual <id> faceplayer &8- &7Rotate visual to face you",
        );
        send_feedback(
            &sender,
            "&e/holo visual <id> clear &8- &7Remove visual from hologram",
        );
        Ok(1)
    }
}

pub struct VisualEntityCommand;

impl CommandHandler for VisualEntityCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let identifier = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo visual <id> entity <type> [offset]");
                return Ok(0);
            }
        };

        let entity_type_name = match get_string_arg(&args, "type") {
            Some(type_name) if !type_name.trim().is_empty() => type_name,
            _ => {
                send_error(
                    &sender,
                    "Please specify an entity type (e.g. allay, iron_golem, zombie, villager).",
                );
                return Ok(0);
            }
        };

        if parse_entity_type(&entity_type_name).is_none() {
            send_error(
                &sender,
                &format!("Unknown entity type '&e{entity_type_name}&c'!"),
            );
            return Ok(0);
        }

        let optional_offset = match get_string_arg(&args, "offset") {
            Some(offset_text) => match parse_numeric_argument(&offset_text, "Offset") {
                Ok(value) => Some(value),
                Err(error_message) => {
                    send_error(&sender, &error_message);
                    return Ok(0);
                }
            },
            None => None,
        };

        let manager_lock = crate::get_manager();
        let mut manager = match manager_lock.write() {
            Ok(manager_guard) => manager_guard,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get_mut(&identifier) {
            Some(holo) => holo,
            None => {
                send_error(&sender, &format!("Hologram '&e{identifier}&c' was not found!"));
                return Ok(0);
            }
        };

        let mut configuration = VisualConfig::entity(&entity_type_name);
        if let Some(offset_value) = optional_offset {
            configuration.offset_y = offset_value;
        }

        configuration.rotation = calculate_facing_from_sender(
            &sender,
            hologram.data.position,
            (configuration.offset_x, configuration.offset_y, configuration.offset_z),
        );

        let target_world = crate::get_world(&hologram.data.world_name);
        hologram.set_visual(Some(configuration), target_world.as_ref());
        let _ = manager.save();

        send_feedback(
            &sender,
            &format!("&aAttached entity visual '&e{entity_type_name}&a' facing you to hologram '&e{identifier}&a'!"),
        );
        Ok(1)
    }
}

pub struct VisualItemCommand;

impl CommandHandler for VisualItemCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let identifier = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo visual <id> item <name> [offset]");
                return Ok(0);
            }
        };

        let item_name = match get_string_arg(&args, "name") {
            Some(name) if !name.trim().is_empty() => name,
            _ => {
                send_error(
                    &sender,
                    "Please specify an item name (e.g. diamond_sword, golden_apple).",
                );
                return Ok(0);
            }
        };

        let optional_offset = match get_string_arg(&args, "offset") {
            Some(offset_text) => match parse_numeric_argument(&offset_text, "Offset") {
                Ok(value) => Some(value),
                Err(error_message) => {
                    send_error(&sender, &error_message);
                    return Ok(0);
                }
            },
            None => None,
        };

        let manager_lock = crate::get_manager();
        let mut manager = match manager_lock.write() {
            Ok(manager_guard) => manager_guard,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get_mut(&identifier) {
            Some(holo) => holo,
            None => {
                send_error(&sender, &format!("Hologram '&e{identifier}&c' was not found!"));
                return Ok(0);
            }
        };

        let mut configuration = VisualConfig::item(&item_name);
        if let Some(offset_value) = optional_offset {
            configuration.offset_y = offset_value;
        }

        configuration.rotation = calculate_facing_from_sender(
            &sender,
            hologram.data.position,
            (configuration.offset_x, configuration.offset_y, configuration.offset_z),
        );

        let target_world = crate::get_world(&hologram.data.world_name);
        hologram.set_visual(Some(configuration), target_world.as_ref());
        let _ = manager.save();

        send_feedback(
            &sender,
            &format!("&aAttached item visual '&e{item_name}&a' facing you to hologram '&e{identifier}&a'!"),
        );
        Ok(1)
    }
}

pub struct VisualBlockCommand;

impl CommandHandler for VisualBlockCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let identifier = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo visual <id> block <name> [offset]");
                return Ok(0);
            }
        };

        let block_name = match get_string_arg(&args, "name") {
            Some(name) if !name.trim().is_empty() => name,
            _ => {
                send_error(
                    &sender,
                    "Please specify a block name (e.g. diamond_block, beacon).",
                );
                return Ok(0);
            }
        };

        let optional_offset = match get_string_arg(&args, "offset") {
            Some(offset_text) => match parse_numeric_argument(&offset_text, "Offset") {
                Ok(value) => Some(value),
                Err(error_message) => {
                    send_error(&sender, &error_message);
                    return Ok(0);
                }
            },
            None => None,
        };

        let manager_lock = crate::get_manager();
        let mut manager = match manager_lock.write() {
            Ok(manager_guard) => manager_guard,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get_mut(&identifier) {
            Some(holo) => holo,
            None => {
                send_error(&sender, &format!("Hologram '&e{identifier}&c' was not found!"));
                return Ok(0);
            }
        };

        let mut configuration = VisualConfig::block(&block_name);
        if let Some(offset_value) = optional_offset {
            configuration.offset_y = offset_value;
        }

        configuration.rotation = calculate_facing_from_sender(
            &sender,
            hologram.data.position,
            (configuration.offset_x, configuration.offset_y, configuration.offset_z),
        );

        let target_world = crate::get_world(&hologram.data.world_name);
        hologram.set_visual(Some(configuration), target_world.as_ref());
        let _ = manager.save();

        send_feedback(
            &sender,
            &format!("&aAttached block visual '&e{block_name}&a' facing you to hologram '&e{identifier}&a'!"),
        );
        Ok(1)
    }
}

pub struct VisualOffsetCommand;

impl CommandHandler for VisualOffsetCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let identifier = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo visual <id> offset <y> OR /holo visual <id> offset <x> <y> <z>",
                );
                return Ok(0);
            }
        };

        let (offset_x, offset_y, offset_z) = match parse_single_or_triple_coordinates(&args) {
            Ok(coordinates) => coordinates,
            Err(error_message) => {
                send_error(&sender, &error_message);
                return Ok(0);
            }
        };

        let manager_lock = crate::get_manager();
        let mut manager = match manager_lock.write() {
            Ok(manager_guard) => manager_guard,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get_mut(&identifier) {
            Some(holo) => holo,
            None => {
                send_error(&sender, &format!("Hologram '&e{identifier}&c' was not found!"));
                return Ok(0);
            }
        };

        let mut configuration = match hologram.data.visual.clone() {
            Some(visual_config) => visual_config,
            None => {
                send_error(
                    &sender,
                    &format!("Hologram '&e{identifier}&c' has no visual attached! Use /holo visual <id> entity|item|block first."),
                );
                return Ok(0);
            }
        };

        configuration.offset_x = offset_x;
        configuration.offset_y = offset_y;
        configuration.offset_z = offset_z;

        let target_world = crate::get_world(&hologram.data.world_name);
        hologram.set_visual(Some(configuration), target_world.as_ref());
        let _ = manager.save();

        send_feedback(
            &sender,
            &format!("&aUpdated visual offset of '&e{identifier}&a' to &e({offset_x}, {offset_y}, {offset_z})&a!"),
        );
        Ok(1)
    }
}

pub struct VisualScaleCommand;

impl CommandHandler for VisualScaleCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let identifier = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo visual <id> scale <uniform> OR /holo visual <id> scale <x> <y> <z>",
                );
                return Ok(0);
            }
        };

        let (scale_x, scale_y, scale_z) = match parse_single_or_triple_scale(&args) {
            Ok(multipliers) => multipliers,
            Err(error_message) => {
                send_error(&sender, &error_message);
                return Ok(0);
            }
        };

        let manager_lock = crate::get_manager();
        let mut manager = match manager_lock.write() {
            Ok(manager_guard) => manager_guard,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get_mut(&identifier) {
            Some(holo) => holo,
            None => {
                send_error(&sender, &format!("Hologram '&e{identifier}&c' was not found!"));
                return Ok(0);
            }
        };

        let mut configuration = match hologram.data.visual.clone() {
            Some(visual_config) => visual_config,
            None => {
                send_error(
                    &sender,
                    &format!("Hologram '&e{identifier}&c' has no visual attached! Use /holo visual <id> entity|item|block first."),
                );
                return Ok(0);
            }
        };

        configuration.scale = (scale_x, scale_y, scale_z);

        let target_world = crate::get_world(&hologram.data.world_name);
        hologram.set_visual(Some(configuration), target_world.as_ref());
        let _ = manager.save();

        send_feedback(
            &sender,
            &format!("&aUpdated visual scale of '&e{identifier}&a' to &e({scale_x}, {scale_y}, {scale_z})&a!"),
        );
        Ok(1)
    }
}

pub struct VisualFacePlayerCommand;

impl CommandHandler for VisualFacePlayerCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let player = is_player(&sender)?;
        let identifier = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo visual <id> faceplayer");
                return Ok(0);
            }
        };

        let manager_lock = crate::get_manager();
        let mut manager = match manager_lock.write() {
            Ok(manager_guard) => manager_guard,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get_mut(&identifier) {
            Some(holo) => holo,
            None => {
                send_error(&sender, &format!("Hologram '&e{identifier}&c' was not found!"));
                return Ok(0);
            }
        };

        let mut configuration = match hologram.data.visual.clone() {
            Some(visual_config) => visual_config,
            None => {
                send_error(
                    &sender,
                    &format!("Hologram '&e{identifier}&c' has no visual attached!"),
                );
                return Ok(0);
            }
        };

        let player_position = player.get_position();
        let player_yaw = player.get_yaw();
        let hologram_position = hologram.data.position;
        let visual_world_position = (
            hologram_position.0 + configuration.offset_x,
            hologram_position.1 + configuration.offset_y,
            hologram_position.2 + configuration.offset_z,
        );
        let facing_direction = calculate_facing_direction(
            visual_world_position,
            player_position,
            player_yaw,
        );

        configuration.rotation = Some(facing_direction);

        let target_world = crate::get_world(&hologram.data.world_name);
        hologram.set_visual(Some(configuration), target_world.as_ref());
        let _ = manager.save();

        send_feedback(
            &sender,
            &format!("&aVisual on hologram '&e{identifier}&a' is now facing you!"),
        );
        Ok(1)
    }
}

pub struct VisualClearCommand;

impl CommandHandler for VisualClearCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let identifier = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo visual <id> clear");
                return Ok(0);
            }
        };

        let manager_lock = crate::get_manager();
        let mut manager = match manager_lock.write() {
            Ok(manager_guard) => manager_guard,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let hologram = match manager.get_mut(&identifier) {
            Some(holo) => holo,
            None => {
                send_error(&sender, &format!("Hologram '&e{identifier}&c' was not found!"));
                return Ok(0);
            }
        };

        hologram.remove_visual();
        let _ = manager.save();

        send_feedback(
            &sender,
            &format!("&aRemoved visual from hologram '&e{identifier}&a'!"),
        );
        Ok(1)
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("visual")
        .execute(VisualHelpCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(VisualHelpCommand)
                .then(
                    CommandNode::literal("entity")
                        .execute(VisualHelpCommand)
                        .then(
                            CommandNode::argument("type", &ArgumentType::String(StringType::SingleWord))
                                .execute(VisualEntityCommand)
                                .then(
                                    CommandNode::argument("offset", &ArgumentType::String(StringType::SingleWord))
                                        .execute(VisualEntityCommand),
                                ),
                        ),
                )
                .then(
                    CommandNode::literal("item")
                        .execute(VisualHelpCommand)
                        .then(
                            CommandNode::argument("name", &ArgumentType::String(StringType::SingleWord))
                                .execute(VisualItemCommand)
                                .then(
                                    CommandNode::argument("offset", &ArgumentType::String(StringType::SingleWord))
                                        .execute(VisualItemCommand),
                                ),
                        ),
                )
                .then(
                    CommandNode::literal("block")
                        .execute(VisualHelpCommand)
                        .then(
                            CommandNode::argument("name", &ArgumentType::String(StringType::SingleWord))
                                .execute(VisualBlockCommand)
                                .then(
                                    CommandNode::argument("offset", &ArgumentType::String(StringType::SingleWord))
                                        .execute(VisualBlockCommand),
                                ),
                        ),
                )
                .then(
                    CommandNode::literal("offset")
                        .execute(VisualHelpCommand)
                        .then(
                            CommandNode::argument("x_or_y", &ArgumentType::String(StringType::SingleWord))
                                .execute(VisualOffsetCommand)
                                .then(
                                    CommandNode::argument("y", &ArgumentType::String(StringType::SingleWord))
                                        .execute(VisualOffsetCommand)
                                        .then(
                                            CommandNode::argument("z", &ArgumentType::String(StringType::SingleWord))
                                                .execute(VisualOffsetCommand),
                                        ),
                                ),
                        ),
                )
                .then(
                    CommandNode::literal("scale")
                        .execute(VisualHelpCommand)
                        .then(
                            CommandNode::argument("x_or_uniform", &ArgumentType::String(StringType::SingleWord))
                                .execute(VisualScaleCommand)
                                .then(
                                    CommandNode::argument("y", &ArgumentType::String(StringType::SingleWord))
                                        .execute(VisualScaleCommand)
                                        .then(
                                            CommandNode::argument("z", &ArgumentType::String(StringType::SingleWord))
                                                .execute(VisualScaleCommand),
                                        ),
                                ),
                        ),
                )
                .then(CommandNode::literal("faceplayer").execute(VisualFacePlayerCommand))
                .then(CommandNode::literal("clear").execute(VisualClearCommand))
                .then(CommandNode::literal("remove").execute(VisualClearCommand)),
        )
}
