use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionManager {
    selected_files: HashSet<PathBuf>,
    last_selected_index: Option<usize>,
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionManager {
    pub fn new() -> Self {
        SelectionManager {
            selected_files: HashSet::new(),
            last_selected_index: None,
        }
    }

    pub fn is_selected(&self, path: &PathBuf) -> bool {
        self.selected_files.contains(path)
    }

    pub fn toggle(&mut self, path: PathBuf) {
        if self.selected_files.contains(&path) {
            self.selected_files.remove(&path);
        } else {
            self.selected_files.insert(path);
        }
    }

    pub fn select(&mut self, path: PathBuf) {
        self.selected_files.insert(path);
    }

    pub fn deselect(&mut self, path: &PathBuf) {
        self.selected_files.remove(path);
    }

    pub fn select_all(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            self.selected_files.insert(path);
        }
    }

    pub fn deselect_all(&mut self) {
        self.selected_files.clear();
    }

    pub fn invert(&mut self, all_paths: Vec<PathBuf>) {
        let mut new_selection = HashSet::new();
        for path in all_paths {
            if !self.selected_files.contains(&path) {
                new_selection.insert(path);
            }
        }
        self.selected_files = new_selection;
    }

    pub fn get_selected(&self) -> Vec<PathBuf> {
        self.selected_files.iter().cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.selected_files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selected_files.is_empty()
    }

    pub fn set_last_selected(&mut self, index: usize) {
        self.last_selected_index = Some(index);
    }

    pub fn get_last_selected(&self) -> Option<usize> {
        self.last_selected_index
    }

    pub fn select_range(
        &mut self,
        from_index: usize,
        to_index: usize,
        files: &[super::FileEntry],
    ) {
        let start = from_index.min(to_index);
        let end = from_index.max(to_index);

        for i in start..=end {
            if let Some(file) = files.get(i) {
                self.selected_files.insert(file.path.clone());
            }
        }
        self.last_selected_index = Some(to_index);
    }
}

#[cfg(test)]
mod tests;
