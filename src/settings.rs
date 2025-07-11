//! Persistent settings management for Alchemist UI
//!
//! This module handles saving and loading user preferences,
//! window positions, and application configuration.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use iced::{window, Theme};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemistSettings {
    pub version: String,
    pub theme: ThemeSettings,
    pub window_settings: HashMap<String, WindowSettings>,
    pub launcher_preferences: LauncherPreferences,
    pub nats_settings: NatsSettings,
    pub editor_settings: EditorSettings,
    pub recently_opened: Vec<RecentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub mode: ThemeMode,
    pub custom_colors: Option<CustomColors>,
    pub font_size_modifier: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColors {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub surface: String,
    pub error: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherPreferences {
    pub show_sidebar: bool,
    pub default_panel: String,
    pub auto_connect_nats: bool,
    pub remember_conversations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSettings {
    pub url: String,
    pub auth_method: AuthMethod,
    pub credentials: Option<Credentials>,
    pub reconnect_attempts: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    None,
    Token,
    UserPassword,
    NKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: Option<String>,
    pub token: Option<String>,
    // Password is stored encrypted
    pub encrypted_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub auto_save: bool,
    pub auto_save_interval_secs: u64,
    pub show_grid: bool,
    pub snap_to_grid: bool,
    pub grid_size: f32,
    pub default_zoom: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub item_type: RecentItemType,
    pub name: String,
    pub path: Option<String>,
    pub last_opened: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecentItemType {
    Workflow,
    Conversation,
    Document,
    Deployment,
}

impl Default for AlchemistSettings {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            theme: ThemeSettings::default(),
            window_settings: HashMap::new(),
            launcher_preferences: LauncherPreferences::default(),
            nats_settings: NatsSettings::default(),
            editor_settings: EditorSettings::default(),
            recently_opened: Vec::new(),
        }
    }
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Dark,
            custom_colors: None,
            font_size_modifier: 1.0,
        }
    }
}

impl Default for LauncherPreferences {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            default_panel: "Conversations".to_string(),
            auto_connect_nats: true,
            remember_conversations: true,
        }
    }
}

impl Default for NatsSettings {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            auth_method: AuthMethod::None,
            credentials: None,
            reconnect_attempts: 5,
            timeout_ms: 5000,
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_save_interval_secs: 30,
            show_grid: true,
            snap_to_grid: true,
            grid_size: 20.0,
            default_zoom: 1.0,
        }
    }
}

/// Settings manager for loading and saving preferences
pub struct SettingsManager {
    settings: AlchemistSettings,
    config_path: PathBuf,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("settings.json");
        
