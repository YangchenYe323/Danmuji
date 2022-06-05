use crate::plugins::GiftThankConfig;
use crate::{DanmujiResult, RoomConfig, UserConfig};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

lazy_static! {
    /// Project's Working Directory
    pub static ref PROJECT_ROOT: PathBuf = std::env::current_dir().unwrap();
    /// User Config File Path
    pub static ref USER_CONFIG: PathBuf = PROJECT_ROOT.join("user-config.json");
    /// Room Config File Path
    pub static ref ROOM_CONFIG: PathBuf = PROJECT_ROOT.join("room-config.json");
    /// Gift Thank Config File Path
    pub static ref THANK_CONFIG: PathBuf = PROJECT_ROOT.join("thank-config.json");
}

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
    save_json(config, USER_CONFIG.as_path())
}

/// Persist Last connected room
pub fn save_room_config(config: &RoomConfig) -> DanmujiResult<()> {
    save_json(config, ROOM_CONFIG.as_path())
}

pub fn save_thank_config(config: &GiftThankConfig) -> DanmujiResult<()> {
    save_json(config, THANK_CONFIG.as_path())
}

/// Load User Authentication from File
pub fn load_user_config() -> Option<UserConfig> {
    load_json(USER_CONFIG.as_path())
}

/// Load Room Configuration from File
pub fn load_room_config() -> Option<RoomConfig> {
    load_json(ROOM_CONFIG.as_path())
}

pub fn load_thank_config() -> GiftThankConfig {
    load_json(THANK_CONFIG.as_path()).unwrap_or_default()
}

pub fn delete_user_config() -> DanmujiResult<()> {
    std::fs::remove_file(USER_CONFIG.as_path())?;
    Ok(())
}

pub fn delete_room_config() -> DanmujiResult<()> {
    std::fs::remove_file(ROOM_CONFIG.as_path())?;
    Ok(())
}
