use super::utils::{
    HologramStorageSuggestions, get_int_arg, get_string_arg, send_error, send_feedback,
};
use crate::hologram::{
    ClickAction, ClickType, VisualElement, VisualType, entity::parse_entity_type,
};
use pumpkin_plugin_api::{
    Server,
    command::{ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
};

fn parse_number(raw: &str, field_name: &str) -> Result<f64, String> {
    raw.parse::<f64>()
        .map_err(|_| format!("{field_name} must be a valid number, got '{raw}'!"))
}

pub struct ElementHelpCommand;

impl CommandHandler for ElementHelpCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        send_feedback(
            &sender,
            "&6=== &ePisual Multi-Visual Element Commands &6===",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> list &8- &7List all visual elements on hologram",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> add <elem_id> <item|block|entity> <name> [x] [y] [z] &8- &7Add element",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> remove <elem_id> &8- &7Remove a visual element",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> offset <elem_id> <x> <y> <z> &8- &7Set element 3D offset",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> scale <elem_id> <uniform_or_x> [y] [z] &8- &7Set element scale",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> size <elem_id> [width] [height] &8- &7Set hitbox dimensions",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> action <elem_id> list &8- &7List actions for element",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> action <elem_id> add <click> <type> <val> &8- &7Add click action",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> action <elem_id> remove <index> &8- &7Remove click action",
        );
        send_feedback(
            &sender,
            "&e/holo element <id> action <elem_id> clear &8- &7Clear all actions on element",
        );
        Ok(1)
    }
}

pub struct ElementListCommand;

impl CommandHandler for ElementListCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo element <id> list");
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

        if hologram.data.elements.is_empty() {
            send_feedback(
                &sender,
                &format!("&7Hologram '&e{hologram_id}&7' has no multi-visual elements."),
            );
            return Ok(1);
        }

        send_feedback(
            &sender,
            &format!(
                "&6=== &eElements of Hologram '&a{hologram_id}&e' ({}) &6===",
                hologram.data.elements.len()
            ),
        );

        for (index, element) in hologram.data.elements.iter().enumerate() {
            let type_description = match &element.visual_type {
                VisualType::Item { item } => format!("&bitem &f{item}"),
                VisualType::Block { block } => format!("&2block &f{block}"),
                VisualType::Entity { entity_type } => format!("&dmob &f{entity_type}"),
            };

            send_feedback(
                &sender,
                &format!(
                    "&8[&e{}&8] &a{} &8- {} &8| &7offset: &f({:.2}, {:.2}, {:.2}) &8| &7actions: &e{}",
                    index + 1,
                    element.id,
                    type_description,
                    element.offset_x,
                    element.offset_y,
                    element.offset_z,
                    element.actions.len()
                ),
            );
        }

        Ok(1)
    }
}

pub struct ElementAddCommand;

