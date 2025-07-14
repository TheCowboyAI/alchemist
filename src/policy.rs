//! Policy and claims management

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::AlchemistConfig,
    shell_commands::{PolicyCommands, ClaimsCommands},
    policy_engine::{PolicyEngine, EvaluationContext, Subject, Resource, Action, Decision},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub description: String,
    pub rules: Vec<Rule>,
    pub claims: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub condition: RuleCondition,
    pub action: RuleAction,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    Always,
    HasClaim(String),
    HasAllClaims(Vec<String>),
    HasAnyClaim(Vec<String>),
    DomainIs(String),
    EventType(String),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Deny,
    RequireApproval,
    Log,
    Transform(String),
    Delegate(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub name: String,
    pub description: String,
    pub domain: Option<String>,
    pub permissions: Vec<String>,
}

pub struct PolicyManager {
    policies: DashMap<String, Policy>,
    claims: DashMap<String, Claim>,
    storage_path: PathBuf,
    validation_enabled: bool,
    engine: PolicyEngine,
}

impl PolicyManager {
    pub async fn new(config: &AlchemistConfig) -> Result<Self> {
        let storage_path = PathBuf::from(
            shellexpand::tilde(&config.policy.storage_path).to_string()
        );
        
        // Ensure storage directories exist
        let policies_dir = storage_path.join("policies");
        let claims_dir = storage_path.join("claims");
        fs::create_dir_all(&policies_dir).await?;
        fs::create_dir_all(&claims_dir).await?;
        
        let mut manager = Self {
            policies: DashMap::new(),
            claims: DashMap::new(),
            storage_path,
            validation_enabled: config.policy.validation_enabled,
            engine: PolicyEngine::new(config.policy.cache_ttl.unwrap_or(300)),
        };
        
        // Load existing policies and claims
        manager.load_all().await?;
        
        // Initialize default claims if none exist
        if manager.claims.is_empty() {
            manager.initialize_default_claims().await?;
        }
        
        Ok(manager)
    }
    
    pub async fn handle_command(&mut self, command: PolicyCommands) -> Result<()> {
        match command {
            PolicyCommands::List { domain } => {
                self.list_policies_cli(domain).await?;
            }
            PolicyCommands::Show { id } => {
                self.show_policy_cli(id).await?;
            }
            PolicyCommands::Edit { id } => {
                self.edit_policy_cli(id).await?;
            }
            PolicyCommands::New { name, domain } => {
                self.new_policy_cli(name, domain).await?;
            }
            PolicyCommands::Claims { command } => {
                self.handle_claims_command(command).await?;
            }
        }
        Ok(())
    }
    
    async fn handle_claims_command(&mut self, command: ClaimsCommands) -> Result<()> {
        match command {
            ClaimsCommands::List => {
                self.list_claims_cli().await?;
            }
            ClaimsCommands::Add { name, description } => {
                self.add_claim_cli(name, description).await?;
            }
            ClaimsCommands::Remove { name } => {
                self.remove_claim_cli(name).await?;
            }
        }
        Ok(())
    }
    
    pub async fn count_policies(&self) -> Result<usize> {
        Ok(self.policies.len())
    }
    
    pub async fn list_policies(&self, domain: Option<String>) -> Result<Vec<PolicySummary>> {
        let mut summaries: Vec<PolicySummary> = self.policies
            .iter()
            .filter(|entry| {
                if let Some(ref d) = domain {
                    entry.value().domain == *d
                } else {
                    true
                }
            })
            .map(|entry| {
                let policy = entry.value();
                PolicySummary {
                    id: policy.id.clone(),
                    name: policy.name.clone(),
                    domain: policy.domain.clone(),
                    enabled: policy.enabled,
                    rule_count: policy.rules.len(),
                    claim_count: policy.claims.len(),
                }
            })
            .collect();
        
        summaries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(summaries)
    }
    
