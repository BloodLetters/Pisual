use crate::hologram::HologramData;
use pumpkin_plugin_api::{
    common::Hand,
    player::Player,
    world::World,
    Server,
};

pub struct PlaceholderContext<'a> {
    pub server: Option<&'a Server>,
    pub world: Option<&'a World>,
    pub hologram: Option<&'a HologramData>,
    pub player: Option<&'a Player>,
}

struct PlayerSnapshot {
    name: String,
    ping: u32,
    health: f32,
    max_health: f32,
    food: u8,
    saturation: f32,
    level: i32,
    exp: i32,
    gamemode: String,
    item_in_hand: String,
    pos: (f64, f64, f64),
    uuid: String,
}

fn snapshot_player(player: &Player) -> PlayerSnapshot {
    let gamemode_str = match player.get_gamemode() {
        pumpkin_plugin_api::common::GameMode::Survival => "Survival",
        pumpkin_plugin_api::common::GameMode::Creative => "Creative",
        pumpkin_plugin_api::common::GameMode::Adventure => "Adventure",
        pumpkin_plugin_api::common::GameMode::Spectator => "Spectator",
    };

    let item_str = player
        .get_item_in_hand(Hand::Right)
        .map(|stack| {
            let key = stack.get_registry_key();
            key.strip_prefix("minecraft:").unwrap_or(&key).to_string()
        })
        .unwrap_or_else(|| "Air".to_string());

    PlayerSnapshot {
        name: player.get_name(),
        ping: player.get_ping(),
        health: player.get_health(),
        max_health: player.get_max_health(),
        food: player.get_food_level(),
        saturation: player.get_saturation(),
        level: player.get_experience_level(),
        exp: player.get_experience_points(),
        gamemode: gamemode_str.to_string(),
        item_in_hand: item_str,
        pos: player.get_position(),
        uuid: player.get_id().to_string(),
    }
}

pub fn has_placeholders(text: &str) -> bool {
    text.contains('{') && text.contains('}')
}

pub fn any_has_placeholders(lines: &[String]) -> bool {
    lines.iter().any(|line| has_placeholders(line))
}

fn ticks_to_minecraft_time(ticks: u64) -> String {
    let day_ticks = (ticks + 6000) % 24000;
    let hours = (day_ticks / 1000) % 24;
    let minutes = (day_ticks % 1000) * 60 / 1000;
    format!("{hours:02}:{minutes:02}")
}

fn find_nearest_player_snapshot(
    server: Option<&Server>,
    world_name: &str,
    target_pos: (f64, f64, f64),
) -> Option<PlayerSnapshot> {
    let server = server?;
    let players = server.get_all_players();
    let mut closest_snapshot = None;
    let mut min_distance_sq = f64::MAX;

    for player in players {
        let player_world = player.get_world();
        if player_world.get_id() != world_name {
            continue;
        }
        let pos = player.get_position();
        let dx = pos.0 - target_pos.0;
        let dy = pos.1 - target_pos.1;
        let dz = pos.2 - target_pos.2;
        let distance_sq = dx * dx + dy * dy + dz * dz;

        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
            closest_snapshot = Some(snapshot_player(&player));
        }
    }

    closest_snapshot
}

fn get_or_resolve_player(
    ctx: &PlaceholderContext,
    cached_player: &mut Option<PlayerSnapshot>,
    checked: &mut bool,
) -> Option<PlayerSnapshot> {
    if !*checked {
        *checked = true;
        if let Some(p) = ctx.player {
            *cached_player = Some(snapshot_player(p));
        } else if let Some(holo) = ctx.hologram {
            *cached_player = find_nearest_player_snapshot(ctx.server, &holo.world_name, holo.position);
        }
    }
    cached_player.as_ref().map(|p| PlayerSnapshot {
        name: p.name.clone(),
        ping: p.ping,
        health: p.health,
        max_health: p.max_health,
        food: p.food,
        saturation: p.saturation,
        level: p.level,
        exp: p.exp,
        gamemode: p.gamemode.clone(),
        item_in_hand: p.item_in_hand.clone(),
        pos: p.pos,
        uuid: p.uuid.clone(),
    })
}

