use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmarks {
    bookmarks: Vec<Bookmark>,
    quick_slots: HashMap<char, PathBuf>, // 0-9 keys
}

impl Default for Bookmarks {
    fn default() -> Self {
        Self::new()
    }
}

impl Bookmarks {
    pub fn new() -> Self {
        Bookmarks {
            bookmarks: Vec::new(),
            quick_slots: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, path: PathBuf) {
        let created_at = chrono::Utc::now().to_rfc3339();
        let bookmark = Bookmark {
            name,
            path,
            created_at,
        };

        // Check for duplicate names and replace if exists
        if let Some(pos) = self.bookmarks.iter().position(|b| b.name == bookmark.name) {
            self.bookmarks[pos] = bookmark;
        } else {
            self.bookmarks.push(bookmark);
        }
    }

    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(pos) = self.bookmarks.iter().position(|b| b.name == name) {
            self.bookmarks.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get(&self, name: &str) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.name == name)
    }

    pub fn get_all(&self) -> &[Bookmark] {
        &self.bookmarks
    }

    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bookmarks.len()
    }

    pub fn set_quick_slot(&mut self, key: char, path: PathBuf) {
        if key.is_ascii_digit() {
            self.quick_slots.insert(key, path);
        }
    }

    pub fn get_quick_slot(&self, key: char) -> Option<&PathBuf> {
        self.quick_slots.get(&key)
    }

    pub fn load_from_config() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read bookmarks config file")?;
            let bookmarks: Bookmarks = serde_json::from_str(&content)
                .context("Failed to parse bookmarks config file")?;
            Ok(bookmarks)
        } else {
            Ok(Bookmarks::new())
        }
    }

    pub fn save_to_config(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize bookmarks")?;
        fs::write(&config_path, content)
            .context("Failed to write bookmarks config file")?;

        Ok(())
    }

    #[cfg(windows)]
    fn config_path() -> Result<PathBuf> {
        let appdata = std::env::var("APPDATA")
            .context("Failed to get APPDATA environment variable")?;
        Ok(PathBuf::from(appdata).join("filebrowser-tui").join("bookmarks.json"))
    }

    #[cfg(not(windows))]
    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("Failed to get HOME environment variable")?;
        Ok(PathBuf::from(home).join(".config").join("filebrowser-tui").join("bookmarks.json"))
    }
}