impl CommandHandler for ElementAddCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo element <id> add <elem_id> <item|block|entity> <name> [x] [y] [z]",
                );
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
                return Ok(0);
            }
        };

        let kind = match get_string_arg(&args, "kind") {
            Some(k) => k.to_ascii_lowercase(),
            _ => {
                send_error(&sender, "Visual kind must be: item, block, or entity");
                return Ok(0);
            }
        };

        let name = match get_string_arg(&args, "name") {
            Some(n) if !n.trim().is_empty() => n,
            _ => {
                send_error(&sender, "Item, block, or entity name must be specified!");
                return Ok(0);
            }
        };

        let visual_type = match kind.as_str() {
            "item" => VisualType::Item { item: name },
            "block" => VisualType::Block { block: name },
            "entity" => {
                if parse_entity_type(&name).is_none() {
                    send_error(&sender, &format!("Unknown entity type '{name}'!"));
                    return Ok(0);
                }
                VisualType::Entity { entity_type: name }
            }
            _ => {
                send_error(
                    &sender,
                    &format!("Invalid kind '{kind}'! Expected: item, block, or entity"),
                );
                return Ok(0);
            }
        };

        let mut element = VisualElement::new(element_id.clone(), visual_type);

        if let (Some(x_str), Some(y_str), Some(z_str)) = (
            get_string_arg(&args, "offset_x"),
            get_string_arg(&args, "offset_y"),
            get_string_arg(&args, "offset_z"),
        ) {
            match (
                parse_number(&x_str, "X offset"),
                parse_number(&y_str, "Y offset"),
                parse_number(&z_str, "Z offset"),
            ) {
                (Ok(x), Ok(y), Ok(z)) => {
                    element.offset_x = x;
                    element.offset_y = y;
                    element.offset_z = z;
                }
                (Err(err), _, _) | (_, Err(err), _) | (_, _, Err(err)) => {
                    send_error(&sender, &err);
                    return Ok(0);
                }
            }
        }

        let lock = crate::get_manager();
        let mut manager = match lock.write() {
            Ok(mgr) => mgr,
            Err(_) => {
                send_error(&sender, "Failed to acquire lock on HologramManager.");
                return Ok(1);
            }
        };

        let world_instance = {
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
            let world_name = hologram.data.world_name.clone();
            let world = crate::get_world(&world_name);
            hologram.add_element(element, world.as_ref());
            world
        };

        let _ = world_instance;
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!("&aAdded element '&e{element_id}&a' to hologram '&e{hologram_id}&a'!"),
        );
        Ok(1)
    }
}

pub struct ElementRemoveCommand;

impl CommandHandler for ElementRemoveCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo element <id> remove <elem_id>");
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
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

        if hologram.remove_element(&element_id).is_some() {
            let _ = manager.save_hologram(&hologram_id);
            send_feedback(
                &sender,
                &format!("&aRemoved element '&e{element_id}&a' from hologram '&e{hologram_id}&a'!"),
            );
            Ok(1)
        } else {
            send_error(
                &sender,
                &format!("Element '&e{element_id}&c' not found in hologram '&e{hologram_id}&c'!"),
            );
            Ok(0)
        }
    }
}

pub struct ElementOffsetCommand;

impl CommandHandler for ElementOffsetCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo element <id> offset <elem_id> <x> <y> <z>",
                );
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
                return Ok(0);
            }
        };

        let x = match get_string_arg(&args, "x").and_then(|v| parse_number(&v, "X offset").ok()) {
            Some(val) => val,
            None => {
                send_error(&sender, "Valid X offset number required!");
                return Ok(0);
            }
        };

        let y = match get_string_arg(&args, "y").and_then(|v| parse_number(&v, "Y offset").ok()) {
            Some(val) => val,
            None => {
                send_error(&sender, "Valid Y offset number required!");
                return Ok(0);
            }
        };

        let z = match get_string_arg(&args, "z").and_then(|v| parse_number(&v, "Z offset").ok()) {
            Some(val) => val,
            None => {
                send_error(&sender, "Valid Z offset number required!");
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

        if let Some(element) = hologram.data.get_element_mut(&element_id) {
            element.offset_x = x;
            element.offset_y = y;
            element.offset_z = z;
        } else {
            send_error(
                &sender,
                &format!("Element '&e{element_id}&c' not found in hologram '&e{hologram_id}&c'!"),
            );
            return Ok(0);
        }

        let world_name = hologram.data.world_name.clone();
        let world = crate::get_world(&world_name);
        hologram.sync_elements(world.as_ref());
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!(
                "&aUpdated offset for element '&e{element_id}&a' to &f({x:.2}, {y:.2}, {z:.2})&a!"
            ),
        );
        Ok(1)
    }
}

pub struct ElementScaleCommand;

