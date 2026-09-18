use super::model::{HologramData, VisualType};

use pumpkin_plugin_api::{
    EntityType,
    block::BlockTypeExt,
    display::{DisplayEntityExt, EntityDisplayExt, TextAlignment},
    item_stack::ItemStack,
    text::TextComponent,
    world::{Block, Entity, World},
};

pub fn render_lines(data: &HologramData, world: Option<&World>) -> String {
    if data.lines.is_empty() {
        return String::new();
    }
    crate::with_server(|server| {
        let ctx = crate::placeholder::PlaceholderContext {
            server: Some(server),
            world,
            hologram: Some(data),
            player: None,
        };
        let lines: Vec<String> = data
            .lines
            .iter()
            .map(|l| crate::placeholder::resolve(l, &ctx))
            .collect();
        lines.join("\n")
    })
    .unwrap_or_else(|| {
        let ctx = crate::placeholder::PlaceholderContext {
            server: None,
            world,
            hologram: Some(data),
            player: None,
        };
        let lines: Vec<String> = data
            .lines
            .iter()
            .map(|l| crate::placeholder::resolve(l, &ctx))
            .collect();
        lines.join("\n")
    })
}

pub fn update_text(entity: &Entity, text: &str) -> bool {
    if let Some(text_display) = entity.as_text_display() {
        let component = TextComponent::from_legacy_string_with_code(text, '&');
        text_display.set_text(component);
        true
    } else {
        false
    }
}

pub fn spawn_display(world: &World, data: &HologramData) -> Option<Entity> {
    let entity = world.spawn_entity(EntityType::TextDisplay, data.position);

    if let Some(text_display) = entity.as_text_display() {
        apply_properties(&text_display, data, Some(world));
        Some(entity)
    } else {
        crate::logger::error(&format!(
            "Failed to downcast spawned entity to TextDisplay for hologram '{}'",
            data.id
        ));
        entity.remove();
        None
    }
}

pub fn update_display(entity: &Entity, data: &HologramData, world: Option<&World>) -> bool {
    if let Some(text_display) = entity.as_text_display() {
        apply_properties(&text_display, data, world);
        true
    } else {
        false
    }
}

pub fn teleport_display(entity: &Entity, pos: (f64, f64, f64), world: World) {
    entity.teleport(pos, world);
}

pub fn despawn_display(entity: &Entity) {
    entity.remove();
}

pub fn calculate_visual_position(
    base_position: (f64, f64, f64),
    data: &HologramData,
) -> (f64, f64, f64) {
    if let Some(configuration) = &data.visual {
        (
            base_position.0 + configuration.offset_x,
            base_position.1 + configuration.offset_y,
            base_position.2 + configuration.offset_z,
        )
    } else {
        (
            base_position.0 - 0.1,
            base_position.1 + 0.8,
            base_position.2 - 0.1,
        )
    }
}

pub fn spawn_visual(world: &World, data: &HologramData) -> Option<Entity> {
    let visual = data.visual.as_ref()?;
    let visual_position = calculate_visual_position(data.position, data);

    let spawned_entity = match &visual.visual_type {
        VisualType::Item { item } => {
            let entity = world.spawn_entity(EntityType::ItemDisplay, visual_position);
            if let Some(item_display) = entity.as_item_display() {
                apply_item_properties(&item_display, item, data, visual.scale);
                Some(entity)
            } else {
                crate::logger::error(&format!(
                    "Failed to downcast spawned entity to ItemDisplay for hologram '{}'",
                    data.id
                ));
                entity.remove();
                None
            }
        }
        VisualType::Block { block } => {
            let entity = world.spawn_entity(EntityType::BlockDisplay, visual_position);
            if let Some(block_display) = entity.as_block_display() {
                apply_block_properties(&block_display, block, data, visual.scale);
                Some(entity)
            } else {
                crate::logger::error(&format!(
                    "Failed to downcast spawned entity to BlockDisplay for hologram '{}'",
                    data.id
                ));
                entity.remove();
                None
            }
        }
        VisualType::Entity { entity_type } => {
            let parsed_type = parse_entity_type(entity_type)?;
            let entity = world.spawn_entity(parsed_type, visual_position);
            apply_frozen_living_properties(&entity);
            Some(entity)
        }
    }?;

    if let Some((yaw, pitch)) = visual.rotation {
        spawned_entity.set_rotation(yaw, pitch);
    }

    Some(spawned_entity)
}

