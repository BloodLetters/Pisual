use super::request::IpcRequest;
use super::response::IpcResponse;
use crate::hologram::HologramData;

pub fn handle_message(_sender: &str, payload: &[u8]) -> Result<Vec<u8>, String> {
    let request: IpcRequest = match serde_json::from_slice(payload) {
        Ok(req) => req,
        Err(err) => {
            let response = IpcResponse::error(format!("Invalid request payload: {err}"));
            return Ok(response.to_bytes());
        }
    };

    let response = process_request(request);
    Ok(response.to_bytes())
}

fn process_request(request: IpcRequest) -> IpcResponse {
    match request {
        IpcRequest::Create {
            id,
            world,
            position,
            lines,
            ram,
            billboard,
            shadow,
            see_through,
            scale,
        } => {
            let mut manager = match crate::get_manager().write() {
                Ok(guard) => guard,
                Err(_) => return IpcResponse::error("Failed to acquire write lock on manager"),
            };

            let world_instance = crate::get_world(&world);
            let mut data = HologramData::new(id.clone(), world, position, lines);
            data.is_ram = ram;
            if let Some(billboard_mode) = billboard {
                data.billboard = billboard_mode;
            }
            if let Some(has_shadow) = shadow {
                data.shadow = has_shadow;
            }
            if let Some(is_see_through) = see_through {
                data.see_through = is_see_through;
            }
            if let Some(custom_scale) = scale {
                data.scale = custom_scale;
            }

            match manager.create_hologram(data, world_instance.as_ref()) {
                Ok(_) => IpcResponse::success(format!("Hologram '{id}' created successfully")),
                Err(err) => IpcResponse::error(err),
            }
        }
        IpcRequest::CreateRam {
            id,
            world,
            position,
            lines,
            billboard,
            shadow,
            see_through,
            scale,
        } => {
            let mut manager = match crate::get_manager().write() {
                Ok(guard) => guard,
                Err(_) => return IpcResponse::error("Failed to acquire write lock on manager"),
            };

            let world_instance = crate::get_world(&world);
            let mut data = HologramData::new(id.clone(), world, position, lines);
            data.is_ram = true;
            if let Some(billboard_mode) = billboard {
                data.billboard = billboard_mode;
            }
            if let Some(has_shadow) = shadow {
                data.shadow = has_shadow;
            }
            if let Some(is_see_through) = see_through {
                data.see_through = is_see_through;
            }
            if let Some(custom_scale) = scale {
                data.scale = custom_scale;
            }

            match manager.create_hologram(data, world_instance.as_ref()) {
                Ok(_) => IpcResponse::success(format!("hologram '{id}' created successfully")),
                Err(err) => IpcResponse::error(err),
            }
        }
        IpcRequest::Edit {
            id,
            lines,
            billboard,
            shadow,
            see_through,
            scale,
        } => {
            let mut manager = match crate::get_manager().write() {
                Ok(guard) => guard,
                Err(_) => return IpcResponse::error("Failed to acquire write lock on manager"),
            };

            let hologram = match manager.get_mut(&id) {
                Some(h) => h,
                None => return IpcResponse::error(format!("Hologram '{id}' not found")),
            };

            if let Some(new_lines) = lines {
                hologram.set_lines(new_lines);
            }
            if let Some(billboard_mode) = billboard {
                hologram.set_billboard(billboard_mode);
            }
            if let Some(has_shadow) = shadow {
                hologram.set_shadow(has_shadow);
            }
            if let Some(is_see_through) = see_through {
                hologram.set_see_through(is_see_through);
            }
            if let Some(custom_scale) = scale {
                hologram.set_scale(custom_scale);
            }

            let _ = manager.save();
            IpcResponse::success(format!("Hologram '{id}' updated successfully"))
        }
        IpcRequest::Move {
            id,
            position,
            world,
        } => {
            let mut manager = match crate::get_manager().write() {
                Ok(guard) => guard,
                Err(_) => return IpcResponse::error("Failed to acquire write lock on manager"),
            };

            let hologram = match manager.get_mut(&id) {
                Some(h) => h,
                None => return IpcResponse::error(format!("Hologram '{id}' not found")),
            };

            let target_world_name = world.unwrap_or_else(|| hologram.data.world_name.clone());
            let world_instance = match crate::get_world(&target_world_name) {
                Some(w) => w,
                None => {
                    return IpcResponse::error(format!("World '{target_world_name}' is not loaded"));
                }
            };

            hologram.teleport(position, &world_instance, target_world_name);
            let _ = manager.save();
            IpcResponse::success(format!("Hologram '{id}' moved successfully"))
        }
        IpcRequest::Delete { id } => {
            let mut manager = match crate::get_manager().write() {
                Ok(guard) => guard,
                Err(_) => return IpcResponse::error("Failed to acquire write lock on manager"),
            };

            match manager.delete_hologram(&id) {
                Ok(_) => IpcResponse::success(format!("Hologram '{id}' deleted successfully")),
                Err(err) => IpcResponse::error(err),
            }
        }
        IpcRequest::Get { id } => {
            let manager = match crate::get_manager().read() {
                Ok(guard) => guard,
                Err(_) => return IpcResponse::error("Failed to acquire read lock on manager"),
            };

            match manager.get(&id) {
                Some(hologram) => match IpcResponse::success_with_data(&hologram.data) {
                    Ok(res) => res,
                    Err(err) => IpcResponse::error(err),
                },
                None => IpcResponse::error(format!("Hologram '{id}' not found")),
            }
        }
        IpcRequest::List => {
            let manager = match crate::get_manager().read() {
                Ok(guard) => guard,
                Err(_) => return IpcResponse::error("Failed to acquire read lock on manager"),
            };

            let holograms: Vec<&HologramData> =
                manager.list().into_iter().map(|h| &h.data).collect();
            match IpcResponse::success_with_data(&holograms) {
                Ok(res) => res,
                Err(err) => IpcResponse::error(err),
            }
        }
    }
}
