use pumpkin_plugin_api::{
    command::{
        Arg, CommandError, CommandSender, CommandSuggestion, CommandSuggestions, ConsumedArgs,
        SuggestionRequest,
    },
    commands::CommandSuggestionHandler,
    command_wit::Number,
    player::Player,
    text::TextComponent,
    Server,
};

pub const PREFIX: &str = "&6[&ePisual&6]&r ";

pub fn send_feedback(sender: &CommandSender, message: &str) {
    let formatted = format!("{PREFIX}{message}");
    let component = TextComponent::from_legacy_string_with_code(&formatted, '&');
    sender.send_message(component);
}

pub fn send_error(sender: &CommandSender, message: &str) {
    let formatted = format!("{PREFIX}&c{message}");
    let component = TextComponent::from_legacy_string_with_code(&formatted, '&');
    sender.send_message(component);
}

pub fn get_string_arg(args: &ConsumedArgs, key: &str) -> Option<String> {
    match args.get_value(key) {
        Arg::Simple(s) | Arg::Msg(s) => {
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        }
        _ => None,
    }
}

pub fn get_int_arg(args: &ConsumedArgs, key: &str) -> Option<i32> {
    match args.get_value(key) {
        Arg::Num(Ok(Number::Int32(n))) => Some(n),
        _ => None,
    }
}

pub fn is_player(sender: &CommandSender) -> Result<Player, CommandError> {
    sender.as_player().ok_or_else(|| {
        send_error(sender, "This command can only be executed by a player in-game!");
        CommandError::CommandFailed(TextComponent::text("Only players can execute this command"))
    })
}

pub fn calculate_facing_direction(
    origin_position: (f64, f64, f64),
    target_position: (f64, f64, f64),
    fallback_yaw: f32,
) -> (f32, f32) {
    let delta_x = target_position.0 - origin_position.0;
    let delta_y = target_position.1 - origin_position.1;
    let delta_z = target_position.2 - origin_position.2;
    let horizontal_distance = (delta_x * delta_x + delta_z * delta_z).sqrt();

    if horizontal_distance > 0.05 {
        let yaw = (-delta_x).atan2(delta_z).to_degrees() as f32;
        let pitch = (-delta_y).atan2(horizontal_distance).to_degrees() as f32;
        (yaw, pitch)
    } else {
        ((fallback_yaw + 180.0) % 360.0, 0.0)
    }
}

pub struct HologramStorageSuggestions;

impl CommandSuggestionHandler for HologramStorageSuggestions {
    fn suggest(
        &self,
        _sender: CommandSender,
        _server: Server,
        request: SuggestionRequest,
    ) -> CommandSuggestions {
        let prefix = &request.remaining;
        let mut identifier_list = Vec::new();

        if let Ok(manager) = crate::get_manager().read() {
            if let Ok(stored_items) = crate::storage::load_from_disk(manager.data_folder()) {
                for item in stored_items {
                    identifier_list.push(item.id);
                }
            }
            for hologram in manager.list() {
                if !identifier_list.contains(&hologram.data.id) {
                    identifier_list.push(hologram.data.id.clone());
                }
            }
        }

        let values = identifier_list
            .into_iter()
            .filter(|id| id.starts_with(prefix))
            .map(|id| CommandSuggestion {
                value: id,
                tooltip: None,
            })
            .collect();

        CommandSuggestions {
            start: request.start,
            length: request.remaining.len() as u32,
            values,
        }
    }
}
