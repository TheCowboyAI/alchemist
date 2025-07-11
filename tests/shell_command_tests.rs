//! Comprehensive tests for shell command parsing and execution

use alchemist::{
    shell_commands::*,
    shell::{AlchemistShell, create_shell},
    config::AlchemistConfig,
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
        // Test AI providers command
        let args = vec!["test", "ai", "providers"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Ai { command: AiCommands::Providers }));
        
        // Test AI models command
        let args = vec!["test", "ai", "models"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Ai { command: AiCommands::Models }));
        
        // Test AI test command with model
        let args = vec!["test", "ai", "test", "--model", "gpt-4"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Ai { command: AiCommands::Test { model } } => {
                assert_eq!(model, Some("gpt-4".to_string()));
            }
            _ => panic!("Expected AI test command"),
        }
    }
    
    #[test]
    fn test_dialog_command_parsing() {
        // Test dialog new command
        let args = vec!["test", "dialog", "new", "My Dialog"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Dialog { command: DialogCommands::New { title, model } } => {
                assert_eq!(title, "My Dialog");
                assert!(model.is_none());
            }
            _ => panic!("Expected Dialog new command"),
        }
        
        // Test dialog new with model
        let args = vec!["test", "dialog", "new", "My Dialog", "--model", "claude"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Dialog { command: DialogCommands::New { title, model } } => {
                assert_eq!(title, "My Dialog");
                assert_eq!(model, Some("claude".to_string()));
            }
            _ => panic!("Expected Dialog new command"),
        }
        
        // Test dialog list
        let args = vec!["test", "dialog", "list"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Dialog { command: DialogCommands::List }));
    }
    
    #[test]
    fn test_policy_command_parsing() {
        // Test policy new
        let args = vec!["test", "policy", "new", "my-policy", "domain1"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Policy { command: PolicyCommands::New { name, domain } } => {
                assert_eq!(name, "my-policy");
                assert_eq!(domain, "domain1");
            }
            _ => panic!("Expected Policy new command"),
        }
        
        // Test policy list
        let args = vec!["test", "policy", "list"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Policy { command: PolicyCommands::List { domain } } => {
                assert!(domain.is_none());
            }
            _ => panic!("Expected Policy list command"),
        }
        
        // Test policy list with domain filter
        let args = vec!["test", "policy", "list", "--domain", "test"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Policy { command: PolicyCommands::List { domain } } => {
                assert_eq!(domain, Some("test".to_string()));
            }
            _ => panic!("Expected Policy list command"),
        }
        
        // Test claims add
        let args = vec!["test", "policy", "claims", "add", "admin", "--description", "Admin access"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Policy { command: PolicyCommands::Claims { command } } => {
                match command {
                    ClaimsCommands::Add { name, description } => {
                        assert_eq!(name, "admin");
                        assert_eq!(description, Some("Admin access".to_string()));
                    }
                    _ => panic!("Expected Claims add command"),
                }
            }
            _ => panic!("Expected Policy claims command"),
        }
    }
    
    #[test]
    fn test_deploy_command_parsing() {
        // Test deploy list
        let args = vec!["test", "deploy", "list"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Deploy { command: DeployCommands::List }));
        
        // Test deploy with domains
        let args = vec!["test", "deploy", "deploy", "production", "-d", "graph", "-d", "workflow"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Deploy { command: DeployCommands::Deploy { target, domains } } => {
                assert_eq!(target, "production");
                assert_eq!(domains, vec!["graph", "workflow"]);
            }
            _ => panic!("Expected Deploy deploy command"),
        }
        
        // Test pipeline creation
        let args = vec!["test", "deploy", "pipeline", "release-v2", "-e", "dev", "-e", "prod", "--canary"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Deploy { command: DeployCommands::Pipeline { name, environments, canary } } => {
                assert_eq!(name, "release-v2");
                assert_eq!(environments, vec!["dev", "prod"]);
                assert!(canary);
            }
            _ => panic!("Expected Deploy pipeline command"),
        }
        
        // Test approval
        let args = vec!["test", "deploy", "approve", "approval-123", "--approve", "-c", "LGTM"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Deploy { command: DeployCommands::Approve { id, approve, comments } } => {
                assert_eq!(id, "approval-123");
                assert!(approve);
                assert_eq!(comments, Some("LGTM".to_string()));
            }
            _ => panic!("Expected Deploy approve command"),
        }
    }
    
    #[test]
    fn test_domain_command_parsing() {
        // Test domain list
        let args = vec!["test", "domain", "list"];
        let cli = TestCli::parse_from(args);
        assert!(matches!(cli.command, Commands::Domain { command: DomainCommands::List }));
        
        // Test domain tree
        let args = vec!["test", "domain", "tree", "--root", "cim"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Domain { command: DomainCommands::Tree { root } } => {
                assert_eq!(root, Some("cim".to_string()));
            }
            _ => panic!("Expected Domain tree command"),
        }
        
        // Test domain graph
        let args = vec!["test", "domain", "graph", "--format", "dot"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Domain { command: DomainCommands::Graph { format } } => {
                assert_eq!(format, "dot");
            }
            _ => panic!("Expected Domain graph command"),
        }
    }
    
    #[test]
    fn test_workflow_command_parsing() {
        // Test workflow new
        let args = vec!["test", "workflow", "new", "data-pipeline", "--description", "ETL workflow"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Workflow { command: WorkflowCommands::New { name, description, file } } => {
                assert_eq!(name, "data-pipeline");
                assert_eq!(description, Some("ETL workflow".to_string()));
                assert!(file.is_none());
            }
            _ => panic!("Expected Workflow new command"),
        }
        
        // Test workflow new from file
        let args = vec!["test", "workflow", "new", "import-workflow", "--file", "workflow.yaml"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Workflow { command: WorkflowCommands::New { name, description, file } } => {
                assert_eq!(name, "import-workflow");
                assert!(description.is_none());
                assert_eq!(file, Some("workflow.yaml".to_string()));
            }
            _ => panic!("Expected Workflow new command"),
        }
        
        // Test workflow execute
        let args = vec!["test", "workflow", "execute", "workflow-123", "--params", "key1=value1", "--params", "key2=value2"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Workflow { command: WorkflowCommands::Execute { id, params } } => {
                assert_eq!(id, "workflow-123");
                assert_eq!(params, vec!["key1=value1", "key2=value2"]);
            }
            _ => panic!("Expected Workflow execute command"),
        }
    }
    
    #[test]
    fn test_progress_command_parsing() {
        // Test progress with default format
        let args = vec!["test", "progress"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Progress { file, format } => {
                assert_eq!(file, "doc/progress/progress.json");
                assert!(matches!(format, alchemist::progress::ProgressFormat::Tree));
            }
            _ => panic!("Expected Progress command"),
        }
        
        // Test progress with custom file and format
        let args = vec!["test", "progress", "-p", "custom.json", "-f", "json"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Progress { file, format } => {
                assert_eq!(file, "custom.json");
                assert!(matches!(format, alchemist::progress::ProgressFormat::Json));
            }
            _ => panic!("Expected Progress command"),
        }
    }
    
    #[test]
    fn test_render_command_parsing() {
        // Test render dashboard
        let args = vec!["test", "render", "dashboard"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Render { command } => {
                assert!(matches!(command, RenderCommands::Dashboard));
            }
            _ => panic!("Expected Render command"),
        }
        
        // Test render bevy
        let args = vec!["test", "render", "bevy"];
        let cli = TestCli::parse_from(args);
        match cli.command {
            Commands::Render { command } => {
                assert!(matches!(command, RenderCommands::Bevy));
            }
            _ => panic!("Expected Render command"),
        }
    }
}

