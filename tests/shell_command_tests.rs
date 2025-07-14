//! Comprehensive tests for shell command parsing and execution

use alchemist::{
    shell_commands::*,
    shell::AlchemistShell,
    config::{AlchemistConfig, CacheConfig, PolicyConfig},
    render_commands::RenderCommands,
};
use clap::Parser;
use anyhow::Result;
use tempfile::TempDir;

#[derive(Parser)]
struct TestCli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(test)]
mod command_parsing_tests {
    use super::*;
    
    #[test]
    fn test_ai_command_parsing() {
        // Test AI list command
        let args = vec!["test", "ai", "list"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Ai { command: AiCommands::List }));
        
        // Test AI add command
        let args = vec!["test", "ai", "add", "gpt-4", "-p", "openai"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Ai { command: AiCommands::Add { name, provider, .. } } => {
                assert_eq!(name, "gpt-4");
                assert_eq!(provider, "openai");
            }
            _ => panic!("Expected AI Add command"),
        }
        
        // Test AI test command
        let args = vec!["test", "ai", "test", "claude-3"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Ai { command: AiCommands::Test { name } } => {
                assert_eq!(name, "claude-3");
            }
            _ => panic!("Expected AI Test command"),
        }
    }
    
    #[test]
    fn test_dialog_command_parsing() {
        // Test dialog new
        let args = vec!["test", "dialog", "new", "-t", "Test Dialog"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Dialog { command: DialogCommands::New { title, .. } } => {
                assert_eq!(title, Some("Test Dialog".to_string()));
            }
            _ => panic!("Expected Dialog New command"),
        }
        
        // Test dialog list
        let args = vec!["test", "dialog", "list"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Dialog { command: DialogCommands::List { .. } }));
    }
    
    #[test]
    fn test_policy_command_parsing() {
        // Test policy list
        let args = vec!["test", "policy", "list"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Policy { command: PolicyCommands::List { .. } }));
        
        // Test policy show
        let args = vec!["test", "policy", "show", "policy-123"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Policy { command: PolicyCommands::Show { id } } => {
                assert_eq!(id, "policy-123");
            }
            _ => panic!("Expected Policy Show command"),
        }
    }
    
    #[test]
    fn test_domain_command_parsing() {
        // Test domain list
        let args = vec!["test", "domain", "list"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Domain { command: DomainCommands::List }));
        
        // Test domain show
        let args = vec!["test", "domain", "show", "workflow"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Domain { command: DomainCommands::Show { name } } => {
                assert_eq!(name, "workflow");
            }
            _ => panic!("Expected Domain Show command"),
        }
    }
    
    #[test]
    fn test_deploy_command_parsing() {
        // Test deploy list
        let args = vec!["test", "deploy", "list"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Deploy { command: DeployCommands::List }));
        
        // Test deploy status
        let args = vec!["test", "deploy", "status", "deploy-123"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Deploy { command: DeployCommands::Status { id } } => {
                assert_eq!(id, "deploy-123");
            }
            _ => panic!("Expected Deploy Status command"),
        }
    }
    
    #[test]
    fn test_progress_command_parsing() {
        // Test progress command
        let args = vec!["test", "progress"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Progress { file, .. } => {
                assert_eq!(file, "doc/progress/progress.json");
            }
            _ => panic!("Expected Progress command"),
        }
    }
    
    #[test]
    fn test_render_command_parsing() {
        // Test render status
        let args = vec!["test", "render", "status"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Render { command } => {
                assert!(matches!(command, RenderCommands::Status));
            }
            _ => panic!("Expected Render command"),
        }
        
        // Test render list
        let args = vec!["test", "render", "list"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Render { command } => {
                assert!(matches!(command, RenderCommands::List));
            }
            _ => panic!("Expected Render command"),
        }
    }
}

#[cfg(test)]
mod command_execution_tests {
    use super::*;
    use std::fs;
    use std::collections::HashMap;
    
    async fn create_test_shell() -> Result<(AlchemistShell, TempDir)> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("config.toml");
        
        let mut config = AlchemistConfig {
            general: Default::default(),
            ai_models: HashMap::new(),
            policy: PolicyConfig {
                storage_path: temp_dir.path().join("policies").to_string_lossy().to_string(),
                validation_enabled: true,
                evaluation_timeout: 5000,
                cache_ttl: Some(300),
            },
            deployments: HashMap::new(),
            domains: Default::default(),
            cache: Some(CacheConfig {
                redis_url: None,
                default_ttl: 3600,
                max_memory_items: 1000,
            }),
        };
        
        // Update paths to use temp directory
        config.general.dialog_history_path = temp_dir.path().join("dialogs").to_string_lossy().to_string();
        config.policy.storage_path = temp_dir.path().join("policies").to_string_lossy().to_string();
        
        // Create required directories
        std::fs::create_dir_all(&config.general.dialog_history_path)?;
        std::fs::create_dir_all(&config.policy.storage_path)?;
        
        let shell = AlchemistShell::new(config).await?;
        Ok((shell, temp_dir))
    }
    
    #[tokio::test]
    async fn test_ai_command_execution() -> Result<()> {
        let (mut shell, _temp_dir) = create_test_shell().await?;
        
        // Test listing AI models (should be empty initially)
        shell.handle_ai_command(AiCommands::List).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_domain_command_execution() -> Result<()> {
        let (mut shell, _temp_dir) = create_test_shell().await?;
        
        // Test listing domains
        shell.handle_domain_command(DomainCommands::List).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_dialog_list_execution() -> Result<()> {
        let (mut shell, _temp_dir) = create_test_shell().await?;
        
        // Test listing dialogs
        shell.handle_dialog_command(DialogCommands::List { count: 10 }).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_policy_list_execution() -> Result<()> {
        let (mut shell, _temp_dir) = create_test_shell().await?;
        
        // Test listing policies
        shell.handle_policy_command(PolicyCommands::List { domain: None }).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_deploy_list_execution() -> Result<()> {
        let (mut shell, _temp_dir) = create_test_shell().await?;
        
        // Test listing deployments
        shell.handle_deploy_command(DeployCommands::List).await?;
        
        Ok(())
    }
}