impl CommandHandler for ElementScaleCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo element <id> scale <elem_id> <scale_or_x> [y] [z]",
                );
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
                return Ok(0);
            }
        };

        let first_val =
            match get_string_arg(&args, "scale_val").and_then(|v| parse_number(&v, "Scale").ok()) {
                Some(val) => val as f32,
                None => {
                    send_error(&sender, "Valid scale number required!");
                    return Ok(0);
                }
            };

        let scale = if let (Some(y_str), Some(z_str)) = (
            get_string_arg(&args, "scale_y"),
            get_string_arg(&args, "scale_z"),
        ) {
            match (
                parse_number(&y_str, "Scale Y"),
                parse_number(&z_str, "Scale Z"),
            ) {
                (Ok(y), Ok(z)) => (first_val, y as f32, z as f32),
                _ => (first_val, first_val, first_val),
            }
        } else {
            (first_val, first_val, first_val)
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

        if let Some(element) = hologram.data.get_element_mut(&element_id) {
            element.scale = scale;
        } else {
            send_error(
                &sender,
                &format!("Element '&e{element_id}&c' not found in hologram '&e{hologram_id}&c'!"),
            );
            return Ok(0);
        }

        let world_name = hologram.data.world_name.clone();
        let world = crate::get_world(&world_name);
        hologram.sync_elements(world.as_ref());
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!(
                "&aUpdated scale for element '&e{element_id}&a' to &f({:.2}, {:.2}, {:.2})&a!",
                scale.0, scale.1, scale.2
            ),
        );
        Ok(1)
    }
}

pub struct ElementSizeCommand;

impl CommandHandler for ElementSizeCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo element <id> size <elem_id> [width] [height]",
                );
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
                return Ok(0);
            }
        };

        let custom_width = get_string_arg(&args, "width")
            .and_then(|val| parse_number(&val, "Hitbox width").ok())
            .map(|val| val as f32);

        let custom_height = get_string_arg(&args, "height")
            .and_then(|val| parse_number(&val, "Hitbox height").ok())
            .map(|val| val as f32);

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

        let (current_width, current_height) = {
            let element = match hologram.data.get_element_mut(&element_id) {
                Some(elem) => elem,
                None => {
                    send_error(
                        &sender,
                        &format!(
                            "Element '&e{element_id}&c' not found in hologram '&e{hologram_id}&c'!"
                        ),
                    );
                    return Ok(0);
                }
            };

            if let Some(w) = custom_width {
                element.interaction_width = Some(w);
            }
            if let Some(h) = custom_height {
                element.interaction_height = Some(h);
            }
            (
                element.interaction_width.unwrap_or(0.8),
                element.interaction_height.unwrap_or(0.8),
            )
        };

        let world_name = hologram.data.world_name.clone();
        let world = crate::get_world(&world_name);
        hologram.sync_elements(world.as_ref());
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!(
                "&aHitbox dimensions for element '&e{element_id}&a': width=&e{:.2}&a, height=&e{:.2}&a!",
                current_width, current_height
            ),
        );
        Ok(1)
    }
}

pub struct ElementActionListCommand;

impl CommandHandler for ElementActionListCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo element <id> action <elem_id> list");
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
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

        let element = match hologram.data.get_element(&element_id) {
            Some(e) => e,
            None => {
                send_error(
                    &sender,
                    &format!(
                        "Element '&e{element_id}&c' not found in hologram '&e{hologram_id}&c'!"
                    ),
                );
                return Ok(0);
            }
        };

        if element.actions.is_empty() {
            send_feedback(
                &sender,
                &format!("&7Element '&e{element_id}&7' has no click actions."),
            );
            return Ok(1);
        }

        send_feedback(
            &sender,
            &format!(
                "&6=== &eActions for Element '&a{element_id}&e' in '&a{hologram_id}&e' ({}) &6===",
                element.actions.len()
            ),
        );

        for (index, action) in element.actions.iter().enumerate() {
            let (action_label, detail, click_type) = match action {
                ClickAction::PlayerCommand {
                    command,
                    click_type,
                } => ("&bplayer_command", command.as_str(), click_type),
                ClickAction::ConsoleCommand {
                    command,
                    click_type,
                } => ("&cconsole_command", command.as_str(), click_type),
                ClickAction::Message {
                    message,
                    click_type,
                } => ("&dmessage", message.as_str(), click_type),
                ClickAction::Sound {
                    sound,
                    volume,
                    pitch,
                    click_type,
                } => {
                    let formatted = format!("{sound} (vol: {volume}, pitch: {pitch})");
                    ("&esound", formatted.leak() as &str, click_type)
                }
            };

            send_feedback(
                &sender,
                &format!(
                    "&8[&e{}&8] &7[{:?}] {}: &f{}",
                    index + 1,
                    click_type,
                    action_label,
                    detail
                ),
            );
        }

        Ok(1)
    }
}

