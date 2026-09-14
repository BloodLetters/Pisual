use super::model::HologramData;

use pumpkin_plugin_api::{
    display::{DisplayEntityExt, EntityDisplayExt, TextAlignment},
    text::TextComponent,
    world::{Entity, World},
    EntityType,
};

pub fn spawn_display(world: &World, data: &HologramData) -> Option<Entity> {
    let entity = world.spawn_entity(EntityType::TextDisplay, data.position);

    if let Some(text_display) = entity.as_text_display() {
        apply_properties(&text_display, data);
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

pub fn update_display(entity: &Entity, data: &HologramData) -> bool {
    if let Some(text_display) = entity.as_text_display() {
        apply_properties(&text_display, data);
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

fn apply_properties(
    text_display: &pumpkin_plugin_api::display::TextDisplayEntity,
    data: &HologramData,
) {
    let joined_text = if data.lines.is_empty() {
        String::new()
    } else {
        data.lines.join("\n")
    };
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
