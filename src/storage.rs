//! Storage module for FSID mappings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Storage for FSID -> Path mappings
#[derive(Serialize, Deserialize, Default)]
pub struct FsidStorage {
    pub mappings: HashMap<String, String>,
}

impl FsidStorage {
    /// Load storage from a specific path
    pub fn load(path: &PathBuf) -> Self {
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save storage to a specific path
    pub fn save(&self, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).unwrap();
        let _ = fs::write(path, content);
    }

    /// Insert a mapping and save
    pub fn insert(&mut self, fsid: String, file_path: String, storage_path: &PathBuf) {
        self.mappings.insert(fsid, file_path);
        self.save(storage_path);
    }

    /// Get a path by FSID
    pub fn get(&self, fsid: &str) -> Option<&String> {
        self.mappings.get(fsid)
    }
}
