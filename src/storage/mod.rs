use crate::hologram::HologramData;
use std::{
    fs,
    path::{Path, PathBuf},
};

const FILE_NAME: &str = "holograms.json";

/// Returns path hologram.json.
pub fn get_storage(data_folder: &Path) -> PathBuf {
    data_folder.join(FILE_NAME)
}

/// Saves the list of hologram configurations to a JSON file on disk.
pub fn save_to_disk(data_folder: &Path, data: &[HologramData]) -> Result<(), String> {
    if !data_folder.exists() {
        fs::create_dir_all(data_folder)
            .map_err(|e| format!("Failed to create plugin data directory: {e}"))?;
    }

    let file_path = get_storage(data_folder);
    let json_string = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize hologram JSON data: {e}"))?;

    fs::write(&file_path, json_string)
        .map_err(|e| format!("Failed to write holograms.json: {e}"))?;

    crate::logger::info(&format!(
        "Successfully saved {} hologram configurations to '{:?}'",
        data.len(),
        file_path
    ));
    Ok(())
}

/// Loads the list of hologram configurations from the JSON file on disk.
pub fn load_from_disk(data_folder: &Path) -> Result<Vec<HologramData>, String> {
    let file_path = get_storage(data_folder);

    if !file_path.exists() {
        crate::logger::info(&format!(
            "Storage file '{:?}' not found. Initializing with empty state.",
            file_path
        ));
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read holograms.json: {e}"))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let data: Vec<HologramData> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON in holograms.json: {e}"))?;

    crate::logger::info(&format!(
        "Successfully loaded {} hologram configurations from '{:?}'",
        data.len(),
        file_path
    ));
    Ok(data)
}
