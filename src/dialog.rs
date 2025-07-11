//! Dialog management for AI interactions

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::{
    config::AlchemistConfig,
    ai::AiManager,
    shell_commands::DialogCommands,
};

// Re-export for dialog window
pub use Message as DialogMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialog {
    pub id: String,
    pub title: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
    pub metadata: DialogMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogMetadata {
    pub domain: Option<String>,
    pub context: Option<String>,
    pub tags: Vec<String>,
    pub total_tokens: u32,
}

#[derive(Clone)]
pub struct DialogManager {
    dialogs: Arc<DashMap<String, Dialog>>,
    storage_path: PathBuf,
    current_dialog: Arc<RwLock<Option<String>>>,
}

impl DialogManager {
    pub async fn new(config: &AlchemistConfig) -> Result<Self> {
        let storage_path = PathBuf::from(
            shellexpand::tilde(&config.general.dialog_history_path).to_string()
        );
        
        // Ensure storage directory exists
        fs::create_dir_all(&storage_path).await?;
        
        let manager = Self {
            dialogs: Arc::new(DashMap::new()),
            storage_path,
            current_dialog: Arc::new(RwLock::new(None)),
        };
        
        // Load recent dialogs
        manager.load_recent_dialogs(10).await?;
        
        Ok(manager)
    }
    
    pub async fn handle_command(&mut self, command: DialogCommands) -> Result<()> {
        match command {
            DialogCommands::New { title, model } => {
                self.new_dialog_cli(title, model).await?;
            }
            DialogCommands::List { count } => {
                self.list_dialogs_cli(count).await?;
            }
            DialogCommands::Continue { id } => {
                self.continue_dialog_cli(id).await?;
            }
            DialogCommands::Export { id, format } => {
                self.export_dialog_cli(id, format).await?;
            }
        }
        Ok(())
    }
    
    pub async fn count_dialogs(&self) -> Result<usize> {
        Ok(self.dialogs.len())
    }
    
    pub async fn list_recent(&self, count: usize) -> Result<Vec<DialogSummary>> {
        let mut summaries: Vec<DialogSummary> = self.dialogs
            .iter()
            .map(|entry| {
                let dialog = entry.value();
                DialogSummary {
                    id: dialog.id.clone(),
                    title: dialog.title.clone(),
                    model: dialog.model.clone(),
                    created_at: dialog.created_at,
                    updated_at: dialog.updated_at,
                    message_count: dialog.messages.len(),
                    total_tokens: dialog.metadata.total_tokens,
                }
            })
            .collect();
        
        // Sort by updated_at descending
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries.truncate(count);
        
        Ok(summaries)
    }
    
    async fn new_dialog_cli(&mut self, title: Option<String>, model: Option<String>) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let title = title.unwrap_or_else(|| format!("Dialog {}", chrono::Local::now().format("%Y-%m-%d %H:%M")));
        let model = model.unwrap_or_else(|| "claude-3-sonnet".to_string());
        
        let dialog = Dialog {
            id: id.clone(),
            title: title.clone(),
            model: model.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
            metadata: DialogMetadata {
                domain: None,
                context: None,
                tags: Vec::new(),
                total_tokens: 0,
            },
        };
        
        self.dialogs.insert(id.clone(), dialog);
        *self.current_dialog.write().await = Some(id.clone());
        
        // Save to disk
        self.save_dialog(&id).await?;
        
        println!("✅ Created new dialog: {}", title);
        println!("   ID: {}", id);
        println!("🚀 Launching dialog window...");
        
        // Note: To launch the dialog window, use shell.launch_dialog_window()
        println!("   To open in UI: Use shell.launch_dialog_window('{}')", id);
        
        // Give the window time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        Ok(())
    }
    
    async fn list_dialogs_cli(&self, count: usize) -> Result<()> {
        let summaries = self.list_recent(count).await?;
        
        if summaries.is_empty() {
            println!("No dialogs found. Use 'ia dialog new' to create one.");
            return Ok(());
        }
        
        println!("💬 Recent Dialogs:");
        println!("─────────────────────────────");
        
        for summary in summaries {
            let current_dialog = self.current_dialog.read().await;
            let current_marker = if Some(&summary.id) == current_dialog.as_ref() {
                " 👉"
            } else {
                ""
            };
            
            println!(
                "{} {} - {} messages{}",
                summary.title,
                summary.message_count,
                summary.model,
                current_marker
            );
            println!(
                "   ID: {} | Updated: {} | Tokens: {}",
                &summary.id[..8],
                summary.updated_at.format("%Y-%m-%d %H:%M"),
                summary.total_tokens
            );
        }
        
        Ok(())
    }
    
    async fn continue_dialog_cli(&mut self, id: String) -> Result<()> {
        if !self.dialogs.contains_key(&id) {
            // Try to load from disk
            self.load_dialog(&id).await?;
        }
        
        let dialog = self.dialogs.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Dialog not found: {}", id))?;
        
        *self.current_dialog.write().await = Some(id.clone());
        
        println!("📝 Continuing dialog: {}", dialog.title);
        println!("   Model: {}", dialog.model);
        println!("   Messages: {}", dialog.messages.len());
        
        // Show last few messages
        let recent_count = dialog.messages.len().min(3);
        if recent_count > 0 {
            println!("\nRecent messages:");
            for msg in dialog.messages.iter().rev().take(recent_count).rev() {
                let role_icon = match msg.role {
                    MessageRole::System => "🔧",
                    MessageRole::User => "👤",
                    MessageRole::Assistant => "🤖",
                };
                
                let content_preview = if msg.content.len() > 80 {
                    format!("{}...", &msg.content[..77])
                } else {
                    msg.content.clone()
                };
                
                println!("{} {}: {}", role_icon, 
                    match msg.role {
                        MessageRole::System => "System",
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant",
                    },
                    content_preview
                );
            }
        }
        
        println!("\nDialog loaded. Ready for interaction.");
        
        Ok(())
    }
    
    async fn export_dialog_cli(&self, id: String, format: String) -> Result<()> {
        let dialog = self.dialogs.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Dialog not found: {}", id))?;
        
        let export_content = match format.as_str() {
            "json" => serde_json::to_string_pretty(&*dialog)?,
            "markdown" => self.export_as_markdown(&dialog),
            _ => return Err(anyhow::anyhow!("Unsupported format: {}. Use 'json' or 'markdown'", format)),
        };
        
        let filename = format!("dialog_{}_{}.{}", 
            &id[..8], 
            dialog.created_at.format("%Y%m%d_%H%M%S"),
            if format == "json" { "json" } else { "md" }
        );
        
        fs::write(&filename, export_content).await?;
        
        println!("✅ Exported dialog to: {}", filename);
        
        Ok(())
    }
    
    fn export_as_markdown(&self, dialog: &Dialog) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# {}\n\n", dialog.title));
        output.push_str(&format!("- **ID**: {}\n", dialog.id));
        output.push_str(&format!("- **Model**: {}\n", dialog.model));
        output.push_str(&format!("- **Created**: {}\n", dialog.created_at.format("%Y-%m-%d %H:%M:%S")));
        output.push_str(&format!("- **Updated**: {}\n", dialog.updated_at.format("%Y-%m-%d %H:%M:%S")));
        output.push_str(&format!("- **Total Tokens**: {}\n", dialog.metadata.total_tokens));
        
        if !dialog.metadata.tags.is_empty() {
            output.push_str(&format!("- **Tags**: {}\n", dialog.metadata.tags.join(", ")));
        }
        
        output.push_str("\n## Messages\n\n");
        
        for msg in &dialog.messages {
            let role = match msg.role {
                MessageRole::System => "System",
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
            };
            
            output.push_str(&format!("### {} - {}\n\n", role, msg.timestamp.format("%H:%M:%S")));
            output.push_str(&format!("{}\n\n", msg.content));
            
            if let Some(tokens) = msg.tokens {
                output.push_str(&format!("*Tokens: {}*\n\n", tokens));
            }
        }
        
        output
    }
    
    async fn save_dialog(&self, id: &str) -> Result<()> {
        if let Some(dialog) = self.dialogs.get(id) {
            let file_path = self.storage_path.join(format!("{}.json", id));
            let content = serde_json::to_string_pretty(&*dialog)?;
            fs::write(file_path, content).await?;
        }
        Ok(())
    }
    
    async fn load_dialog(&self, id: &str) -> Result<()> {
        let file_path = self.storage_path.join(format!("{}.json", id));
        
        if !file_path.exists() {
            return Err(anyhow::anyhow!("Dialog file not found"));
        }
        
        let content = fs::read_to_string(file_path).await?;
        let dialog: Dialog = serde_json::from_str(&content)?;
        
        self.dialogs.insert(id.to_string(), dialog);
        
        Ok(())
    }
    
    async fn load_recent_dialogs(&self, count: usize) -> Result<()> {
        let mut entries = fs::read_dir(&self.storage_path).await?;
        let mut dialog_files = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".json") {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            dialog_files.push((entry.path(), modified));
                        }
                    }
                }
            }
        }
        
        // Sort by modification time, newest first
        dialog_files.sort_by(|a, b| b.1.cmp(&a.1));
        dialog_files.truncate(count);
        
        for (path, _) in dialog_files {
            if let Some(stem) = path.file_stem() {
                if let Some(id) = stem.to_str() {
                    if let Err(e) = self.load_dialog(id).await {
                        warn!("Failed to load dialog {}: {}", id, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn add_message(&mut self, dialog_id: &str, role: MessageRole, content: String, metadata: Option<serde_json::Value>) -> Result<DialogMessage> {
        let mut dialog = self.dialogs.get_mut(dialog_id)
            .ok_or_else(|| anyhow::anyhow!("Dialog not found"))?;
        
        let mut message = Message {
            role: role.clone(),
            content: content.clone(),
            timestamp: Utc::now(),
            tokens: None,
        };
        
        // Extract tokens from metadata if provided
        if let Some(meta) = &metadata {
            if let Some(tokens) = meta.get("tokens").and_then(|v| v.as_u64()) {
                message.tokens = Some(tokens as u32);
            }
        }
        
        dialog.messages.push(message.clone());
        dialog.updated_at = Utc::now();
        
        drop(dialog); // Release the lock
        
        // Save to disk
        self.save_dialog(dialog_id).await?;
        
        Ok(DialogMessage {
            role,
            content,
            timestamp: message.timestamp,
            tokens: message.tokens,
        })
    }
    
    pub async fn get_dialog(&self, dialog_id: &str) -> Result<Dialog> {
        self.dialogs.get(dialog_id)
            .map(|d| d.clone())
            .ok_or_else(|| anyhow::anyhow!("Dialog not found"))
    }
    
    pub async fn get_messages(&self, dialog_id: &str, limit: usize) -> Result<Vec<DialogMessage>> {
        let dialog = self.dialogs.get(dialog_id)
            .ok_or_else(|| anyhow::anyhow!("Dialog not found"))?;
        
        let messages: Vec<DialogMessage> = dialog.messages
            .iter()
            .rev()
            .take(limit)
            .rev()
            .map(|m| DialogMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                timestamp: m.timestamp,
                tokens: m.tokens,
            })
            .collect();
        
        Ok(messages)
    }
    
    pub async fn get_all_messages(&self, dialog_id: &str) -> Result<Vec<DialogMessage>> {
        let dialog = self.dialogs.get(dialog_id)
            .ok_or_else(|| anyhow::anyhow!("Dialog not found"))?;
        
        let messages: Vec<DialogMessage> = dialog.messages
            .iter()
            .map(|m| DialogMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                timestamp: m.timestamp,
                tokens: m.tokens,
            })
            .collect();
        
        Ok(messages)
    }
}

#[derive(Debug, Clone)]
pub struct DialogSummary {
    pub id: String,
    pub title: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::collections::HashMap;
    
    // Mock configuration for testing
    fn create_test_config(temp_dir: &TempDir) -> AlchemistConfig {
        AlchemistConfig {
            general: crate::config::GeneralConfig {
                dialog_history_path: temp_dir.path().to_str().unwrap().to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    #[tokio::test]
    async fn test_dialog_manager_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        
        let manager = DialogManager::new(&config).await.unwrap();
        
        assert_eq!(manager.dialogs.len(), 0);
        assert!(manager.current_dialog.is_none());
        assert!(temp_dir.path().exists());
    }
    
    #[tokio::test]
    async fn test_create_new_dialog_with_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        manager.new_dialog_cli(None, None).await.unwrap();
        
        assert_eq!(manager.dialogs.len(), 1);
        assert!(manager.current_dialog.is_some());
        
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        let dialog = manager.dialogs.get(dialog_id).unwrap();
        
        assert!(dialog.title.starts_with("Dialog "));
        assert_eq!(dialog.model, "default");
        assert_eq!(dialog.messages.len(), 0);
        assert_eq!(dialog.metadata.total_tokens, 0);
        
        // Check that dialog was saved to disk
        let file_path = temp_dir.path().join(format!("{}.json", dialog_id));
        assert!(file_path.exists());
    }
    
    #[tokio::test]
    async fn test_create_new_dialog_with_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        let title = "Test Dialog".to_string();
        let model = "gpt-4".to_string();
        
        manager.new_dialog_cli(Some(title.clone()), Some(model.clone())).await.unwrap();
        
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        let dialog = manager.dialogs.get(dialog_id).unwrap();
        
        assert_eq!(dialog.title, title);
        assert_eq!(dialog.model, model);
    }
    
    #[tokio::test]
    async fn test_add_messages_to_dialog() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        // Create a dialog
        manager.new_dialog_cli(None, None).await.unwrap();
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        
        // Add different types of messages
        manager.add_message(&dialog_id, MessageRole::System, "System prompt".to_string(), None).await.unwrap();
        manager.add_message(&dialog_id, MessageRole::User, "User question".to_string(), None).await.unwrap();
        manager.add_message(&dialog_id, MessageRole::Assistant, "Assistant response".to_string(), None).await.unwrap();
        
        let dialog = manager.dialogs.get(&dialog_id).unwrap();
        assert_eq!(dialog.messages.len(), 3);
        
        // Verify message order and content
        assert!(matches!(dialog.messages[0].role, MessageRole::System));
        assert_eq!(dialog.messages[0].content, "System prompt");
        
        assert!(matches!(dialog.messages[1].role, MessageRole::User));
        assert_eq!(dialog.messages[1].content, "User question");
        
        assert!(matches!(dialog.messages[2].role, MessageRole::Assistant));
        assert_eq!(dialog.messages[2].content, "Assistant response");
        
        // Verify timestamps are in order
        assert!(dialog.messages[0].timestamp <= dialog.messages[1].timestamp);
        assert!(dialog.messages[1].timestamp <= dialog.messages[2].timestamp);
    }
    
    #[tokio::test]
    async fn test_list_and_retrieve_dialogs() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        // Create multiple dialogs
        for i in 0..5 {
            let _ = manager.new_dialog_cli(
                Some(format!("Dialog {}", i)), 
                Some("test-model".to_string())
            ).await.unwrap();
            
            // Add a small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        // List dialogs
        let summaries = manager.list_recent(3).await.unwrap();
        assert_eq!(summaries.len(), 3);
        
        // Verify they are sorted by updated_at descending
        for i in 0..summaries.len()-1 {
            assert!(summaries[i].updated_at >= summaries[i+1].updated_at);
        }
        
        // Count dialogs
        let count = manager.count_dialogs().await.unwrap();
        assert_eq!(count, 5);
    }
    
    #[tokio::test]
    async fn test_continue_existing_dialog() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        // Create a dialog
        manager.new_dialog_cli(Some("Original Dialog".to_string()), None).await.unwrap();
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        
        // Add a message
        manager.add_message(&dialog_id, MessageRole::User, "First message".to_string(), None).await.unwrap();
        
        // Change current dialog
        manager.current_dialog = None;
        
        // Continue the dialog
        manager.continue_dialog_cli(dialog_id.clone()).await.unwrap();
        
        assert_eq!(manager.current_dialog, Some(dialog_id.clone()));
        
        // Verify dialog state
        let dialog = manager.dialogs.get(&dialog_id).unwrap();
        assert_eq!(dialog.title, "Original Dialog");
        assert_eq!(dialog.messages.len(), 1);
    }
    
    #[tokio::test]
    async fn test_export_dialog_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        // Create and populate a dialog
        let _ = manager.new_dialog_cli(Some("Export Test".to_string()), None).await.unwrap();
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        
        manager.add_message(&dialog_id, MessageRole::User, "Test message".to_string(), None).await.unwrap();
        
        // Export to JSON
        manager.export_dialog_cli(dialog_id.clone(), "json".to_string()).await.unwrap();
        
        // Find the exported file
        let entries = std::fs::read_dir(".").unwrap();
        let json_file = entries
            .filter_map(|e| e.ok())
            .find(|e| {
                let name = e.file_name();
                let name_str = name.to_str().unwrap();
                name_str.starts_with(&format!("dialog_{}", &dialog_id[..8])) && name_str.ends_with(".json")
            })
            .expect("JSON export file not found");
        
        // Verify content
        let content = std::fs::read_to_string(json_file.path()).unwrap();
        let exported_dialog: Dialog = serde_json::from_str(&content).unwrap();
        
        assert_eq!(exported_dialog.id, dialog_id);
        assert_eq!(exported_dialog.title, "Export Test");
        assert_eq!(exported_dialog.messages.len(), 1);
        
        // Clean up
        std::fs::remove_file(json_file.path()).unwrap();
    }
    
    #[tokio::test]
    async fn test_export_dialog_markdown() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        // Create and populate a dialog
        let _ = manager.new_dialog_cli(Some("Markdown Export".to_string()), None).await.unwrap();
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        
        manager.add_message(&dialog_id, MessageRole::System, "System setup".to_string(), None).await.unwrap();
        manager.add_message(&dialog_id, MessageRole::User, "Hello".to_string(), None).await.unwrap();
        manager.add_message(&dialog_id, MessageRole::Assistant, "Hi there!".to_string(), None).await.unwrap();
        
        // Update metadata
        if let Some(mut dialog) = manager.dialogs.get_mut(&dialog_id) {
            dialog.metadata.tags = vec!["test".to_string(), "export".to_string()];
            dialog.metadata.total_tokens = 100;
        }
        
        // Export to Markdown
        manager.export_dialog_cli(dialog_id.clone(), "markdown".to_string()).await.unwrap();
        
        // Find the exported file
        let entries = std::fs::read_dir(".").unwrap();
        let md_file = entries
            .filter_map(|e| e.ok())
            .find(|e| {
                let name = e.file_name();
                let name_str = name.to_str().unwrap();
                name_str.starts_with(&format!("dialog_{}", &dialog_id[..8])) && name_str.ends_with(".md")
            })
            .expect("Markdown export file not found");
        
        // Verify content
        let content = std::fs::read_to_string(md_file.path()).unwrap();
        
        assert!(content.contains("# Markdown Export"));
        assert!(content.contains("**Tags**: test, export"));
        assert!(content.contains("**Total Tokens**: 100"));
        assert!(content.contains("### System"));
        assert!(content.contains("System setup"));
        assert!(content.contains("### User"));
        assert!(content.contains("Hello"));
        assert!(content.contains("### Assistant"));
        assert!(content.contains("Hi there!"));
        
        // Clean up
        std::fs::remove_file(md_file.path()).unwrap();
    }
    
    #[tokio::test]
    async fn test_dialog_persistence_and_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        
        let dialog_id: String;
        
        // Create a dialog and save it
        {
            let mut manager = DialogManager::new(&config).await.unwrap();
            let _ = manager.new_dialog_cli(Some("Persistent Dialog".to_string()), Some("gpt-4".to_string())).await.unwrap();
            dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
            
            manager.add_message(&dialog_id, MessageRole::User, "Saved message".to_string(), None).await.unwrap();
        }
        
        // Create a new manager and verify the dialog was loaded from recent
        {
            let manager = DialogManager::new(&config).await.unwrap();
            
            // Dialog should be loaded from recent dialogs
            // If not, load it explicitly
            if !manager.dialogs.contains_key(&dialog_id) {
                manager.load_dialog(&dialog_id).await.unwrap();
            }
            
            // Verify loaded dialog
            let dialog = manager.dialogs.get(&dialog_id).unwrap();
            assert_eq!(dialog.title, "Persistent Dialog");
            assert_eq!(dialog.model, "gpt-4");
            assert_eq!(dialog.messages.len(), 1);
            assert_eq!(dialog.messages[0].content, "Saved message");
        }
    }
    
    #[tokio::test]
    async fn test_load_recent_dialogs() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        
        // Create multiple dialogs
        {
            let mut manager = DialogManager::new(&config).await.unwrap();
            for i in 0..15 {
                let _ = manager.new_dialog_cli(Some(format!("Dialog {}", i)), None).await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
        
        // Create a new manager that should load recent dialogs
        {
            let manager = DialogManager::new(&config).await.unwrap();
            
            // Should load only 10 most recent dialogs
            assert_eq!(manager.dialogs.len(), 10);
            
            // Verify they are the most recent ones
            let summaries = manager.list_recent(10).await.unwrap();
            for summary in summaries {
                assert!(summary.title.contains("Dialog"));
            }
        }
    }
    
    #[tokio::test]
    async fn test_dialog_id_generation() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        let mut ids = std::collections::HashSet::new();
        
        // Create multiple dialogs and ensure unique IDs
        for _ in 0..10 {
            manager.new_dialog_cli(None, None).await.unwrap();
            let id = manager.current_dialog.read().await.as_ref().unwrap().clone();
            
            // Verify ID format (UUID v4)
            assert_eq!(id.len(), 36);
            assert!(id.contains('-'));
            
            // Verify uniqueness
            assert!(ids.insert(id));
        }
    }
    
    #[tokio::test]
    async fn test_concurrent_dialog_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let manager = Arc::new(Mutex::new(DialogManager::new(&config).await.unwrap()));
        
        // Create a dialog
        let dialog_id = {
            let mut m = manager.lock().await;
            m.new_dialog_cli(Some("Concurrent Test".to_string()), None).await.unwrap();
            m.current_dialog.read().await.as_ref().unwrap().clone()
        };
        
        // Spawn multiple tasks to add messages concurrently
        let mut handles = vec![];
        
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let dialog_id_clone = dialog_id.clone();
            
            let handle = tokio::spawn(async move {
                let mut m = manager_clone.lock().await;
                m.add_message(
                    &dialog_id_clone, 
                    MessageRole::User, 
                    format!("Concurrent message {}", i)
                ).await.unwrap();
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify all messages were added
        let m = manager.lock().await;
        let dialog = m.dialogs.get(&dialog_id).unwrap();
        assert_eq!(dialog.messages.len(), 10);
        
        // Verify all messages are present
        let contents: Vec<String> = dialog.messages.iter().map(|m| m.content.clone()).collect();
        for i in 0..10 {
            assert!(contents.contains(&format!("Concurrent message {}", i)));
        }
    }
    
    #[tokio::test]
    async fn test_message_ordering_and_timestamps() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        manager.new_dialog_cli(None, None).await.unwrap();
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        
        // Add messages with small delays to ensure different timestamps
        for i in 0..5 {
            manager.add_message(
                &dialog_id, 
                MessageRole::User, 
                format!("Message {}", i)
            ).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        let dialog = manager.dialogs.get(&dialog_id).unwrap();
        
        // Verify messages are in order
        for i in 0..5 {
            assert_eq!(dialog.messages[i].content, format!("Message {}", i));
        }
        
        // Verify timestamps are increasing
        for i in 0..4 {
            assert!(dialog.messages[i].timestamp < dialog.messages[i+1].timestamp);
        }
        
        // Verify dialog updated_at is recent
        assert!(dialog.updated_at >= dialog.messages.last().unwrap().timestamp);
    }
    
    #[tokio::test]
    async fn test_error_handling_dialog_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        // Try to continue non-existent dialog
        let result = manager.continue_dialog_cli("non-existent-id".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Dialog file not found"));
        
        // Try to add message to non-existent dialog
        let result = manager.add_message("non-existent-id", MessageRole::User, "Test".to_string(), None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Dialog not found"));
        
        // Try to export non-existent dialog
        let result = manager.export_dialog_cli("non-existent-id".to_string(), "json".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Dialog not found"));
    }
    
    #[tokio::test]
    async fn test_error_handling_invalid_export_format() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        manager.new_dialog_cli(None, None).await.unwrap();
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        
        let result = manager.export_dialog_cli(dialog_id, "invalid-format".to_string()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported format"));
    }
    
    #[tokio::test]
    async fn test_dialog_metadata_management() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        manager.new_dialog_cli(None, None).await.unwrap();
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        
        // Update metadata
        {
            let mut dialog = manager.dialogs.get_mut(&dialog_id).unwrap();
            dialog.metadata.domain = Some("test-domain".to_string());
            dialog.metadata.context = Some("test-context".to_string());
            dialog.metadata.tags = vec!["tag1".to_string(), "tag2".to_string()];
            dialog.metadata.total_tokens = 500;
        }
        
        // Save and reload
        manager.save_dialog(&dialog_id).await.unwrap();
        manager.dialogs.clear();
        manager.load_dialog(&dialog_id).await.unwrap();
        
        // Verify metadata persisted
        let dialog = manager.dialogs.get(&dialog_id).unwrap();
        assert_eq!(dialog.metadata.domain, Some("test-domain".to_string()));
        assert_eq!(dialog.metadata.context, Some("test-context".to_string()));
        assert_eq!(dialog.metadata.tags, vec!["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(dialog.metadata.total_tokens, 500);
    }
    
    #[tokio::test]
    async fn test_token_counting() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        manager.new_dialog_cli(None, None).await.unwrap();
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        
        // Add messages with token counts
        {
            let mut dialog = manager.dialogs.get_mut(&dialog_id).unwrap();
            
            dialog.messages.push(Message {
                role: MessageRole::User,
                content: "Test message".to_string(),
                timestamp: Utc::now(),
                tokens: Some(10),
            });
            
            dialog.messages.push(Message {
                role: MessageRole::Assistant,
                content: "Response".to_string(),
                timestamp: Utc::now(),
                tokens: Some(20),
            });
            
            dialog.metadata.total_tokens = 30;
        }
        
        let dialog = manager.dialogs.get(&dialog_id).unwrap();
        assert_eq!(dialog.messages[0].tokens, Some(10));
        assert_eq!(dialog.messages[1].tokens, Some(20));
        assert_eq!(dialog.metadata.total_tokens, 30);
    }
    
    #[tokio::test]
    async fn test_command_handling() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);
        let mut manager = DialogManager::new(&config).await.unwrap();
        
        // Test New command
        let new_cmd = DialogCommands::New {
            title: Some("Command Test".to_string()),
            model: Some("gpt-3.5".to_string()),
        };
        let _ = manager.handle_command(new_cmd).await.unwrap();
        assert_eq!(manager.dialogs.len(), 1);
        
        // Test List command
        let list_cmd = DialogCommands::List { count: 5 };
        let _ = manager.handle_command(list_cmd).await.unwrap();
        
        // Test Continue command
        let dialog_id = manager.current_dialog.read().await.as_ref().unwrap().clone();
        manager.current_dialog = None;
        
        let continue_cmd = DialogCommands::Continue { id: dialog_id.clone() };
        let _ = manager.handle_command(continue_cmd).await.unwrap();
        assert_eq!(manager.current_dialog, Some(dialog_id.clone()));
        
        // Test Export command
        let export_cmd = DialogCommands::Export {
            id: dialog_id,
            format: "json".to_string(),
        };
        let _ = manager.handle_command(export_cmd).await.unwrap();
    }
}