#[cfg(test)]
mod command_execution_tests {
    use super::*;
    
    async fn create_test_shell() -> Result<(AlchemistShell, TempDir)> {
        let temp_dir = TempDir::new()?;
        let mut config = AlchemistConfig::default();
        
        // Configure test paths
        config.general.home_dir = temp_dir.path().to_str().unwrap().to_string();
        config.general.dialog_history_path = temp_dir.path().join("dialogs").to_str().unwrap().to_string();
        config.policy.storage_path = temp_dir.path().join("policies").to_str().unwrap().to_string();
        
        // Create required directories
        std::fs::create_dir_all(&config.general.dialog_history_path)?;
        std::fs::create_dir_all(&config.policy.storage_path)?;
        
        let shell = create_shell(&config).await?;
        Ok((shell, temp_dir))
    }
    
    #[tokio::test]
    async fn test_ai_command_execution() -> Result<()> {
        let (mut shell, _temp) = create_test_shell().await?;
        
        // Test AI providers
        let result = shell.execute("ai providers").await?;
        assert!(result.contains("AI Providers"));
        
        // Test AI models
        let result = shell.execute("ai models").await?;
        assert!(result.contains("Available Models"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_dialog_command_execution() -> Result<()> {
        let (mut shell, _temp) = create_test_shell().await?;
        
        // Create dialog
        let result = shell.execute("dialog new \"Test Dialog\"").await?;
        assert!(result.contains("Created new dialog"));
        
        // List dialogs
        let result = shell.execute("dialog list").await?;
        assert!(result.contains("Test Dialog"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_policy_command_execution() -> Result<()> {
        let (mut shell, _temp) = create_test_shell().await?;
        
        // Create policy
        let result = shell.execute("policy new test-policy test-domain").await?;
        assert!(result.contains("Created new policy"));
        
        // List policies
        let result = shell.execute("policy list").await?;
        assert!(result.contains("test-policy") || result.contains("Policies"));
        
        // Add claim
        let result = shell.execute("policy claims add test-claim").await?;
        assert!(result.contains("Added claim"));
        
        // List claims
        let result = shell.execute("policy claims list").await?;
        assert!(result.contains("test-claim") || result.contains("Claims"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_invalid_command_handling() -> Result<()> {
        let (mut shell, _temp) = create_test_shell().await?;
        
        // Test completely invalid command
        let result = shell.execute("invalid_command_xyz").await;
        assert!(result.is_err() || result.unwrap().contains("Unknown"));
        
        // Test invalid subcommand
        let result = shell.execute("ai invalid_subcommand").await;
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_command_with_special_characters() -> Result<()> {
        let (mut shell, _temp) = create_test_shell().await?;
        
        // Test dialog with special characters
        let result = shell.execute(r#"dialog new "Test @ Dialog #1""#).await?;
        assert!(result.contains("Created new dialog"));
        
        // Test with escaped quotes
        let result = shell.execute(r#"dialog new "Dialog with \"quotes\"""#).await?;
        assert!(result.contains("Created new dialog"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_long_running_commands() -> Result<()> {
        let (mut shell, _temp) = create_test_shell().await?;
        
        // Test command that might take time (AI test)
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            shell.execute("ai test")
        ).await;
        
        assert!(result.is_ok(), "Command should complete within timeout");
        
        Ok(())
    }
}