        let settings = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            // Create default settings and save
            let default_settings = AlchemistSettings::default();
            Self::save_to_file(&config_path, &default_settings)?;
            default_settings
        };
        
        Ok(Self {
            settings,
            config_path,
        })
    }
    
    /// Get the configuration directory
    fn get_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("alchemist");
        
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }
        
        Ok(config_dir)
    }
    
    /// Load settings from file
    fn load_from_file(path: &PathBuf) -> Result<AlchemistSettings> {
        let contents = std::fs::read_to_string(path)
            .context("Failed to read settings file")?;
        
        let settings: AlchemistSettings = serde_json::from_str(&contents)
            .context("Failed to parse settings file")?;
        
        Ok(settings)
    }
    
    /// Save settings to file
    fn save_to_file(path: &PathBuf, settings: &AlchemistSettings) -> Result<()> {
        let json = serde_json::to_string_pretty(settings)
            .context("Failed to serialize settings")?;
        
        std::fs::write(path, json)
            .context("Failed to write settings file")?;
        
        Ok(())
    }
    
    /// Get current settings
    pub fn settings(&self) -> &AlchemistSettings {
        &self.settings
    }
    
    /// Get mutable settings
    pub fn settings_mut(&mut self) -> &mut AlchemistSettings {
        &mut self.settings
    }
    
    /// Save current settings
    pub fn save(&self) -> Result<()> {
        Self::save_to_file(&self.config_path, &self.settings)
    }
    
    /// Update window position
    pub fn update_window_position(&mut self, window_id: &str, window_settings: window::Settings) {
        // Extract position coordinates based on the Position enum
        let (x, y) = match window_settings.position {
            window::Position::Default => (100.0, 100.0),
            window::Position::Centered => (500.0, 300.0),
            window::Position::Specific(point) => (point.x, point.y),
            _ => (100.0, 100.0), // Fallback for any other variants
        };
        
        let ws = WindowSettings {
            x,
            y,
            width: window_settings.size.width,
            height: window_settings.size.height,
            maximized: false, // TODO: Get actual maximized state
        };
        
        self.settings.window_settings.insert(window_id.to_string(), ws);
    }
    
    /// Get window settings
    pub fn get_window_settings(&self, window_id: &str) -> Option<window::Settings> {
        self.settings.window_settings.get(window_id).map(|ws| {
            window::Settings {
                size: iced::Size::new(ws.width, ws.height),
                position: window::Position::Specific(iced::Point::new(ws.x, ws.y)),
                resizable: true,
                ..Default::default()
            }
        })
    }
    
    /// Add recent item
    pub fn add_recent_item(&mut self, item_type: RecentItemType, name: String, path: Option<String>) {
        let item = RecentItem {
            item_type,
            name,
            path,
            last_opened: chrono::Utc::now(),
        };
        
        // Remove existing item with same name
        self.settings.recently_opened.retain(|i| i.name != item.name);
        
        // Add to front
        self.settings.recently_opened.insert(0, item);
        
        // Keep only last 20 items
        self.settings.recently_opened.truncate(20);
    }
    
    /// Get theme
    pub fn get_theme(&self) -> Theme {
        match self.settings.theme.mode {
            ThemeMode::Light => Theme::Light,
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::System => {
                // TODO: Detect system theme
                Theme::Dark
            }
            ThemeMode::Custom => {
                // TODO: Implement custom theme
                Theme::Dark
            }
        }
    }
    
    /// Export settings
    pub fn export_settings(&self, path: &PathBuf) -> Result<()> {
        Self::save_to_file(path, &self.settings)
    }
    
    /// Import settings
    pub fn import_settings(&mut self, path: &PathBuf) -> Result<()> {
        self.settings = Self::load_from_file(path)?;
        self.save()?;
        Ok(())
    }
    
    /// Reset to defaults
    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.settings = AlchemistSettings::default();
        self.save()
    }
}

/// Global settings instance
static SETTINGS: std::sync::OnceLock<std::sync::Arc<tokio::sync::RwLock<SettingsManager>>> = std::sync::OnceLock::new();

/// Initialize global settings
pub async fn initialize_settings() -> Result<()> {
    let manager = SettingsManager::new()?;
    let settings = std::sync::Arc::new(tokio::sync::RwLock::new(manager));
    SETTINGS.set(settings).map_err(|_| anyhow::anyhow!("Settings already initialized"))?;
    Ok(())
}

/// Get global settings
pub fn settings() -> &'static std::sync::Arc<tokio::sync::RwLock<SettingsManager>> {
    SETTINGS.get().expect("Settings not initialized")
}

/// Helper to save window position on close
pub async fn save_window_position(window_id: &str, window_settings: window::Settings) -> Result<()> {
    let mut manager = settings().write().await;
    manager.update_window_position(window_id, window_settings);
    manager.save()
}

/// Helper to get window settings
pub async fn get_window_settings(window_id: &str) -> Option<window::Settings> {
    let manager = settings().read().await;
    manager.get_window_settings(window_id)
}

/// Helper to add recent item
pub async fn add_recent_item(item_type: RecentItemType, name: String, path: Option<String>) -> Result<()> {
    let mut manager = settings().write().await;
    manager.add_recent_item(item_type, name, path);
    manager.save()
}