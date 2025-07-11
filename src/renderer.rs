//! Renderer management for spawning Bevy and Iced windows

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::process::{Command, Child};
use std::path::PathBuf;
use tracing::{info, warn, debug};
use dashmap::DashMap;
use uuid::Uuid;
use tokio::sync::mpsc;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RendererType {
    Bevy,
    Iced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct RenderRequest {
    pub id: String,
    pub renderer: RendererType,
    pub title: String,
    pub data: RenderData,
    pub config: RenderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RenderData {
    /// 3D graph visualization
    Graph3D {
        nodes: Vec<GraphNode>,
        edges: Vec<GraphEdge>,
    },
    /// Workflow visualization  
    Workflow {
        workflow_id: String,
        workflow_data: serde_json::Value,
    },
    /// Document viewer
    Document {
        content: String,
        format: String,
    },
    /// Text editor
    TextEditor {
        file_path: Option<String>,
        content: Option<String>,
        language: Option<String>,
    },
    /// Video player
    Video {
        url: String,
    },
    /// Music player
    Audio {
        url: String,
        playlist: Vec<String>,
    },
    /// 3D scene
    Scene3D {
        scene_data: serde_json::Value,
    },
    /// AI Dialog interface
    Dialog {
        dialog_id: String,
        ai_model: String,
        messages: Vec<DialogMessage>,
        system_prompt: Option<String>,
    },
    /// Event monitoring view
    EventMonitor {
        max_events: usize,
        initial_events: Vec<crate::event_monitor::MonitoredEvent>,
    },
    /// Markdown document viewer
    Markdown {
        content: String,
        theme: Option<String>, // "light" or "dark"
    },
    /// Chart visualization
    Chart {
        data: serde_json::Value, // Contains series data
        chart_type: String, // "line", "bar", "scatter", "pie", "area"
        options: serde_json::Value, // Chart options
    },
    /// Dashboard view
    Dashboard(crate::dashboard::DashboardData),
    /// Custom data
    Custom {
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub position: Option<(i32, i32)>,
    pub fullscreen: bool,
    pub resizable: bool,
    pub always_on_top: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub position: Option<[f32; 3]>,
    pub color: Option<[f32; 4]>,
    pub size: Option<f32>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub weight: Option<f32>,
    pub color: Option<[f32; 4]>,
}

pub struct RendererManager {
    processes: DashMap<String, RendererProcess>,
    render_binary_path: PathBuf,
    message_sender: mpsc::Sender<RendererMessage>,
    message_receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<RendererMessage>>>,
}

struct RendererProcess {
    id: String,
    renderer_type: RendererType,
    process: Child,
    title: String,
}

#[derive(Debug)]
pub enum RendererMessage {
    WindowClosed(String),
    DataUpdate(String, serde_json::Value),
    Error(String, String),
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            position: None,
            fullscreen: false,
            resizable: true,
            always_on_top: false,
        }
    }
}

impl RendererManager {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        
        // Try to find the renderer binary in several locations
        let render_binary_path = if let Ok(exe_path) = std::env::current_exe() {
            // Look for renderer next to the main binary
            let mut path = exe_path.parent().unwrap().to_path_buf();
            path.push("alchemist-renderer");
            if path.exists() {
                path
            } else {
                // Fallback to PATH
                PathBuf::from("alchemist-renderer")
            }
        } else {
            PathBuf::from("alchemist-renderer")
        };
        
        Ok(Self {
            processes: DashMap::new(),
            render_binary_path,
            message_sender: tx,
            message_receiver: Arc::new(tokio::sync::Mutex::new(rx)),
        })
    }
    
    /// Spawn a new renderer window
    pub async fn spawn(&self, request: RenderRequest) -> Result<String> {
        info!("Spawning {} renderer: {}", 
            match request.renderer {
                RendererType::Bevy => "Bevy",
                RendererType::Iced => "Iced",
            },
            request.title
        );
        
        // Serialize the request to pass to the renderer process
        let request_json = serde_json::to_string(&request)?;
        
        // Create a temporary file for complex data
        let data_file = tempfile::NamedTempFile::new()?;
        std::fs::write(data_file.path(), &request_json)?;
        
        // Spawn the renderer process
        let mut cmd = Command::new(&self.render_binary_path);
        cmd.arg(match request.renderer {
                RendererType::Bevy => "bevy",
                RendererType::Iced => "iced",
            })
            .arg("--data-file")
            .arg(data_file.path())
            .arg("--id")
            .arg(&request.id);
        
        debug!("Spawning renderer with command: {:?}", cmd);
        
        let child = cmd.spawn()
            .context("Failed to spawn renderer process")?;
        
        let process = RendererProcess {
            id: request.id.clone(),
            renderer_type: request.renderer.clone(),
            process: child,
            title: request.title.clone(),
        };
        
        self.processes.insert(request.id.clone(), process);
        
        // Keep the temp file alive until the process starts
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            drop(data_file);
        });
        
        Ok(request.id)
    }
    
    /// Spawn a 3D graph visualization in Bevy
    pub async fn spawn_graph_3d(
        &self,
        title: &str,
        nodes: Vec<GraphNode>,
        edges: Vec<GraphEdge>,
    ) -> Result<String> {
        let request = RenderRequest {
            id: Uuid::new_v4().to_string(),
            renderer: RendererType::Bevy,
            title: title.to_string(),
            data: RenderData::Graph3D { nodes, edges },
            config: RenderConfig::default(),
        };
        
        self.spawn(request).await
    }
    
    /// Spawn a document viewer in Iced
    pub async fn spawn_document(
        &self,
        title: &str,
        content: String,
        format: &str,
    ) -> Result<String> {
        let request = RenderRequest {
            id: Uuid::new_v4().to_string(),
            renderer: RendererType::Iced,
            title: title.to_string(),
            data: RenderData::Document {
                content,
                format: format.to_string(),
            },
            config: RenderConfig::default(),
        };
        
        self.spawn(request).await
    }
    
    /// Spawn a text editor in Iced
    pub async fn spawn_text_editor(
        &self,
        title: &str,
        file_path: Option<String>,
        content: Option<String>,
    ) -> Result<String> {
        let request = RenderRequest {
            id: Uuid::new_v4().to_string(),
            renderer: RendererType::Iced,
            title: title.to_string(),
            data: RenderData::TextEditor {
                file_path,
                content,
                language: None,
            },
            config: RenderConfig::default(),
        };
        
        self.spawn(request).await
    }
    
    /// Spawn a workflow visualization
    pub async fn spawn_workflow(
        &self,
        title: &str,
        workflow_id: String,
        workflow_data: serde_json::Value,
        use_3d: bool,
    ) -> Result<String> {
        let request = RenderRequest {
            id: Uuid::new_v4().to_string(),
            renderer: if use_3d { RendererType::Bevy } else { RendererType::Iced },
            title: title.to_string(),
            data: RenderData::Workflow {
                workflow_id,
                workflow_data,
            },
            config: RenderConfig::default(),
        };
        
        self.spawn(request).await
    }
    
    /// Spawn an AI dialog window
    pub async fn spawn_dialog(
        &self,
        title: &str,
        dialog_id: String,
        ai_model: String,
        messages: Vec<DialogMessage>,
        system_prompt: Option<String>,
    ) -> Result<String> {
        let request = RenderRequest {
            id: Uuid::new_v4().to_string(),
            renderer: RendererType::Iced,
            title: title.to_string(),
            data: RenderData::Dialog {
                dialog_id,
                ai_model,
                messages,
                system_prompt,
            },
            config: RenderConfig {
                width: 800,
                height: 600,
                ..RenderConfig::default()
            },
        };
        
        self.spawn(request).await
    }
    
    /// Spawn an event monitor window
    pub async fn spawn_event_monitor(
        &self,
        title: &str,
        max_events: usize,
    ) -> Result<String> {
        let request = RenderRequest {
            id: Uuid::new_v4().to_string(),
            renderer: RendererType::Iced,
            title: title.to_string(),
            data: RenderData::EventMonitor {
                max_events,
                initial_events: Vec::new(),
            },
            config: RenderConfig {
                width: 1200,
                height: 800,
                ..RenderConfig::default()
            },
        };
        
        self.spawn(request).await
    }
    
    /// Spawn a markdown viewer
    pub async fn spawn_markdown(
        &self,
        title: &str,
        content: String,
        theme: Option<&str>,
    ) -> Result<String> {
        let request = RenderRequest {
            id: Uuid::new_v4().to_string(),
            renderer: RendererType::Iced,
            title: title.to_string(),
            data: RenderData::Markdown {
                content,
                theme: theme.map(|t| t.to_string()),
            },
            config: RenderConfig {
                width: 800,
                height: 600,
                ..RenderConfig::default()
            },
        };
        
        self.spawn(request).await
    }
    
    /// Spawn a chart viewer
    pub async fn spawn_chart(
        &self,
        title: &str,
        data: serde_json::Value,
        chart_type: &str,
        options: serde_json::Value,
    ) -> Result<String> {
        let request = RenderRequest {
            id: Uuid::new_v4().to_string(),
            renderer: RendererType::Iced,
            title: title.to_string(),
            data: RenderData::Chart {
                data,
                chart_type: chart_type.to_string(),
                options,
            },
            config: RenderConfig {
                width: 1000,
                height: 700,
                ..RenderConfig::default()
            },
        };
        
        self.spawn(request).await
    }
    
    /// Send data update to a renderer
    pub async fn update_data(&self, renderer_id: &str, data: serde_json::Value) -> Result<()> {
        if let Some(_process) = self.processes.get(renderer_id) {
            // Send through our message channel
            self.message_sender
                .send(RendererMessage::DataUpdate(renderer_id.to_string(), data.clone()))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to send data update: {}", e))?;
            
            info!("Sent data update for renderer {}: {:?}", renderer_id, data);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Renderer not found: {}", renderer_id))
        }
    }
    
    /// Close a renderer window
    pub async fn close(&self, renderer_id: &str) -> Result<()> {
        if let Some((_, mut process)) = self.processes.remove(renderer_id) {
            process.process.kill()?;
            info!("Closed renderer: {}", renderer_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Renderer not found: {}", renderer_id))
        }
    }
    
    /// List active renderers
    pub fn list_active(&self) -> Vec<(String, RendererType, String)> {
        self.processes
            .iter()
            .map(|entry| {
                let p = entry.value();
                (p.id.clone(), p.renderer_type.clone(), p.title.clone())
            })
            .collect()
    }
    
    /// Check and clean up dead processes
    pub async fn cleanup_dead_processes(&self) {
        let mut to_remove = Vec::new();
        
        // Collect the IDs first
        let ids: Vec<String> = self.processes.iter()
            .map(|entry| entry.key().clone())
            .collect();
        
        // Check each process
        for id in ids {
            let mut should_remove = false;
            
            if let Some(mut entry) = self.processes.get_mut(&id) {
                match entry.process.try_wait() {
                    Ok(Some(_status)) => {
                        // Process has exited
                        should_remove = true;
                    }
                    Ok(None) => {
                        // Still running
                    }
                    Err(e) => {
                        warn!("Error checking process {}: {}", id, e);
                        should_remove = true;
                    }
                }
            }
            
            if should_remove {
                to_remove.push(id);
            }
        }
        
        for id in to_remove {
            self.processes.remove(&id);
            let _ = self.message_sender.send(RendererMessage::WindowClosed(id)).await;
        }
    }
    
    /// Process incoming renderer messages
    pub async fn process_messages(&self) {
        let mut receiver = self.message_receiver.lock().await;
        
        while let Some(message) = receiver.recv().await {
            match message {
                RendererMessage::WindowClosed(renderer_id) => {
                    info!("Renderer window closed: {}", renderer_id);
                    // Remove from active processes if not already removed
                    if self.processes.remove(&renderer_id).is_some() {
                        debug!("Removed closed renderer from active processes: {}", renderer_id);
                    }
                }
                RendererMessage::DataUpdate(renderer_id, data) => {
                    info!("Data update received for renderer {}: {:?}", renderer_id, data);
                    // Forward the data update to the appropriate renderer
                    if self.processes.contains_key(&renderer_id) {
                        // TODO: Implement NATS-based communication to forward updates
                        debug!("Would forward data update to renderer {}", renderer_id);
                    } else {
                        warn!("Data update for unknown renderer: {}", renderer_id);
                    }
                }
                RendererMessage::Error(renderer_id, error) => {
                    warn!("Error from renderer {}: {}", renderer_id, error);
                    // Handle renderer errors - potentially restart or cleanup
                    if let Some((_, mut process)) = self.processes.remove(&renderer_id) {
                        // Kill the errored process
                        if let Err(e) = process.process.kill() {
                            warn!("Failed to kill errored renderer process {}: {}", renderer_id, e);
                        }
                    }
                }
            }
        }
    }
    
    /// Start the message processing loop
    pub fn start_message_processor(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                self.process_messages().await;
                // If the receiver is closed, wait a bit before retrying
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    }
}