    async fn list_policies_cli(&self, domain: Option<String>) -> Result<()> {
        let policies = self.list_policies(domain.clone()).await?;
        
        if policies.is_empty() {
            if let Some(d) = domain {
                println!("No policies found for domain: {}", d);
            } else {
                println!("No policies found. Use 'ia policy new' to create one.");
            }
            return Ok(());
        }
        
        println!("📜 Policies:");
        println!("─────────────────────────────");
        
        for policy in policies {
            let status = if policy.enabled { "✅" } else { "⏸️" };
            
            println!(
                "{} {} - {} (domain: {})",
                status,
                policy.name,
                policy.id[..8].to_string(),
                policy.domain
            );
            println!(
                "   Rules: {} | Claims: {}",
                policy.rule_count,
                policy.claim_count
            );
        }
        
        Ok(())
    }
    
    async fn show_policy_cli(&self, id: String) -> Result<()> {
        let policy = self.policies.get(&id)
            .ok_or_else(|| anyhow::anyhow!("Policy not found: {}", id))?;
        
        println!("📋 Policy: {}", policy.name);
        println!("─────────────────────────────");
        println!("ID: {}", policy.id);
        println!("Domain: {}", policy.domain);
        println!("Description: {}", policy.description);
        println!("Status: {}", if policy.enabled { "Enabled" } else { "Disabled" });
        println!("Created: {}", policy.created_at.format("%Y-%m-%d %H:%M"));
        println!("Updated: {}", policy.updated_at.format("%Y-%m-%d %H:%M"));
        
        if !policy.claims.is_empty() {
            println!("\n🔑 Required Claims:");
            for claim in &policy.claims {
                println!("  • {}", claim);
            }
        }
        
        if !policy.rules.is_empty() {
            println!("\n📏 Rules:");
            for (idx, rule) in policy.rules.iter().enumerate() {
                println!("  {}. [Priority: {}]", idx + 1, rule.priority);
                println!("     Condition: {}", self.format_condition(&rule.condition));
                println!("     Action: {}", self.format_action(&rule.action));
            }
        }
        
        Ok(())
    }
    
    async fn edit_policy_cli(&mut self, id: String) -> Result<()> {
        if !self.policies.contains_key(&id) {
            return Err(anyhow::anyhow!("Policy not found: {}", id));
        }
        
        println!("📝 Editing policy: {}", id);
        println!("Note: In a full implementation, this would open an editor.");
        println!("For now, use the API or configuration files to edit policies.");
        
        Ok(())
    }
    
    async fn new_policy_cli(&mut self, name: String, domain: String) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        
        let policy = Policy {
            id: id.clone(),
            name: name.clone(),
            domain: domain.clone(),
            description: format!("Policy for {}", domain),
            rules: Vec::new(),
            claims: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            enabled: true,
        };
        
        self.policies.insert(id.clone(), policy);
        self.save_policy(&id).await?;
        
        println!("✅ Created new policy: {}", name);
        println!("   ID: {}", id);
        println!("   Domain: {}", domain);
        println!("   Use 'ia policy edit {}' to add rules", id);
        
