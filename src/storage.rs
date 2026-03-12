use crate::api::Track;
use crate::config::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

fn favorites_path() -> PathBuf {
    config_dir().join("favorites.json")
}

fn playlists_path() -> PathBuf {
    config_dir().join("playlists.json")
}

fn queue_path() -> PathBuf {
    config_dir().join("queue.json")
}

pub fn load_favorites() -> HashSet<String> {
    let path = favorites_path();
    if !path.exists() {
        return HashSet::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn save_favorites(favorites: &HashSet<String>) {
    if let Some(parent) = favorites_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(
        favorites_path(),
        serde_json::to_string_pretty(favorites).unwrap_or_default(),
    );
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

pub fn load_playlists() -> Vec<Playlist> {
    let path = playlists_path();
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn save_playlists(playlists: &[Playlist]) {
    if let Some(parent) = playlists_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(
        playlists_path(),
        serde_json::to_string_pretty(playlists).unwrap_or_default(),
    );
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedQueue {
    pub tracks: Vec<Track>,
    pub now_playing: Option<Track>,
}

pub fn load_queue() -> SavedQueue {
    let path = queue_path();
    if !path.exists() {
        return SavedQueue::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

pub fn save_queue(queue: &[Track], now_playing: &Option<Track>) {
    if let Some(parent) = queue_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let saved = SavedQueue {
        tracks: queue.to_vec(),
        now_playing: now_playing.clone(),
    };
    let _ = std::fs::write(
        queue_path(),
        serde_json::to_string_pretty(&saved).unwrap_or_default(),
    );
}
