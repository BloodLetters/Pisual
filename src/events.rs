use pumpkin_plugin_api::{
    Context, Server,
    events::{
        EventHandler, EventPriority,
        player::{
            PlayerChangedWorldEvent, PlayerInteractEntityEvent, PlayerJoinEvent, PlayerRespawnEvent,
        },
    },
    events_wit::{
        EntityInteractionAction, PlayerChangedWorldEventData, PlayerInteractEntityEventData,
        PlayerJoinEventData, PlayerRespawnEventData,
    },
    player::SoundCategory,
    scheduler::SchedulerExt,
    server::CommandSender,
    text::TextComponent,
};

pub fn schedule_hologram_refresh(server: &Server, delay_in_ticks: u64) {
    server.schedule_delayed_task(delay_in_ticks, |server_instance| {
        if let Ok(mut manager) = crate::get_manager().write() {
            manager.respawn_all(|world_name| server_instance.get_world_by_name(world_name));
        }
    });
}

pub struct PlayerJoinEventListener;

impl EventHandler<PlayerJoinEvent> for PlayerJoinEventListener {
    fn handle(&self, server: Server, event: PlayerJoinEventData) -> PlayerJoinEventData {
        schedule_hologram_refresh(&server, 1);
        schedule_hologram_refresh(&server, 10);
        event
    }
}

pub struct PlayerChangedWorldEventListener;

impl EventHandler<PlayerChangedWorldEvent> for PlayerChangedWorldEventListener {
    fn handle(
        &self,
        server: Server,
        event: PlayerChangedWorldEventData,
    ) -> PlayerChangedWorldEventData {
        schedule_hologram_refresh(&server, 1);
        schedule_hologram_refresh(&server, 10);
        event
    }
}

pub struct PlayerRespawnEventListener;

impl EventHandler<PlayerRespawnEvent> for PlayerRespawnEventListener {
    fn handle(&self, server: Server, event: PlayerRespawnEventData) -> PlayerRespawnEventData {
        schedule_hologram_refresh(&server, 1);
        schedule_hologram_refresh(&server, 10);
        event
    }
}

pub struct PlayerInteractEntityEventListener;

impl EventHandler<PlayerInteractEntityEvent> for PlayerInteractEntityEventListener {
    fn handle(
        &self,
        server: Server,
        mut event: PlayerInteractEntityEventData,
    ) -> PlayerInteractEntityEventData {
        let interaction_id = event.entity_id;
        let is_attack = matches!(event.action, EntityInteractionAction::Attack);

        let matched_interaction = if let Ok(manager) = crate::get_manager().read() {
            manager.list().into_iter().find_map(|hologram| {
                let matches_root_interaction = hologram
                    .interaction_entity
                    .as_ref()
                    .is_some_and(|entity| entity.get_id() as i32 == interaction_id);
                let matches_root_visual = hologram
                    .visual_entity
                    .as_ref()
                    .is_some_and(|entity| entity.get_id() as i32 == interaction_id);

                if matches_root_interaction || matches_root_visual {
                    return Some((
                        hologram.data.id.clone(),
                        "root".to_string(),
                        hologram.data.world_name.clone(),
                        hologram.data.actions.clone(),
                    ));
                }

                for handle in &hologram.element_handles {
                    let matches_element_interaction = handle
                        .interaction_entity
                        .as_ref()
                        .is_some_and(|entity| entity.get_id() as i32 == interaction_id);
                    let matches_element_visual = handle
                        .visual_entity
                        .as_ref()
                        .is_some_and(|entity| entity.get_id() as i32 == interaction_id);

                    if matches_element_interaction || matches_element_visual {
                        let element_actions = hologram
                            .data
                            .get_element(&handle.element_id)
                            .map(|element| element.actions.clone())
                            .unwrap_or_default();

                        return Some((
                            hologram.data.id.clone(),
                            handle.element_id.clone(),
                            hologram.data.world_name.clone(),
                            element_actions,
                        ));
                    }
                }

                None
            })
        } else {
            None
        };

        if let Some((hologram_id, target_part, world_name, click_actions)) = matched_interaction {
            event.cancelled = true;

            let player_key = (
                event.player.get_id().to_string(),
                format!("{hologram_id}:{target_part}"),
            );
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();

            let can_click = {
                static LAST_CLICKS: std::sync::Mutex<
                    Option<std::collections::HashMap<(String, String), u128>>,
                > = std::sync::Mutex::new(None);
                let mut guard = LAST_CLICKS
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let cache = guard.get_or_insert_with(std::collections::HashMap::new);

                if cache.len() > 1000 {
                    cache.retain(|_, &mut recorded_time| {
                        current_time.saturating_sub(recorded_time) < 10_000
                    });
                }

                if let Some(last_time) = cache.get(&player_key) {
                    if current_time.saturating_sub(*last_time) < 250 {
                        false
                    } else {
                        cache.insert(player_key, current_time);
                        true
                    }
                } else {
                    cache.insert(player_key, current_time);
                    true
                }
            };

            if !can_click {
                return event;
            }

            let world = server.get_world_by_name(&world_name);
            let placeholder_context = crate::placeholder::PlaceholderContext {
                server: Some(&server),
                world: world.as_ref(),
                hologram: None,
                player: Some(&event.player),
            };

            for action in click_actions {
                if !action.matches_click(is_attack) {
                    continue;
                }

                match action {
                    crate::hologram::ClickAction::PlayerCommand { command, .. } => {
                        let resolved_command =
                            crate::placeholder::resolve(&command, &placeholder_context);
                        let player_name = event.player.get_name();
                        if let Some(target_player) = server.get_player_by_name(&player_name) {
                            server.execute_command(
                                &resolved_command,
                                CommandSender::Player(target_player),
                            );
                        }
                    }
                    crate::hologram::ClickAction::ConsoleCommand { command, .. } => {
                        let resolved_command =
                            crate::placeholder::resolve(&command, &placeholder_context);
                        server.execute_command(&resolved_command, CommandSender::Console);
                    }
                    crate::hologram::ClickAction::Message { message, .. } => {
                        let resolved_message =
                            crate::placeholder::resolve(&message, &placeholder_context);
                        let component =
                            TextComponent::from_legacy_string_with_code(&resolved_message, '&');
                        event.player.send_system_message(component, false);
                    }
                    crate::hologram::ClickAction::Sound {
                        sound,
                        volume,
                        pitch,
                        ..
                    } => {
                        event.player.play_custom_sound(
                            &sound,
                            SoundCategory::Master,
                            volume,
                            pitch,
                        );
                    }
                }
            }
        }

        event
    }
}

pub fn register_event_listeners(context: &Context) -> pumpkin_plugin_api::Result<()> {
    context.register_event_handler::<PlayerJoinEvent, _>(
        PlayerJoinEventListener,
        EventPriority::Normal,
        false,
    )?;
    context.register_event_handler::<PlayerChangedWorldEvent, _>(
        PlayerChangedWorldEventListener,
        EventPriority::Normal,
        false,
    )?;
    context.register_event_handler::<PlayerRespawnEvent, _>(
        PlayerRespawnEventListener,
        EventPriority::Normal,
        false,
    )?;
    context.register_event_handler::<PlayerInteractEntityEvent, _>(
        PlayerInteractEntityEventListener,
        EventPriority::Normal,
        true,
    )?;
    Ok(())
}