        Ok(())
    }
    
    async fn list_claims_cli(&self) -> Result<()> {
        if self.claims.is_empty() {
            println!("No claims defined. Use 'ia policy claims add' to create one.");
            return Ok(());
        }
        
        println!("🔑 Available Claims:");
        println!("─────────────────────────────");
        
        let mut claims: Vec<_> = self.claims.iter().collect();
        claims.sort_by(|a, b| a.key().cmp(b.key()));
        
        for entry in claims {
            let claim = entry.value();
            println!("• {} - {}", claim.name, claim.description);
            
            if let Some(domain) = &claim.domain {
                println!("  Domain: {}", domain);
            }
            
            if !claim.permissions.is_empty() {
                println!("  Permissions: {}", claim.permissions.join(", "));
            }
        }
        
        Ok(())
    }
    
    async fn add_claim_cli(&mut self, name: String, description: Option<String>) -> Result<()> {
        if self.claims.contains_key(&name) {
            return Err(anyhow::anyhow!("Claim already exists: {}", name));
        }
        
        let claim = Claim {
            name: name.clone(),
            description: description.unwrap_or_else(|| format!("Claim: {}", name)),
            domain: None,
            permissions: Vec::new(),
        };
        
        self.claims.insert(name.clone(), claim);
        self.save_claim(&name).await?;
        
        println!("✅ Added claim: {}", name);
        
        Ok(())
    }
    
    async fn remove_claim_cli(&mut self, name: String) -> Result<()> {
        if self.claims.remove(&name).is_some() {
            // Remove claim file
            let claim_file = self.storage_path.join("claims").join(format!("{}.json", name));
            if claim_file.exists() {
                fs::remove_file(claim_file).await?;
            }
            
            println!("✅ Removed claim: {}", name);
            
            // Warn about policies using this claim
            let affected_policies: Vec<_> = self.policies
                .iter()
                .filter(|entry| entry.value().claims.contains(&name))
                .map(|entry| entry.value().name.clone())
                .collect();
            
            if !affected_policies.is_empty() {
                println!("⚠️  Warning: The following policies reference this claim:");
                for policy in affected_policies {
                    println!("   - {}", policy);
                }
            }
        } else {
            println!("❌ Claim not found: {}", name);
        }
        
        Ok(())
    }
    
    async fn initialize_default_claims(&mut self) -> Result<()> {
        let default_claims = vec![
            ("admin", "Full administrative access"),
            ("read", "Read access to resources"),
            ("write", "Write access to resources"),
            ("execute", "Execute workflows and processes"),
            ("deploy", "Deploy to CIM instances"),
            ("manage_policies", "Create and modify policies"),
            ("manage_domains", "Configure domain settings"),
            ("view_logs", "View system logs and audit trails"),
        ];
        
        for (name, description) in default_claims {
            let claim = Claim {
                name: name.to_string(),
                description: description.to_string(),
                domain: None,
                permissions: Vec::new(),
            };
            
            self.claims.insert(name.to_string(), claim);
            self.save_claim(name).await?;
        }
        
        info!("Initialized {} default claims", self.claims.len());
        
        Ok(())
    }
    
    async fn save_policy(&self, id: &str) -> Result<()> {
        if let Some(policy) = self.policies.get(id) {
            let file_path = self.storage_path.join("policies").join(format!("{}.json", id));
            let content = serde_json::to_string_pretty(&*policy)?;
            fs::write(file_path, content).await?;
        }
        Ok(())
    }
    
    async fn save_claim(&self, name: &str) -> Result<()> {
        if let Some(claim) = self.claims.get(name) {
            let file_path = self.storage_path.join("claims").join(format!("{}.json", name));
            let content = serde_json::to_string_pretty(&*claim)?;
            fs::write(file_path, content).await?;
        }
        Ok(())
    }
    
    async fn load_all(&mut self) -> Result<()> {
        // Load policies
        let policies_dir = self.storage_path.join("policies");
        if policies_dir.exists() {
            let mut entries = fs::read_dir(&policies_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".json") {
                        let content = fs::read_to_string(entry.path()).await?;
                        if let Ok(policy) = serde_json::from_str::<Policy>(&content) {
                            self.policies.insert(policy.id.clone(), policy.clone());
                            // Load into engine
                            self.engine.load_policy(policy)?;
                        }
                    }
                }
            }
        }
        
        // Load claims
        let claims_dir = self.storage_path.join("claims");
        if claims_dir.exists() {
            let mut entries = fs::read_dir(&claims_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".json") {
                        let content = fs::read_to_string(entry.path()).await?;
                        if let Ok(claim) = serde_json::from_str::<Claim>(&content) {
                            self.claims.insert(claim.name.clone(), claim);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn format_condition(&self, condition: &RuleCondition) -> String {
        match condition {
            RuleCondition::Always => "Always".to_string(),
            RuleCondition::HasClaim(claim) => format!("Has claim: {}", claim),
            RuleCondition::HasAllClaims(claims) => format!("Has all claims: {}", claims.join(", ")),
            RuleCondition::HasAnyClaim(claims) => format!("Has any claim: {}", claims.join(", ")),
            RuleCondition::DomainIs(domain) => format!("Domain is: {}", domain),
            RuleCondition::EventType(event) => format!("Event type: {}", event),
            RuleCondition::Custom(expr) => format!("Custom: {}", expr),
        }
    }
    
    fn format_action(&self, action: &RuleAction) -> String {
        match action {
            RuleAction::Allow => "Allow".to_string(),
            RuleAction::Deny => "Deny".to_string(),
            RuleAction::RequireApproval => "Require approval".to_string(),
            RuleAction::Log => "Log event".to_string(),
            RuleAction::Transform(transform) => format!("Transform: {}", transform),
            RuleAction::Delegate(target) => format!("Delegate to: {}", target),
        }
    }
    
    /// Evaluate a policy decision
    pub async fn evaluate(
        &self,
        subject_id: String,
        subject_type: String,
        subject_claims: Vec<String>,
        resource_id: String,
        resource_type: String,
        resource_domain: String,
        action_name: String,
    ) -> Result<Decision> {
        use std::collections::HashSet;
        
        let context = EvaluationContext {
            subject: Subject {
                id: subject_id,
                subject_type,
                claims: subject_claims.into_iter().collect(),
                domains: HashSet::from([resource_domain.clone()]),
                attributes: HashMap::new(),
            },
            resource: Resource {
                id: resource_id,
                resource_type,
                domain: resource_domain,
                attributes: HashMap::new(),
            },
            action: Action {
                name: action_name.clone(),
                action_type: action_name,
                parameters: HashMap::new(),
            },
            metadata: HashMap::new(),
            event: None,
        };
        
        let result = self.engine.evaluate(context).await?;
        Ok(result.decision)
    }
    
    /// Load policies into the evaluation engine
    async fn sync_policies_to_engine(&self) -> Result<()> {
        for entry in self.policies.iter() {
            let policy = entry.value().clone();
            self.engine.load_policy(policy)?;
        }
        Ok(())
    }
    
    /// Check if a subject has a specific claim
    pub fn has_claim(&self, subject_claims: &[String], required_claim: &str) -> bool {
        subject_claims.contains(&required_claim.to_string())
    }
    
    /// Get all permissions for a set of claims
    pub fn get_permissions_for_claims(&self, claim_names: &[String]) -> Vec<String> {
        use std::collections::HashSet;
        let mut permissions = HashSet::new();
        
        for claim_name in claim_names {
            if let Some(claim) = self.claims.get(claim_name) {
                permissions.extend(claim.permissions.iter().cloned());
            }
        }
        
        permissions.into_iter().collect()
    }
    
    /// Create a new policy
    pub async fn create_policy(&mut self, policy: Policy) -> Result<()> {
        let policy_id = policy.id.clone();
        self.policies.insert(policy_id.clone(), policy.clone());
        self.engine.load_policy(policy.clone())?;
        self.save_policy(&policy_id).await?;
        Ok(())
    }
    
    /// Add a new claim
    pub async fn add_claim(
        &mut self,
        name: String,
        description: String,
        domain: Option<String>,
        permissions: Vec<String>,
    ) -> Result<()> {
        let claim = Claim {
            name: name.clone(),
            description,
            domain,
            permissions,
        };
        
        self.claims.insert(name.clone(), claim);
        self.save_claim(&name).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PolicySummary {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub enabled: bool,
    pub rule_count: usize,
    pub claim_count: usize,
}