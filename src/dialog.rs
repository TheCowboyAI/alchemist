//! Dialog management for AI interactions

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::{
    config::AlchemistConfig,
    ai::AiManager,
    shell_commands::DialogCommands,
};

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

pub struct DialogManager {
    dialogs: DashMap<String, Dialog>,
    storage_path: PathBuf,
    current_dialog: Option<String>,
}

impl DialogManager {
    pub async fn new(config: &AlchemistConfig) -> Result<Self> {
        let storage_path = PathBuf::from(
            shellexpand::tilde(&config.general.dialog_history_path).to_string()
        );
        
        // Ensure storage directory exists
        fs::create_dir_all(&storage_path).await?;
        
        let manager = Self {
            dialogs: DashMap::new(),
            storage_path,
            current_dialog: None,
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
        let model = model.unwrap_or_else(|| "default".to_string());
        
        let dialog = Dialog {
            id: id.clone(),
            title: title.clone(),
            model,
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
        self.current_dialog = Some(id.clone());
        
        // Save to disk
        self.save_dialog(&id).await?;
        
        println!("✅ Created new dialog: {}", title);
        println!("   ID: {}", id);
        println!("   Use 'dialog continue {}' to add messages", id);
        
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
            let current_marker = if Some(&summary.id) == self.current_dialog.as_ref() {
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
        
        self.current_dialog = Some(id.clone());
        
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
    
    pub async fn add_message(&mut self, dialog_id: &str, role: MessageRole, content: String) -> Result<()> {
        let mut dialog = self.dialogs.get_mut(dialog_id)
            .ok_or_else(|| anyhow::anyhow!("Dialog not found"))?;
        
        let message = Message {
            role,
            content,
            timestamp: Utc::now(),
            tokens: None,
        };
        
        dialog.messages.push(message);
        dialog.updated_at = Utc::now();
        
        drop(dialog); // Release the lock
        
        // Save to disk
        self.save_dialog(dialog_id).await?;
        
        Ok(())
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