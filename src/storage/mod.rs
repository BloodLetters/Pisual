use crate::hologram::HologramData;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const HOLOGRAMS_DIR: &str = "holograms";
pub const DATA_FILE_NAME: &str = "data.json";
pub const LEGACY_FILE_NAME: &str = "holograms.json";

pub fn get_holograms_dir(data_folder: &Path) -> PathBuf {
    data_folder.join(HOLOGRAMS_DIR)
}

pub fn get_hologram_dir(data_folder: &Path, id: &str) -> PathBuf {
    get_holograms_dir(data_folder).join(id)
}

pub fn get_hologram_file(data_folder: &Path, id: &str) -> PathBuf {
    get_hologram_dir(data_folder, id).join(DATA_FILE_NAME)
}

pub fn get_storage(data_folder: &Path) -> PathBuf {
    get_holograms_dir(data_folder)
}

/// Validates that a hologram ID is safe for filesystem directory names.
/// Only allows alphanumeric, underscores, and hyphens (1-64 characters).
pub fn is_valid_id(id: &str) -> bool {
    let trimmed = id.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return false;
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return false;
    }

    if trimmed == "." || trimmed == ".." {
        return false;
    }

    true
}

pub fn save_hologram(data_folder: &Path, data: &HologramData) -> Result<(), String> {
    if !is_valid_id(&data.id) {
        return Err(format!(
            "Invalid hologram ID '{}'! ID must only contain alphanumeric, underscore, or hyphen (max 64 chars).",
            data.id
        ));
    }

    let dir = get_hologram_dir(data_folder, &data.id);
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create directory for hologram '{}': {e}", data.id))?;
    }

    let file_path = dir.join(DATA_FILE_NAME);
    let json_string = serde_json::to_string_pretty(data).map_err(|e| {
        format!(
            "Failed to serialize hologram JSON data for '{}': {e}",
            data.id
        )
    })?;

    fs::write(&file_path, json_string)
        .map_err(|e| format!("Failed to write '{:?}': {e}", file_path))?;
    Ok(())
}

pub fn delete_hologram_file(data_folder: &Path, id: &str) -> Result<(), String> {
    let dir = get_hologram_dir(data_folder, id);
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to delete hologram directory '{:?}': {e}", dir))?;
    }
    Ok(())
}

pub fn save_to_disk(data_folder: &Path, data: &[HologramData]) -> Result<(), String> {
    let holo_dir = get_holograms_dir(data_folder);
    if !holo_dir.exists() {
        fs::create_dir_all(&holo_dir)
            .map_err(|e| format!("Failed to create plugin holograms directory: {e}"))?;
    }

    for item in data {
        save_hologram(data_folder, item)?;
    }
    Ok(())
}

fn migrate_legacy_storage(data_folder: &Path) {
    let legacy_file = data_folder.join(LEGACY_FILE_NAME);
    if !legacy_file.exists() {
        return;
    }

    match fs::read_to_string(&legacy_file) {
        Ok(content) => {
            if content.trim().is_empty() {
                let _ = fs::remove_file(&legacy_file);
                return;
            }

            match serde_json::from_str::<Vec<HologramData>>(&content) {
                Ok(items) => {
                    let total = items.len();
                    let mut migrated = 0;
                    for item in &items {
                        if save_hologram(data_folder, item).is_ok() {
                            migrated += 1;
                        }
                    }

                    let backup_file = data_folder.join(format!("{LEGACY_FILE_NAME}.migrated"));
                    if let Err(e) = fs::rename(&legacy_file, &backup_file) {
                        crate::logger::warn(&format!(
                            "Migrated {migrated}/{total} holograms, but failed to rename legacy file: {e}"
                        ));
                    } else {
                        crate::logger::info(&format!(
                            "Successfully migrated {migrated}/{total} holograms from '{:?}' to modular storage.",
                            legacy_file
                        ));
                    }
                }
                Err(e) => {
                    crate::logger::warn(&format!(
                        "Legacy '{:?}' exists but could not be parsed as HologramData array: {e}",
                        legacy_file
                    ));
                }
            }
        }
        Err(e) => {
            crate::logger::warn(&format!(
                "Failed to read legacy storage '{:?}': {e}",
                legacy_file
            ));
        }
    }
}

/// Loads all holograms from `<data_folder>/holograms/<id>/data.json`.
/// Fault-tolerant: If one hologram file is corrupted, it logs a warning and continues loading others.
pub fn load_from_disk(data_folder: &Path) -> Result<Vec<HologramData>, String> {
    migrate_legacy_storage(data_folder);

    let holo_dir = get_holograms_dir(data_folder);
    if !holo_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&holo_dir)
        .map_err(|e| format!("Failed to read holograms directory '{:?}': {e}", holo_dir))?;

    let mut result = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let data_file = path.join(DATA_FILE_NAME);
            if data_file.exists() {
                match fs::read_to_string(&data_file) {
                    Ok(content) => {
                        if content.trim().is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<HologramData>(&content) {
                            Ok(data) => result.push(data),
                            Err(e) => {
                                crate::logger::warn(&format!(
                                    "Failed to parse JSON in '{:?}': {e}. Skipping.",
                                    data_file
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        crate::logger::warn(&format!(
                            "Failed to read file '{:?}': {e}. Skipping.",
                            data_file
                        ));
                    }
                }
            }
        }
    }

    crate::logger::info(&format!(
        "Successfully loaded {} hologram configuration(s) from '{:?}'",
        result.len(),
        holo_dir
    ));
    Ok(result)
}
