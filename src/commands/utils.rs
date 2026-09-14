use pumpkin_plugin_api::{
    command::{Arg, CommandError, CommandSender, ConsumedArgs},
    command_wit::Number,
    player::Player,
    text::TextComponent,
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
        Arg::Simple(s) | Arg::Msg(s) => Some(s),
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