fn apply_item_properties(
    item_display: &pumpkin_plugin_api::display::ItemDisplayEntity,
    item_name: &str,
    data: &HologramData,
    scale: (f32, f32, f32),
) {
    let key = normalize_identifier(item_name);
    let stack = ItemStack::new(&key, 1);
    item_display.set_item(Some(stack));

    let display = item_display.get_display();
    display.set_billboard(data.billboard.into());
    display.set_view_range(data.view_range);
    display.set_scale(scale.0, scale.1, scale.2);
}

fn apply_block_properties(
    block_display: &pumpkin_plugin_api::display::BlockDisplayEntity,
    block_identifier: &str,
    data: &HologramData,
    scale: (f32, f32, f32),
) {
    let state_id = if let Ok(parsed_id) = block_identifier.parse::<u16>() {
        Some(parsed_id)
    } else {
        let key = normalize_identifier(block_identifier);
        Block::of(&key).map(|block| block.default_state_id)
    };

    if let Some(valid_state_id) = state_id {
        block_display.set_block_state_id(valid_state_id);
    }

    let display = block_display.get_display();
    display.set_billboard(data.billboard.into());
    display.set_view_range(data.view_range);
    display.set_scale(scale.0, scale.1, scale.2);
}

fn apply_frozen_living_properties(entity: &Entity) {
    entity.set_has_gravity(false);
    entity.set_invulnerable(true);
    entity.set_silent(true);
    entity.set_velocity((0.0, 0.0, 0.0));

    if let Some(mob) = entity.as_mob() {
        mob.set_ai_disabled(true);
        mob.clear_ai_goals();
        mob.set_target(None);
    }
}

fn normalize_identifier(raw_identifier: &str) -> String {
    let trimmed = raw_identifier.trim();
    if trimmed.contains(':') {
        trimmed.to_string()
    } else {
        format!("minecraft:{trimmed}")
    }
}

