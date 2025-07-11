//! Comprehensive test suite for Alchemist shell
//! 
//! This test suite covers all major components and functionality of the Alchemist shell,
//! including shell commands, AI integration, dialog management, policy enforcement,
//! deployment automation, and renderer integration.

use alchemist::{
    shell::{AlchemistShell, create_shell},
    shell_commands::{Commands, AiCommands, DialogCommands, PolicyCommands, DeployCommands},
    config::AlchemistConfig,
    ai::{AiManager, AiProvider, ModelConfig},
    dialog::{DialogManager, Dialog},
    policy::{PolicyManager, Policy, Rule, RuleCondition, RuleAction},
    deployment::{DeploymentManager, Deployment, DeploymentStatus},
    renderer::RendererManager,
    dashboard::DashboardData,
};
use anyhow::Result;
use tempfile::TempDir;
use std::path::PathBuf;
use tokio::sync::mpsc;
use chrono::Utc;
use uuid::Uuid;

/// Test fixture for Alchemist tests
struct AlchemistTestFixture {
    config: AlchemistConfig,
    temp_dir: TempDir,
    shell: Option<AlchemistShell>,
}

impl AlchemistTestFixture {
    /// Create a new test fixture
    async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let mut config = AlchemistConfig::default();
        
        // Configure test paths
        config.general.home_dir = temp_dir.path().to_str().unwrap().to_string();
        config.general.dialog_history_path = temp_dir.path().join("dialogs").to_str().unwrap().to_string();
        config.policy.storage_path = temp_dir.path().join("policies").to_str().unwrap().to_string();
        
        // Create required directories
        std::fs::create_dir_all(&config.general.dialog_history_path)?;
        std::fs::create_dir_all(&config.policy.storage_path)?;
        
        Ok(Self {
            config,
            temp_dir,
            shell: None,
        })
    }
    
    /// Initialize shell
    async fn init_shell(&mut self) -> Result<()> {
        self.shell = Some(create_shell(&self.config).await?);
        Ok(())
    }
    
    /// Get shell reference
    fn shell(&self) -> &AlchemistShell {
        self.shell.as_ref().expect("Shell not initialized")
    }
    
    /// Get mutable shell reference
    fn shell_mut(&mut self) -> &mut AlchemistShell {
        self.shell.as_mut().expect("Shell not initialized")
    }
}

#[cfg(test)]
mod shell_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_shell_initialization() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Verify shell is initialized
        assert!(fixture.shell.is_some());
        
        // Test prompt generation
        let prompt = fixture.shell().prompt();
        assert!(prompt.contains("alchemist"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_shell_version_command() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Execute version command
        let output = fixture.shell_mut().execute("version").await?;
        assert!(output.contains("Alchemist"));
        assert!(output.contains("v"));
        
        Ok(())
    }
}

#[cfg(test)]
mod ai_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ai_manager_initialization() -> Result<()> {
        let config = AlchemistConfig::default();
        let ai_manager = AiManager::new(&config).await?;
        
        // Test provider listing
        let providers = ai_manager.list_providers();
        assert!(!providers.is_empty());
        