/// Helper to determine best renderer for content type
pub fn suggest_renderer(data: &RenderData) -> RendererType {
    match data {
        RenderData::Graph3D { .. } => RendererType::Bevy,
        RenderData::Scene3D { .. } => RendererType::Bevy,
        RenderData::Workflow { .. } => RendererType::Bevy, // 3D by default
        RenderData::Document { .. } => RendererType::Iced,
        RenderData::TextEditor { .. } => RendererType::Iced,
        RenderData::Dialog { .. } => RendererType::Iced,
        RenderData::Video { .. } => RendererType::Iced,
        RenderData::Audio { .. } => RendererType::Iced,
        RenderData::EventMonitor { .. } => RendererType::Iced,
        RenderData::Markdown { .. } => RendererType::Iced,
        RenderData::Chart { .. } => RendererType::Iced,
        RenderData::Dashboard(_) => RendererType::Iced,
        RenderData::Custom { .. } => RendererType::Iced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_suggest_renderer() {
        let graph_data = RenderData::Graph3D {
            nodes: vec![],
            edges: vec![],
        };
        assert!(matches!(suggest_renderer(&graph_data), RendererType::Bevy));
        
        let doc_data = RenderData::Document {
            content: "test".to_string(),
            format: "markdown".to_string(),
        };
        assert!(matches!(suggest_renderer(&doc_data), RendererType::Iced));
    }
}