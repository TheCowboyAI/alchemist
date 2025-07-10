//! Example of using the Dialog UI API

use alchemist::{
    shell::AlchemistShell,
    config::AlchemistConfig,
    renderer::{RendererManager, DialogMessage},
    renderer_api::{RendererApi, DialogCommand, DialogEvent},
};
use chrono::Utc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = AlchemistConfig::load()?;
    
    // Create shell with all managers
    let shell = AlchemistShell::new(config).await?;
    
    // Example 1: Spawn a dialog window
    let dialog_id = "example-dialog-123";
    let ai_model = "claude-3";
    
    // Initial messages
    let messages = vec![
        DialogMessage {
            role: "system".to_string(),
            content: "You are a helpful AI assistant.".to_string(),
            timestamp: Utc::now(),
        },
        DialogMessage {
            role: "user".to_string(),
            content: "Hello! Can you help me understand the Alchemist system?".to_string(),
            timestamp: Utc::now(),
        },
    ];
    
    // Spawn the dialog window
    let window_id = shell.renderer_manager.spawn_dialog(
        "AI Chat - Alchemist Help",
        dialog_id.to_string(),
        ai_model.to_string(),
        messages,
        Some("You are an expert on the Alchemist CIM system.".to_string()),
    ).await?;
    
    println!("✅ Dialog window spawned with ID: {}", window_id);
    
    // Example 2: Send a streaming response
    let response_tokens = vec![
        "Hello! ",
        "I'd be happy ",
        "to help you ",
        "understand ",
        "the Alchemist system. ",
        "\n\n",
        "Alchemist is a ",
        "Control Information Model (CIM) ",
        "system that provides:\n",
        "- AI model management\n",
        "- Dialog interfaces\n", 
        "- Policy evaluation\n",
        "- Domain visualization\n",
        "- And much more!"
    ];
    
    // Create renderer API for communication
    let renderer_api = RendererApi::new();
    
    // Simulate streaming response
    for token in response_tokens {
        renderer_api.send_dialog_command(
            &window_id,
            DialogCommand::StreamToken { 
                token: token.to_string() 
            }
        ).await?;
        
        // Small delay to simulate streaming
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    
    // Complete the stream
    renderer_api.send_dialog_command(
        &window_id,
        DialogCommand::CompleteStream
    ).await?;
    
    // Example 3: Handle dialog events
    let event_receiver = renderer_api.get_event_sender();
    
    // In a real application, you would listen for events in a loop
    // This shows how to handle different event types:
    match dialog_event {
        DialogEvent::UserMessage { content } => {
            println!("User sent: {}", content);
            
            // Send to AI model
            let ai_response = shell.ai_manager.complete(
                ai_model,
                &content,
                Some("Continue the conversation naturally."),
            ).await?;
            
            // Add response to dialog
            renderer_api.send_dialog_command(
                &window_id,
                DialogCommand::AddMessage {
                    role: "assistant".to_string(),
                    content: ai_response,
                }
            ).await?;
        }
        DialogEvent::ClearRequested => {
            println!("User requested to clear dialog");
            renderer_api.send_dialog_command(
                &window_id,
                DialogCommand::ClearMessages
            ).await?;
        }
        DialogEvent::ExportRequested { format } => {
            println!("User wants to export dialog as: {}", format);
            // Export logic here
        }
        _ => {}
    }
    
    Ok(())
}

// Example of integrating with the shell command system
async fn handle_dialog_command(shell: &AlchemistShell, args: &[&str]) -> anyhow::Result<()> {
    match args.first() {
        Some(&"new") => {
            // Create new dialog with optional model selection
            let model = args.get(1).unwrap_or(&"claude-3");
            let dialog_id = uuid::Uuid::new_v4().to_string();
            
            let window_id = shell.renderer_manager.spawn_dialog(
                &format!("New Dialog - {}", model),
                dialog_id,
                model.to_string(),
                vec![],
                None,
            ).await?;
            
            println!("Created new dialog window: {}", window_id);
        }
        Some(&"list") => {
            // List active dialog windows
            let active = shell.renderer_manager.list_active();
            for (id, renderer_type, title) in active {
                if matches!(renderer_type, alchemist::renderer::RendererType::Iced) {
                    println!("📝 {} - {}", &id[..8], title);
                }
            }
        }
        _ => {
            println!("Usage: dialog [new|list|ui]");
        }
    }
    Ok(())
}