fn resolve_placeholder(
    key: &str,
    ctx: &PlaceholderContext,
    cached_player: &mut Option<PlayerSnapshot>,
    checked: &mut bool,
) -> Option<String> {
    match key.to_ascii_lowercase().as_str() {
        "player_name" | "player" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| player.name).unwrap_or_else(|| "None".to_string()))
        }
        "ping" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| player.ping.to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "health" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| format!("{:.1}", player.health)).unwrap_or_else(|| "0.0".to_string()))
        }
        "max_health" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| format!("{:.1}", player.max_health)).unwrap_or_else(|| "0.0".to_string()))
        }
        "food" | "food_level" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| player.food.to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "saturation" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| format!("{:.1}", player.saturation)).unwrap_or_else(|| "0.0".to_string()))
        }
        "level" | "exp_level" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| player.level.to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "exp" | "points" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| player.exp.to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "gamemode" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| player.gamemode).unwrap_or_else(|| "None".to_string()))
        }
        "item_in_hand" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| player.item_in_hand).unwrap_or_else(|| "Air".to_string()))
        }
        "player_x" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| format!("{:.2}", player.pos.0)).unwrap_or_else(|| "0.00".to_string()))
        }
        "player_y" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| format!("{:.2}", player.pos.1)).unwrap_or_else(|| "0.00".to_string()))
        }
        "player_z" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| format!("{:.2}", player.pos.2)).unwrap_or_else(|| "0.00".to_string()))
        }
        "player_uuid" | "uuid" => {
            let p = get_or_resolve_player(ctx, cached_player, checked);
            Some(p.map(|player| player.uuid).unwrap_or_else(|| "None".to_string()))
        }
        "online_players" | "online" => {
            Some(ctx.server.map(|s| s.get_player_count().to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "max_players" | "max" => {
            Some(ctx.server.map(|s| s.get_max_players().to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "tps" => {
            Some(ctx.server.map(|s| format!("{:.2}", s.get_tps())).unwrap_or_else(|| "20.00".to_string()))
        }
        "mspt" => {
            Some(ctx.server.map(|s| format!("{:.2}", s.get_mspt())).unwrap_or_else(|| "0.00".to_string()))
        }
        "server_motd" | "motd" => {
            Some(ctx.server.map(|s| s.get_motd()).unwrap_or_default())
        }
        "server_difficulty" | "difficulty" => {
            Some(
                ctx.server
                    .map(|s| match s.get_difficulty() {
                        pumpkin_plugin_api::server::Difficulty::Peaceful => "Peaceful".to_string(),
                        pumpkin_plugin_api::server::Difficulty::Easy => "Easy".to_string(),
                        pumpkin_plugin_api::server::Difficulty::Normal => "Normal".to_string(),
                        pumpkin_plugin_api::server::Difficulty::Hard => "Hard".to_string(),
                    })
                    .unwrap_or_else(|| "Normal".to_string()),
            )
        }
        "server_version" | "version" => {
            Some(
                ctx.server
                    .map(|s| s.get_sys_info().pumpkin_version)
                    .unwrap_or_else(|| "Pumpkin".to_string()),
            )
        }
        "memory_used" => {
            Some(
                ctx.server
                    .and_then(|s| s.get_sys_info().used_memory)
                    .map(|bytes| format!("{} MB", bytes / 1_048_576))
                    .unwrap_or_else(|| "N/A".to_string()),
            )
        }
        "memory_total" | "memory_max" => {
            Some(
                ctx.server
                    .and_then(|s| s.get_sys_info().total_memory)
                    .map(|bytes| format!("{} MB", bytes / 1_048_576))
                    .unwrap_or_else(|| "N/A".to_string()),
            )
        }
        "cpu_count" => {
            Some(
                ctx.server
                    .and_then(|s| s.get_sys_info().cpu_count)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
            )
        }
        "total_holograms" | "holograms" => {
            Some(
                crate::get_manager()
                    .read()
                    .map(|mgr| mgr.count().to_string())
                    .unwrap_or_else(|_| "0".to_string()),
            )
        }
        "world" | "world_name" => {
            Some(
                ctx.world
                    .map(|w| w.get_id())
                    .or_else(|| ctx.hologram.map(|h| h.world_name.clone()))
                    .unwrap_or_else(|| "world".to_string()),
            )
        }
        "world_time" | "time" => {
            Some(ctx.world.map(|w| w.get_time_of_day().to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "world_age" => {
            Some(ctx.world.map(|w| w.get_world_age().to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "mc_time" => {
            Some(
                ctx.world
                    .map(|w| ticks_to_minecraft_time(w.get_time_of_day()))
                    .unwrap_or_else(|| "06:00".to_string()),
            )
        }
        "mc_day" => {
            Some(
                ctx.world
                    .map(|w| (w.get_world_age() / 24_000).to_string())
                    .unwrap_or_else(|| "0".to_string()),
            )
        }
        "world_players" => {
            Some(
                if let (Some(server), Some(world)) = (ctx.server, ctx.world) {
                    if let Some(w) = server.get_world_by_name(&world.get_id()) {
                        server.get_player_count_in_world(w).to_string()
                    } else {
                        "0".to_string()
                    }
                } else {
                    "0".to_string()
                },
            )
        }
        "is_raining" => {
            Some(ctx.world.map(|w| w.is_raining().to_string()).unwrap_or_else(|| "false".to_string()))
        }
        "is_thundering" => {
            Some(ctx.world.map(|w| w.is_thundering().to_string()).unwrap_or_else(|| "false".to_string()))
        }
        "holo_id" | "hologram_id" => {
            Some(ctx.hologram.map(|h| h.id.clone()).unwrap_or_default())
        }
        "x" => {
            Some(ctx.hologram.map(|h| format!("{:.2}", h.position.0)).unwrap_or_default())
        }
        "y" => {
            Some(ctx.hologram.map(|h| format!("{:.2}", h.position.1)).unwrap_or_default())
        }
        "z" => {
            Some(ctx.hologram.map(|h| format!("{:.2}", h.position.2)).unwrap_or_default())
        }
        "line_count" => {
            Some(ctx.hologram.map(|h| h.lines.len().to_string()).unwrap_or_else(|| "0".to_string()))
        }
        "billboard" => {
            Some(ctx.hologram.map(|h| format!("{:?}", h.billboard)).unwrap_or_default())
        }
        "view_range" => {
            Some(ctx.hologram.map(|h| format!("{:.1}", h.view_range)).unwrap_or_default())
        }
        _ => None,
    }
}

pub fn resolve(template: &str, ctx: &PlaceholderContext) -> String {
    if !has_placeholders(template) {
        return template.to_string();
    }

    let mut cached_player: Option<PlayerSnapshot> = None;
    let mut checked = false;

    let mut result = String::with_capacity(template.len());
    let mut remaining = template;

    while let Some(start_idx) = remaining.find('{') {
        result.push_str(&remaining[..start_idx]);
        let after_start = &remaining[start_idx + 1..];

        if let Some(end_idx) = after_start.find('}') {
            let candidate_key = &after_start[..end_idx];
            if !candidate_key.contains('{') {
                if let Some(val) = resolve_placeholder(candidate_key, ctx, &mut cached_player, &mut checked) {
                    result.push_str(&val);
                } else {
                    result.push('{');
                    result.push_str(candidate_key);
                    result.push('}');
                }
                remaining = &after_start[end_idx + 1..];
                continue;
            }
        }

        result.push('{');
        remaining = after_start;
    }

    result.push_str(remaining);
    result
}
