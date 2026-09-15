use pumpkin_plugin_api::{
    events::{
        player::{PlayerChangedWorldEvent, PlayerJoinEvent, PlayerRespawnEvent},
        EventHandler, EventPriority,
    },
    events_wit::{PlayerChangedWorldEventData, PlayerJoinEventData, PlayerRespawnEventData},
    scheduler::SchedulerExt,
    Context, Server,
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
    Ok(())
}
