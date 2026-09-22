use super::utils::{
    HologramStorageSuggestions, get_int_arg, get_string_arg, send_error, send_feedback,
};
use pumpkin_plugin_api::{
    Server,
    command::{ArgumentType, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType},
    commands::CommandHandler,
};

pub struct IntervalCommand;

impl CommandHandler for IntervalCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let id = match get_string_arg(&args, "id") {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                send_error(&sender, "Usage: /holo interval <id> [ticks]");
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

        if let Some(ticks) = get_int_arg(&args, "ticks") {
            if ticks < 0 {
                send_error(
                    &sender,
                    "Interval ticks must be non-negative (0 to disable)!",
                );
                return Ok(0);
            }

            let interval = Some(ticks as u64);
            match mgr.set_refresh_interval(&id, interval) {
                Ok(_) => {
                    if ticks == 0 {
                        send_feedback(
                            &sender,
                            &format!("&aAuto-refresh disabled for hologram '&e{id}&a'."),
                        );
                    } else {
                        let seconds = ticks as f64 / 20.0;
                        send_feedback(
                            &sender,
                            &format!(
                                "&aSet auto-refresh interval for hologram '&e{id}&a' to &e{ticks}&a ticks (&e{seconds:.1}s&a)!"
                            ),
                        );
                    }
                    Ok(1)
                }
                Err(e) => {
                    send_error(&sender, &e);
                    Ok(0)
                }
            }
        } else {
            let holo = match mgr.get(&id) {
                Some(h) => h,
                None => {
                    send_error(&sender, &format!("Hologram '&e{id}&c' was not found!"));
                    return Ok(0);
                }
            };

            match holo.data.refresh_interval {
                Some(0) => {
                    send_feedback(
                        &sender,
                        &format!(
                            "&7Auto-refresh for hologram '&e{id}&7' is currently &cDisabled&7."
                        ),
                    );
                }
                Some(n) => {
                    let seconds = n as f64 / 20.0;
                    send_feedback(
                        &sender,
                        &format!(
                            "&7Auto-refresh for hologram '&e{id}&7' is configured to &e{n}&7 ticks (&e{seconds:.1}s&7)."
                        ),
                    );
                }
                None => {
                    let has_vars = holo.has_placeholders();
                    if has_vars {
                        send_feedback(
                            &sender,
                            &format!(
                                "&7Hologram '&e{id}&7' uses dynamic placeholders with default refresh (&e20&7 ticks / &e1.0s&7)."
                            ),
                        );
                    } else {
                        send_feedback(
                            &sender,
                            &format!(
                                "&7Hologram '&e{id}&7' contains no placeholders. Auto-refresh is currently inactive."
                            ),
                        );
                    }
                }
            }
            Ok(1)
        }
    }
}

pub fn build_node() -> CommandNode {
    CommandNode::literal("interval")
        .execute(IntervalCommand)
        .then(
            CommandNode::argument("id", &ArgumentType::String(StringType::SingleWord))
                .suggest(HologramStorageSuggestions)
                .execute(IntervalCommand)
                .then(
                    CommandNode::argument("ticks", &ArgumentType::Integer((Some(0), None)))
                        .execute(IntervalCommand),
                ),
        )
}
