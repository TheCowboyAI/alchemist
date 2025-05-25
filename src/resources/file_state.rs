use bevy::prelude::*;
use std::path::PathBuf;

/// Resource for file operation state
#[derive(Resource, Default)]
pub struct FileOperationState {
    pub current_file_path: Option<String>,
    pub has_unsaved_changes: bool,
    pub last_save_time: Option<f64>,
    pub available_files: Vec<PathBuf>,
}

impl FileOperationState {
    /// Scan the models directory for available files
    pub fn scan_models_directory(&mut self) {
        self.available_files.clear();

        if let Ok(entries) = std::fs::read_dir("models") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    self.available_files.push(path);
                }
            }
        }

        self.available_files.sort();
    }
}

/// Resource for auto-save configuration
#[derive(Resource)]
pub struct AutoSaveConfig {
    pub enabled: bool,
    pub interval_seconds: f32,
    pub last_auto_save: f64,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 300.0, // 5 minutes
            last_auto_save: 0.0,
        }
    }
}
