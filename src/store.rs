//! Saves the team between launches, as one JSON file.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{model::*, theme::ThemeMode};

#[derive(Serialize, Deserialize, Default)]
pub struct Saved {
    pub signed_in: bool,
    pub profile: Profile,
    pub bots: Vec<Bot>,
    pub next_id: u64,
    pub theme: ThemeMode,
    pub installed_plugins: Vec<String>,
    #[serde(default)]
    pub settings: Settings,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub notifications: bool,
    pub sounds: bool,
    pub only_when_needed: bool,
    pub auto_review: bool,
    pub ask_before_sending: bool,
    pub ask_before_purchases: bool,
    pub ask_before_publishing: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            notifications: true,
            sounds: false,
            only_when_needed: false,
            auto_review: false,
            ask_before_sending: true,
            ask_before_purchases: true,
            ask_before_publishing: true,
        }
    }
}

pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SUZHOU_DATA_DIR") {
        return PathBuf::from(dir);
    }
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("suzhou")
}

fn state_file() -> PathBuf {
    data_dir().join("state.json")
}

pub fn load() -> Option<Saved> {
    let text = std::fs::read_to_string(state_file()).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save(saved: &Saved) {
    let path = state_file();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(saved) {
        Ok(text) => {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(tmp, path);
            }
        }
        Err(err) => eprintln!("suzhou: could not save: {err}"),
    }
}

pub fn clear() {
    let _ = std::fs::remove_file(state_file());
}