pub fn parse_entity_type(name: &str) -> Option<EntityType> {
    let clean_name = name
        .trim()
        .strip_prefix("minecraft:")
        .unwrap_or(name.trim())
        .to_ascii_lowercase()
        .replace('-', "_");

    match clean_name.as_str() {
        "allay" => Some(EntityType::Allay),
        "armadillo" => Some(EntityType::Armadillo),
        "armor_stand" => Some(EntityType::ArmorStand),
        "arrow" => Some(EntityType::Arrow),
        "axolotl" => Some(EntityType::Axolotl),
        "bat" => Some(EntityType::Bat),
        "bee" => Some(EntityType::Bee),
        "blaze" => Some(EntityType::Blaze),
        "block_display" => Some(EntityType::BlockDisplay),
        "bogged" => Some(EntityType::Bogged),
        "breeze" => Some(EntityType::Breeze),
        "camel" => Some(EntityType::Camel),
        "cat" => Some(EntityType::Cat),
        "cave_spider" => Some(EntityType::CaveSpider),
        "chicken" => Some(EntityType::Chicken),
        "cod" => Some(EntityType::Cod),
        "copper_golem" => Some(EntityType::CopperGolem),
        "cow" => Some(EntityType::Cow),
        "creaking" => Some(EntityType::Creaking),
        "creeper" => Some(EntityType::Creeper),
        "dolphin" => Some(EntityType::Dolphin),
        "donkey" => Some(EntityType::Donkey),
        "drowned" => Some(EntityType::Drowned),
        "elder_guardian" => Some(EntityType::ElderGuardian),
        "end_crystal" => Some(EntityType::EndCrystal),
        "ender_dragon" => Some(EntityType::EnderDragon),
        "enderman" => Some(EntityType::Enderman),
        "endermite" => Some(EntityType::Endermite),
        "evoker" => Some(EntityType::Evoker),
        "fox" => Some(EntityType::Fox),
        "frog" => Some(EntityType::Frog),
        "ghast" => Some(EntityType::Ghast),
        "giant" => Some(EntityType::Giant),
        "glow_squid" => Some(EntityType::GlowSquid),
        "goat" => Some(EntityType::Goat),
        "guardian" => Some(EntityType::Guardian),
        "happy_ghast" => Some(EntityType::HappyGhast),
        "hoglin" => Some(EntityType::Hoglin),
        "horse" => Some(EntityType::Horse),
        "husk" => Some(EntityType::Husk),
        "illusioner" => Some(EntityType::Illusioner),
        "interaction" => Some(EntityType::Interaction),
        "iron_golem" => Some(EntityType::IronGolem),
        "item" => Some(EntityType::Item),
        "item_display" => Some(EntityType::ItemDisplay),
        "item_frame" => Some(EntityType::ItemFrame),
        "llama" => Some(EntityType::Llama),
        "magma_cube" => Some(EntityType::MagmaCube),
        "marker" => Some(EntityType::Marker),
        "mooshroom" => Some(EntityType::Mooshroom),
        "mule" => Some(EntityType::Mule),
        "nautilus" => Some(EntityType::Nautilus),
        "ocelot" => Some(EntityType::Ocelot),
        "panda" => Some(EntityType::Panda),
        "parrot" => Some(EntityType::Parrot),
        "phantom" => Some(EntityType::Phantom),
        "pig" => Some(EntityType::Pig),
        "piglin" => Some(EntityType::Piglin),
        "piglin_brute" => Some(EntityType::PiglinBrute),
        "pillager" => Some(EntityType::Pillager),
        "player" => Some(EntityType::Player),
        "polar_bear" => Some(EntityType::PolarBear),
        "pufferfish" => Some(EntityType::Pufferfish),
        "rabbit" => Some(EntityType::Rabbit),
        "ravager" => Some(EntityType::Ravager),
        "salmon" => Some(EntityType::Salmon),
        "sheep" => Some(EntityType::Sheep),
        "shulker" => Some(EntityType::Shulker),
        "silverfish" => Some(EntityType::Silverfish),
        "skeleton" => Some(EntityType::Skeleton),
        "skeleton_horse" => Some(EntityType::SkeletonHorse),
        "slime" => Some(EntityType::Slime),
        "sniffer" => Some(EntityType::Sniffer),
        "snow_golem" => Some(EntityType::SnowGolem),
        "spider" => Some(EntityType::Spider),
        "squid" => Some(EntityType::Squid),
        "stray" => Some(EntityType::Stray),
        "strider" => Some(EntityType::Strider),
        "tadpole" => Some(EntityType::Tadpole),
        "text_display" => Some(EntityType::TextDisplay),
        "trader_llama" => Some(EntityType::TraderLlama),
        "tropical_fish" => Some(EntityType::TropicalFish),
        "turtle" => Some(EntityType::Turtle),
        "vex" => Some(EntityType::Vex),
        "villager" => Some(EntityType::Villager),
        "vindicator" => Some(EntityType::Vindicator),
        "wandering_trader" => Some(EntityType::WanderingTrader),
        "warden" => Some(EntityType::Warden),
        "witch" => Some(EntityType::Witch),
        "wither" => Some(EntityType::Wither),
        "wither_skeleton" => Some(EntityType::WitherSkeleton),
        "wolf" => Some(EntityType::Wolf),
        "zoglin" => Some(EntityType::Zoglin),
        "zombie" => Some(EntityType::Zombie),
        "zombie_horse" => Some(EntityType::ZombieHorse),
        "zombie_villager" => Some(EntityType::ZombieVillager),
        "zombified_piglin" => Some(EntityType::ZombifiedPiglin),
        _ => None,
    }
}

fn apply_properties(
    text_display: &pumpkin_plugin_api::display::TextDisplayEntity,
    data: &HologramData,
    world: Option<&World>,
) {
    let joined_text = render_lines(data, world);
    let component = TextComponent::from_legacy_string_with_code(&joined_text, '&');
    text_display.set_text(component);

    text_display.set_alignment(TextAlignment::Center);
    text_display.set_shadow(data.shadow);
    text_display.set_see_through(data.see_through);

    if let Some(bg_color) = data.background {
        text_display.set_background(bg_color);
    } else {
        text_display.set_default_background(false);
    }

    let display = text_display.get_display();
    display.set_billboard(data.billboard.into());
    display.set_view_range(data.view_range);
    display.set_scale(data.scale.0, data.scale.1, data.scale.2);
}
