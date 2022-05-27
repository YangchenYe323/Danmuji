use std::path::Path;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use serde::{Serialize, de::DeserializeOwned};
use crate::{
	DanmujiResult,
	UserConfig,
	RoomConfig,
};

fn save_json(object: &impl Serialize, path: impl AsRef<Path>) -> DanmujiResult<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, object)?;
    Ok(())
}

fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Option<T> {
    let file = OpenOptions::new().read(true).open(path).ok()?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).ok()
}

/// Persist User Authentication Config
pub fn save_user_config(config: &UserConfig) -> DanmujiResult<()> {
    save_json(config, "user.json")
}

/// Persist Last connected room
pub fn save_room_config(config: &RoomConfig) -> DanmujiResult<()> {
    save_json(config, "room.json")
}

/// Load User Authentication from File
pub fn load_user_config() -> Option<UserConfig> {
    load_json("user.json")
}

/// Load Room Configuration from File
pub fn load_room_config() -> Option<RoomConfig> {
    load_json("room.json")
}
