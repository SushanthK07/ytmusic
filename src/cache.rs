use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub video_id: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub last_accessed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CacheIndex {
    entries: Vec<CacheEntry>,
}

pub struct AudioCache {
    cache_dir: PathBuf,
    max_size_mb: u64,
    index: CacheIndex,
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl AudioCache {
    pub fn new(max_size_mb: u64) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ytmusic");
        let _ = std::fs::create_dir_all(&cache_dir);
        let index = Self::load_index(&cache_dir);
        Self {
            cache_dir,
            max_size_mb,
            index,
        }
    }

    pub fn lookup(&mut self, video_id: &str) -> Option<PathBuf> {
        let entry = self
            .index
            .entries
            .iter_mut()
            .find(|e| e.video_id == video_id)?;
        let path = self.cache_dir.join(&entry.file_name);
        if path.exists() {
            entry.last_accessed = unix_now();
            self.save_index();
            Some(path)
        } else {
            self.index.entries.retain(|e| e.video_id != video_id);
            self.save_index();
            None
        }
    }

    pub fn cache_path_for(&self, video_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.opus", video_id))
    }

    pub fn lookup_exists(&self, video_id: &str) -> bool {
        self.index
            .entries
            .iter()
            .any(|e| e.video_id == video_id && self.cache_dir.join(&e.file_name).exists())
    }

    pub fn register(&mut self, video_id: &str, size_bytes: u64) {
        self.index.entries.retain(|e| e.video_id != video_id);
        self.index.entries.push(CacheEntry {
            video_id: video_id.to_string(),
            file_name: format!("{}.opus", video_id),
            size_bytes,
            last_accessed: unix_now(),
        });
        self.evict_if_needed();
        self.save_index();
    }

    fn total_size_bytes(&self) -> u64 {
        self.index.entries.iter().map(|e| e.size_bytes).sum()
    }

    fn evict_if_needed(&mut self) {
        let max_bytes = self.max_size_mb * 1024 * 1024;
        while self.total_size_bytes() > max_bytes && !self.index.entries.is_empty() {
            let oldest_idx = self
                .index
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.last_accessed)
                .map(|(i, _)| i)
                .unwrap();
            let removed = self.index.entries.remove(oldest_idx);
            let path = self.cache_dir.join(&removed.file_name);
            let _ = std::fs::remove_file(path);
        }
    }

    fn index_path(cache_dir: &Path) -> PathBuf {
        cache_dir.join("index.json")
    }

    fn load_index(cache_dir: &Path) -> CacheIndex {
        let path = Self::index_path(cache_dir);
        if !path.exists() {
            return CacheIndex::default();
        }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    fn save_index(&self) {
        let path = Self::index_path(&self.cache_dir);
        let _ = std::fs::write(
            path,
            serde_json::to_string_pretty(&self.index).unwrap_or_default(),
        );
    }
}
