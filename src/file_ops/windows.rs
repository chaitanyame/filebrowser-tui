use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub letter: char,
    pub drive_type: DriveType,
    pub volume_label: Option<String>,
    pub total_space: Option<u64>,
    pub free_space: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriveType {
    Unknown,
    NoRoot,
    Removable,
    Fixed,
    Network,
    CDROM,
    RAMDisk,
}

pub struct WindowsDrives {
    pub drives: Vec<DriveInfo>,
}

impl WindowsDrives {
    pub fn new() -> Self {
        let mut drives = Vec::new();

        // Get available drives
        for letter in 'A'..='Z' {
            let drive_path = format!("{}:\\", letter);
            let path = PathBuf::from(&drive_path);

            if path.exists() {
                let (drive_type, volume_label, total_space, free_space) = Self::probe_drive(&drive_path);

                drives.push(DriveInfo {
                    letter,
                    drive_type,
                    volume_label,
                    total_space,
                    free_space,
                });
            }
        }

        WindowsDrives { drives }
    }

    fn probe_drive(drive_path: &str) -> (DriveType, Option<String>, Option<u64>, Option<u64>) {
        let path = PathBuf::from(drive_path);

        // Try to get disk space
        let (total_space, free_space) = if let Ok(metadata) = path.metadata() {
            // On Unix, we can't easily get this without sysinfo crate
            // On Windows, this would be filled by Windows API
            (None, None)
        } else {
            (None, None)
        };

        // Basic drive type detection based on existence
        let drive_type = DriveType::Fixed;

        #[cfg(windows)]
        {
            // On Windows, we could use Windows API for better detection
            // For simplicity, we'll mark all as Fixed
        }

        (drive_type, None, total_space, free_space)
    }

    pub fn refresh(&mut self) {
        *self = Self::new();
    }

    pub fn get_by_letter(&self, letter: char) -> Option<&DriveInfo> {
        self.drives.iter()
            .find(|d| d.letter.eq_ignore_ascii_case(&letter))
    }

    pub fn get_path_for_drive(&self, letter: char) -> Option<PathBuf> {
        for drive in &self.drives {
            if drive.letter.eq_ignore_ascii_case(&letter) {
                return Some(PathBuf::from(format!("{}:\\", letter.to_ascii_uppercase())));
            }
        }
        None
    }
}

impl Default for WindowsDrives {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a path is a UNC path (\\server\share)
pub fn is_unc_path(path: &PathBuf) -> bool {
    if let Some(s) = path.to_str() {
        return s.starts_with("\\\\");
    }
    false
}

/// Convert a path to use UNC prefix for long paths on Windows
#[cfg(windows)]
pub fn add_unc_prefix(path: &PathBuf) -> PathBuf {
    use std::path::Path;
    if let Some(s) = path.to_str() {
        if s.starts_with("\\\\?\\") {
            return path.clone();
        }
        return PathBuf::from(format!("\\\\?\\{}", s));
    }
    path.clone()
}

#[cfg(not(windows))]
pub fn add_unc_prefix(path: &PathBuf) -> PathBuf {
    path.clone()
}
