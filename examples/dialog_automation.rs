//! Example: Automating dialog interactions
//!
//! This example shows how to programmatically interact with dialogs
//! for automation, testing, or integration purposes.

use alchemist::{
    config::AlchemistConfig,
    dialog::{DialogManager, MessageRole},
    ai::AiManager,
};
use anyhow::Result;
use futures::StreamExt;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize
    let config = AlchemistConfig::default();
    let mut dialog_manager = DialogManager::new(&config).await?;
    let ai_manager = AiManager::new(&config).await?;
    
    println!("🤖 Dialog Automation Example");
    println!("===========================\n");
    
    // Example 1: Simple Q&A automation
    simple_qa_example(&mut dialog_manager, &ai_manager).await?;
    
    // Example 2: Multi-turn conversation
    multi_turn_example(&mut dialog_manager, &ai_manager).await?;
    
    // Example 3: Batch processing
    batch_processing_example(&mut dialog_manager, &ai_manager).await?;
    
    Ok(())
}

async fn simple_qa_example(
    dialog_manager: &mut DialogManager,
    ai_manager: &AiManager,
) -> Result<()> {
    println!("1. Simple Q&A Example");
    println!("--------------------");
    
    // Create dialog
    let dialog_id = dialog_manager.new_dialog_cli(
        Some("Q&A Session".to_string()),
        Some("claude-3-sonnet".to_string())
    ).await?;
    
    // Questions to ask
    let questions = vec![
        "What is Rust programming language?",
        "What are Rust's main advantages?",
        "How does Rust ensure memory safety?",
    ];
    
    for question in questions {
        println!("\n❓ Question: {}", question);
        
        // Add user message
        dialog_manager.add_message(
            &dialog_id,
            MessageRole::User,
            question.to_string(),
            None
        ).await?;
        
        // Get AI response
        let prompt = format!("User: {}\n\nAssistant:", question);
        let response = ai_manager.get_completion("claude-3-sonnet", &prompt).await?;
        
        // Add AI response
        dialog_manager.add_message(
            &dialog_id,
            MessageRole::Assistant,
            response.clone(),
            Some(serde_json::json!({
                "model": "claude-3-sonnet",
                "question": question,
            }))
        ).await?;
        
        // Display response (truncated)
        let truncated = if response.len() > 200 {
            format!("{}...", &response[..200])
        } else {
            response
        };
        println!("✅ Answer: {}", truncated);
    }
    
    println!("\n");
    Ok(())
}

async fn multi_turn_example(
    dialog_manager: &mut DialogManager,
    ai_manager: &AiManager,
) -> Result<()> {
    println!("2. Multi-turn Conversation Example");
    println!("---------------------------------");
    
    let dialog_id = dialog_manager.new_dialog_cli(
        Some("Code Review".to_string()),
        Some("claude-3-sonnet".to_string())
    ).await?;
    
    // Conversation flow
    let turns = vec![
        ("User", "I have a Rust function that processes a vector. Can you help me review it?"),
        ("User", "Here's the code:\n```rust\nfn process_vec(mut v: Vec<i32>) -> Vec<i32> {\n    v.sort();\n    v.dedup();\n    v\n}\n```"),
        ("User", "Is this implementation efficient? Are there any improvements you'd suggest?"),
        ("User", "What about error handling? Should I return a Result instead?"),
    ];
    
    for (role, message) in turns {
        println!("\n{}: {}", role, message);
        
        // Add message
        dialog_manager.add_message(
            &dialog_id,
            MessageRole::User,
            message.to_string(),
            None
        ).await?;
        
        // Build conversation context
        let history = dialog_manager.get_messages(&dialog_id, 10).await?;
        let mut prompt = String::new();
        
        for msg in &history {
            match msg.role {
                MessageRole::User => prompt.push_str(&format!("User: {}\n", msg.content)),
                MessageRole::Assistant => prompt.push_str(&format!("Assistant: {}\n", msg.content)),
                _ => {}
            }
        }
        prompt.push_str("Assistant:");
        
        // Get response with streaming
        let start_time = Instant::now();
        let mut stream = ai_manager.stream_completion("claude-3-sonnet", &prompt).await?;
        let mut full_response = String::new();
        let mut token_count = 0;
        
        print!("Assistant: ");
        while let Some(chunk) = stream.next().await {
            if let Ok(token) = chunk {
                full_response.push_str(&token);
                token_count += 1;
                print!("{}", token);
            }
        }
        println!();
        
        let duration = start_time.elapsed();
        println!("⏱️  {} tokens in {:.2}s ({:.1} tokens/sec)", 
            token_count, 
            duration.as_secs_f64(),
            token_count as f64 / duration.as_secs_f64()
        );
        
        // Save response
        dialog_manager.add_message(
            &dialog_id,
            MessageRole::Assistant,
            full_response,
            Some(serde_json::json!({
                "model": "claude-3-sonnet",
                "tokens": token_count,
                "duration_ms": duration.as_millis(),
            }))
        ).await?;
    }
    
    println!("\n");
    Ok(())
}

async fn batch_processing_example(
    dialog_manager: &mut DialogManager,
    ai_manager: &AiManager,
) -> Result<()> {
    println!("3. Batch Processing Example");
    println!("--------------------------");
    
    // Create a dialog for batch processing
    let dialog_id = dialog_manager.new_dialog_cli(
        Some("Batch Translation".to_string()),
        Some("claude-3-sonnet".to_string())
    ).await?;
    
    // Items to process
    let items = vec![
        ("Hello, world!", "Spanish"),
        ("How are you today?", "French"),
        ("Thank you very much", "Japanese"),
        ("Good morning", "German"),
    ];
    
    println!("Translating {} phrases...\n", items.len());
    
    let mut results = Vec::new();
    let start_time = Instant::now();
    
    for (text, language) in items {
        let prompt = format!(
            "Translate the following text to {}:\n\n{}\n\nTranslation:",
            language, text
        );
        
        // Get translation
        let translation = ai_manager.get_completion("claude-3-sonnet", &prompt).await?;
        
        // Store in dialog
        dialog_manager.add_message(
            &dialog_id,
            MessageRole::User,
            format!("Translate to {}: {}", language, text),
            None
        ).await?;
        
        dialog_manager.add_message(
            &dialog_id,
            MessageRole::Assistant,
            translation.clone(),
            Some(serde_json::json!({
                "task": "translation",
                "source_text": text,
                "target_language": language,
            }))
        ).await?;
        
        results.push((text, language, translation));
        println!("✅ {} -> {}: {}", text, language, translation.trim());
    }
    
    let total_duration = start_time.elapsed();
    println!("\n📊 Batch complete: {} translations in {:.2}s", 
        results.len(), 
        total_duration.as_secs_f64()
    );
    
    // Export results
    let export_data = serde_json::json!({
        "task": "batch_translation",
        "total_items": results.len(),
        "duration_seconds": total_duration.as_secs_f64(),
        "results": results.iter().map(|(src, lang, trans)| {
            serde_json::json!({
                "source": src,
                "language": lang,
                "translation": trans.trim(),
            })
        }).collect::<Vec<_>>(),
    });
    
    std::fs::write(
        "translation_results.json",
        serde_json::to_string_pretty(&export_data)?
    )?;
    
    println!("💾 Results saved to translation_results.json");
    
    Ok(())
}