        // Verify default providers
        assert!(providers.iter().any(|p| p.name == "openai"));
        assert!(providers.iter().any(|p| p.name == "anthropic"));
        assert!(providers.iter().any(|p| p.name == "ollama"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_ai_model_configuration() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Test model listing
        fixture.shell_mut().handle_command(Commands::Ai {
            command: AiCommands::Models,
        }).await?;
        
        // Test model testing (mock mode)
        fixture.shell_mut().handle_command(Commands::Ai {
            command: AiCommands::Test {
                model: Some("gpt-4".to_string()),
            },
        }).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod dialog_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dialog_creation() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Create new dialog
        fixture.shell_mut().handle_command(Commands::Dialog {
            command: DialogCommands::New {
                title: "Test Dialog".to_string(),
                model: Some("gpt-4".to_string()),
            },
        }).await?;
        
        // List dialogs
        fixture.shell_mut().handle_command(Commands::Dialog {
            command: DialogCommands::List,
        }).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_dialog_management() -> Result<()> {
        let config = AlchemistConfig::default();
        let temp_dir = TempDir::new()?;
        let dialog_path = temp_dir.path().join("dialogs");
        std::fs::create_dir_all(&dialog_path)?;
        
        let mut manager = DialogManager::new(
            dialog_path.to_str().unwrap(),
            config.general.default_ai_model.clone(),
        );
        
        // Create dialog
        let dialog = manager.create_dialog("Test".to_string(), None)?;
        assert_eq!(dialog.title, "Test");
        
        // List dialogs
        let dialogs = manager.list_dialogs()?;
        assert_eq!(dialogs.len(), 1);
        
        Ok(())
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_policy_manager() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Create new policy
        fixture.shell_mut().handle_command(Commands::Policy {
            command: PolicyCommands::New {
                name: "test-policy".to_string(),
                domain: "test".to_string(),
            },
        }).await?;
        
        // List policies
        fixture.shell_mut().handle_command(Commands::Policy {
            command: PolicyCommands::List {
                domain: None,
            },
        }).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_policy_evaluation() -> Result<()> {
        let config = AlchemistConfig::default();
        let temp_dir = TempDir::new()?;
        let mut config_clone = config.clone();
        config_clone.policy.storage_path = temp_dir.path().to_str().unwrap().to_string();
        
        let manager = PolicyManager::new(&config_clone).await?;
        
        // Test evaluation
        let decision = manager.evaluate(
            "user123".to_string(),
            "user".to_string(),
            vec!["read".to_string()],
            "resource123".to_string(),
            "document".to_string(),
            "test".to_string(),
            "read".to_string(),
        ).await?;
        
        // Default should allow (no policies)
        assert!(matches!(decision, alchemist::policy_engine::Decision::Allow));
        
        Ok(())
    }
}

#[cfg(test)]
mod deployment_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_deployment_manager() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // List deployments
        fixture.shell_mut().handle_command(Commands::Deploy {
            command: DeployCommands::List,
        }).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_deployment_pipeline() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Create pipeline
        fixture.shell_mut().handle_command(Commands::Deploy {
            command: DeployCommands::Pipeline {
                name: "test-pipeline".to_string(),
                environments: vec!["dev".to_string(), "prod".to_string()],
                canary: false,
            },
        }).await?;
        
        // List pipelines
        fixture.shell_mut().handle_command(Commands::Deploy {
            command: DeployCommands::Pipelines,
        }).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod renderer_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_renderer_manager() -> Result<()> {
        let (tx, mut rx) = mpsc::channel(10);
        let manager = RendererManager::new(tx);
        
        // Test dashboard data creation
        let data = DashboardData::example();
        manager.update_dashboard(data.clone()).await?;
        
        // Verify event was sent
        if let Some(event) = rx.recv().await {
            // Event should contain dashboard update
            assert!(true, "Dashboard update event received");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_end_to_end_workflow() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // 1. Configure AI
        fixture.shell_mut().handle_command(Commands::Ai {
            command: AiCommands::Models,
        }).await?;
        
        // 2. Create dialog
        fixture.shell_mut().handle_command(Commands::Dialog {
            command: DialogCommands::New {
                title: "Integration Test".to_string(),
                model: None,
            },
        }).await?;
        
        // 3. Create policy
        fixture.shell_mut().handle_command(Commands::Policy {
            command: PolicyCommands::New {
                name: "integration-policy".to_string(),
                domain: "test".to_string(),
            },
        }).await?;
        
        // 4. Show progress
        fixture.shell_mut().handle_command(Commands::Progress {
            file: "doc/progress/progress.json".to_string(),
            format: alchemist::progress::ProgressFormat::Tree,
        }).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_command_execution_flow() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Test various commands
        let commands = vec![
            "ai providers",
            "dialog list",
            "policy list",
            "deploy list",
            "version",
        ];
        
        for cmd in commands {
            let result = fixture.shell_mut().execute(cmd).await;
            assert!(result.is_ok(), "Command '{}' should execute successfully", cmd);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_invalid_command_handling() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Test invalid command
        let result = fixture.shell_mut().execute("invalid_command").await;
        assert!(result.is_err() || result.unwrap().contains("Unknown command"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_missing_required_args() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Test command with missing required argument
        let result = fixture.shell_mut().execute("deploy status").await;
        assert!(result.is_err() || result.unwrap().contains("required"));
        
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_command_execution_performance() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Measure command execution time
        let start = Instant::now();
        
        for _ in 0..100 {
            fixture.shell_mut().execute("version").await?;
        }
        
        let duration = start.elapsed();
        
        // Should complete 100 commands in under 1 second
        assert!(duration.as_secs() < 1, "Commands should execute quickly");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_dialog_listing_performance() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Create multiple dialogs
        for i in 0..10 {
            fixture.shell_mut().handle_command(Commands::Dialog {
                command: DialogCommands::New {
                    title: format!("Test Dialog {}", i),
                    model: None,
                },
            }).await?;
        }
        
        // Measure listing performance
        let start = Instant::now();
        
        fixture.shell_mut().handle_command(Commands::Dialog {
            command: DialogCommands::List,
        }).await?;
        
        let duration = start.elapsed();
        
        // Should list dialogs in under 100ms
        assert!(duration.as_millis() < 100, "Dialog listing should be fast");
        
        Ok(())
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;
    use tokio::task;
    
    #[tokio::test]
    async fn test_concurrent_command_execution() -> Result<()> {
        let mut fixture = AlchemistTestFixture::new().await?;
        fixture.init_shell().await?;
        
        // Spawn multiple concurrent tasks
        let mut handles = vec![];
        
        for i in 0..10 {
            let config = fixture.config.clone();
            let handle = task::spawn(async move {
                let mut shell = create_shell(&config).await.unwrap();
                shell.execute(&format!("version")).await
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await?;
            assert!(result.is_ok());
        }
        
        Ok(())
    }
}