pub struct ElementActionAddCommand;

impl CommandHandler for ElementActionAddCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo element <id> action <elem_id> add <any|right|left> <type> <payload>",
                );
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
                return Ok(0);
            }
        };

        let raw_click_type = match get_string_arg(&args, "click_type") {
            Some(ct) => ct.to_ascii_lowercase(),
            _ => {
                send_error(&sender, "Click type must be: any, right, or left");
                return Ok(0);
            }
        };

        let click_type = match raw_click_type.as_str() {
            "any" => ClickType::Any,
            "right" | "interact" => ClickType::Right,
            "left" | "attack" => ClickType::Left,
            _ => {
                send_error(
                    &sender,
                    &format!("Invalid click type '{raw_click_type}'! Use: any, right, or left"),
                );
                return Ok(0);
            }
        };

        let raw_action_type = match get_string_arg(&args, "action_type") {
            Some(at) => at.to_ascii_lowercase(),
            _ => {
                send_error(
                    &sender,
                    "Action type must be: player_command, console_command, message, or sound",
                );
                return Ok(0);
            }
        };

        let payload = match get_string_arg(&args, "payload") {
            Some(p) if !p.trim().is_empty() => p.trim().to_string(),
            _ => {
                send_error(&sender, "Action payload cannot be empty!");
                return Ok(0);
            }
        };

        let action = match raw_action_type.as_str() {
            "player_command" | "player" | "command" => {
                let clean_command = payload.strip_prefix('/').unwrap_or(&payload).to_string();
                ClickAction::PlayerCommand {
                    command: clean_command,
                    click_type,
                }
            }
            "console_command" | "console" | "server" => {
                let clean_command = payload.strip_prefix('/').unwrap_or(&payload).to_string();
                ClickAction::ConsoleCommand {
                    command: clean_command,
                    click_type,
                }
            }
            "message" | "msg" | "tell" => ClickAction::Message {
                message: payload,
                click_type,
            },
            "sound" | "playsound" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                let sound_name = parts[0].to_string();
                let volume = parts
                    .get(1)
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(1.0);
                let pitch = parts
                    .get(2)
                    .and_then(|s| s.parse::<f32>().ok())
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
                    &format!(
                        "Invalid action type '{raw_action_type}'! Use: player_command, console_command, message, sound"
                    ),
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

        let total_actions = {
            let element = match hologram.data.get_element_mut(&element_id) {
                Some(e) => e,
                None => {
                    send_error(
                        &sender,
                        &format!(
                            "Element '&e{element_id}&c' not found in hologram '&e{hologram_id}&c'!"
                        ),
                    );
                    return Ok(0);
                }
            };
            element.actions.push(action);
            element.actions.len()
        };

        let world_name = hologram.data.world_name.clone();
        let world = crate::get_world(&world_name);
        hologram.sync_elements(world.as_ref());
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!(
                "&aAdded action &8[&e#{total_actions}&8]&a to element '&e{element_id}&a' in hologram '&e{hologram_id}&a'!"
            ),
        );
        Ok(1)
    }
}

pub struct ElementActionRemoveCommand;

impl CommandHandler for ElementActionRemoveCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(
                    &sender,
                    "Usage: /holo element <id> action <elem_id> remove <index>",
                );
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
                return Ok(0);
            }
        };

        let index_number = match get_int_arg(&args, "index") {
            Some(num) if num >= 1 => (num - 1) as usize,
            _ => {
                send_error(&sender, "Action index must be an integer >= 1!");
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

        let remove_result = {
            let element = match hologram.data.get_element_mut(&element_id) {
                Some(e) => e,
                None => {
                    send_error(
                        &sender,
                        &format!(
                            "Element '&e{element_id}&c' not found in hologram '&e{hologram_id}&c'!"
                        ),
                    );
                    return Ok(0);
                }
            };
            if index_number < element.actions.len() {
                Ok(element.actions.remove(index_number))
            } else {
                Err(format!(
                    "Action index {} out of bounds (total actions: {})",
                    index_number + 1,
                    element.actions.len()
                ))
            }
        };

        match remove_result {
            Ok(_) => {
                let world_name = hologram.data.world_name.clone();
                let world = crate::get_world(&world_name);
                hologram.sync_elements(world.as_ref());
                let _ = manager.save_hologram(&hologram_id);

                send_feedback(
                    &sender,
                    &format!(
                        "&aRemoved action &8[&e#{}&8]&a from element '&e{element_id}&a' in hologram '&e{hologram_id}&a'!",
                        index_number + 1
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

pub struct ElementActionClearCommand;

impl CommandHandler for ElementActionClearCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let hologram_id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo element <id> action <elem_id> clear");
                return Ok(0);
            }
        };

        let element_id = match get_string_arg(&args, "element_id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Element ID cannot be empty!");
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

        let count = {
            let element = match hologram.data.get_element_mut(&element_id) {
                Some(e) => e,
                None => {
                    send_error(
                        &sender,
                        &format!(
                            "Element '&e{element_id}&c' not found in hologram '&e{hologram_id}&c'!"
                        ),
                    );
                    return Ok(0);
                }
            };
            let c = element.actions.len();
            element.actions.clear();
            c
        };

        let world_name = hologram.data.world_name.clone();
        let world = crate::get_world(&world_name);
        hologram.sync_elements(world.as_ref());
        let _ = manager.save_hologram(&hologram_id);

        send_feedback(
            &sender,
            &format!(
                "&aCleared all &e{count}&a actions from element '&e{element_id}&a' in hologram '&e{hologram_id}&a'!"
            ),
        );
        Ok(1)
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("element")
        .execute(ElementHelpCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(ElementHelpCommand)
                .then(CommandNode::literal("list").execute(ElementListCommand))
                .then(
                    CommandNode::literal("remove")
                        .execute(ElementHelpCommand)
                        .then(
                            CommandNode::argument(
                                "element_id",
                                &ArgumentType::String(StringType::SingleWord),
                            )
                            .execute(ElementRemoveCommand),
                        ),
                )
                .then(
                    CommandNode::literal("offset")
                        .execute(ElementHelpCommand)
                        .then(
                            CommandNode::argument(
                                "element_id",
                                &ArgumentType::String(StringType::SingleWord),
                            )
                            .execute(ElementHelpCommand)
                            .then(
                                CommandNode::argument(
                                    "x",
                                    &ArgumentType::String(StringType::SingleWord),
                                )
                                .execute(ElementHelpCommand)
                                .then(
                                    CommandNode::argument(
                                        "y",
                                        &ArgumentType::String(StringType::SingleWord),
                                    )
                                    .execute(ElementHelpCommand)
                                    .then(
                                        CommandNode::argument(
                                            "z",
                                            &ArgumentType::String(StringType::SingleWord),
                                        )
                                        .execute(ElementOffsetCommand),
                                    ),
                                ),
                            ),
                        ),
                )
                .then(
                    CommandNode::literal("scale")
                        .execute(ElementHelpCommand)
                        .then(
                            CommandNode::argument(
                                "element_id",
                                &ArgumentType::String(StringType::SingleWord),
                            )
                            .execute(ElementHelpCommand)
                            .then(
                                CommandNode::argument(
                                    "scale_val",
                                    &ArgumentType::String(StringType::SingleWord),
                                )
                                .execute(ElementScaleCommand)
                                .then(
                                    CommandNode::argument(
                                        "scale_y",
                                        &ArgumentType::String(StringType::SingleWord),
                                    )
                                    .execute(ElementScaleCommand)
                                    .then(
                                        CommandNode::argument(
                                            "scale_z",
                                            &ArgumentType::String(StringType::SingleWord),
                                        )
                                        .execute(ElementScaleCommand),
                                    ),
                                ),
                            ),
                        ),
                )
                .then(
                    CommandNode::literal("size")
                        .execute(ElementHelpCommand)
                        .then(
                            CommandNode::argument(
                                "element_id",
                                &ArgumentType::String(StringType::SingleWord),
                            )
                            .execute(ElementSizeCommand)
                            .then(
                                CommandNode::argument(
                                    "width",
                                    &ArgumentType::String(StringType::SingleWord),
                                )
                                .execute(ElementSizeCommand)
                                .then(
                                    CommandNode::argument(
                                        "height",
                                        &ArgumentType::String(StringType::SingleWord),
                                    )
                                    .execute(ElementSizeCommand),
                                ),
                            ),
                        ),
                )
                .then(
                    CommandNode::literal("add")
                        .execute(ElementHelpCommand)
                        .then(
                            CommandNode::argument(
                                "element_id",
                                &ArgumentType::String(StringType::SingleWord),
                            )
                            .execute(ElementHelpCommand)
                            .then(
                                CommandNode::argument(
                                    "kind",
                                    &ArgumentType::String(StringType::SingleWord),
                                )
                                .execute(ElementHelpCommand)
                                .then(
                                    CommandNode::argument(
                                        "name",
                                        &ArgumentType::String(StringType::SingleWord),
                                    )
                                    .execute(ElementAddCommand)
                                    .then(
                                        CommandNode::argument(
                                            "offset_x",
                                            &ArgumentType::String(StringType::SingleWord),
                                        )
                                        .execute(ElementHelpCommand)
                                        .then(
                                            CommandNode::argument(
                                                "offset_y",
                                                &ArgumentType::String(StringType::SingleWord),
                                            )
                                            .execute(ElementHelpCommand)
                                            .then(
                                                CommandNode::argument(
                                                    "offset_z",
                                                    &ArgumentType::String(StringType::SingleWord),
                                                )
                                                .execute(ElementAddCommand),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                )
                .then(
                    CommandNode::literal("action")
                        .execute(ElementHelpCommand)
                        .then(
                            CommandNode::argument(
                                "element_id",
                                &ArgumentType::String(StringType::SingleWord),
                            )
                            .execute(ElementHelpCommand)
                            .then(CommandNode::literal("list").execute(ElementActionListCommand))
                            .then(CommandNode::literal("clear").execute(ElementActionClearCommand))
                            .then(
                                CommandNode::literal("remove")
                                    .execute(ElementHelpCommand)
                                    .then(
                                        CommandNode::argument(
                                            "index",
                                            &ArgumentType::Integer((Some(1), None)),
                                        )
                                        .execute(ElementActionRemoveCommand),
                                    ),
                            )
                            .then(
                                CommandNode::literal("add")
                                    .execute(ElementHelpCommand)
                                    .then(
                                        CommandNode::argument(
                                            "click_type",
                                            &ArgumentType::String(StringType::SingleWord),
                                        )
                                        .execute(ElementHelpCommand)
                                        .then(
                                            CommandNode::argument(
                                                "action_type",
                                                &ArgumentType::String(StringType::SingleWord),
                                            )
                                            .execute(ElementHelpCommand)
                                            .then(
                                                CommandNode::argument(
                                                    "payload",
                                                    &ArgumentType::String(StringType::Greedy),
                                                )
                                                .execute(ElementActionAddCommand),
                                            ),
                                        ),
                                    ),
                            ),
                        ),
                ),
        )
}
