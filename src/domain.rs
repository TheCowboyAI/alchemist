//! Domain hierarchy management and visualization

use anyhow::{Result, Context};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{info, warn, debug};

use crate::{
    config::{AlchemistConfig, DomainConfig, DomainRelationship},
    shell_commands::DomainCommands,
};

#[derive(Debug, Clone)]
pub struct Domain {
    pub name: String,
    pub description: String,
    pub module_path: String,
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

pub struct DomainManager {
    domains: DashMap<String, Domain>,
    relationships: Vec<DomainRelationship>,
}

impl DomainManager {
    pub async fn new(config: &AlchemistConfig) -> Result<Self> {
        let domains = DashMap::new();
        
        // Load domains from config
        for domain_config in &config.domains.available {
            let mut domain = Domain {
                name: domain_config.name.clone(),
                description: domain_config.description.clone(),
                module_path: domain_config.module_path.clone(),
                enabled: domain_config.enabled,
                dependencies: domain_config.dependencies.clone(),
                dependents: Vec::new(),
            };
            
            domains.insert(domain.name.clone(), domain);
        }
        
        // Calculate dependents
        let domain_names: Vec<String> = domains.iter().map(|e| e.key().clone()).collect();
        for name in &domain_names {
            if let Some(domain) = domains.get(name) {
                let deps = domain.dependencies.clone();
                drop(domain);
                
                for dep in deps {
                    if let Some(mut dep_domain) = domains.get_mut(&dep) {
                        dep_domain.dependents.push(name.clone());
                    }
                }
            }
        }
        
        Ok(Self {
            domains,
            relationships: config.domains.relationships.clone(),
        })
    }
    
    pub async fn handle_command(&mut self, command: DomainCommands) -> Result<()> {
        match command {
            DomainCommands::List => {
                self.list_domains_cli().await?;
            }
            DomainCommands::Tree { root } => {
                self.show_tree_cli(root).await?;
            }
            DomainCommands::Show { name } => {
                self.show_domain_cli(name).await?;
            }
            DomainCommands::Graph { format } => {
                self.show_graph_cli(format).await?;
            }
        }
        Ok(())
    }
    
    pub async fn list_domains(&self) -> Result<Vec<Domain>> {
        let mut domains: Vec<Domain> = self.domains
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        domains.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(domains)
    }
    
    pub async fn show_hierarchy(&self, root: Option<String>) -> Result<String> {
        let mut output = String::new();
        
        if let Some(root_name) = root {
            if let Some(domain) = self.domains.get(&root_name) {
                self.build_tree(&mut output, &domain, "", true, &mut HashSet::new());
            } else {
                return Err(anyhow::anyhow!("Domain not found: {}", root_name));
            }
        } else {
            // Show all root domains (those with no dependencies)
            let mut roots: Vec<_> = self.domains
                .iter()
                .filter(|entry| entry.value().dependencies.is_empty())
                .map(|entry| entry.value().clone())
                .collect();
            
            roots.sort_by(|a, b| a.name.cmp(&b.name));
            
            for (idx, domain) in roots.iter().enumerate() {
                let is_last = idx == roots.len() - 1;
                self.build_tree(&mut output, domain, "", is_last, &mut HashSet::new());
            }
        }
        
        Ok(output)
    }
    
    async fn list_domains_cli(&self) -> Result<()> {
        let domains = self.list_domains().await?;
        
        if domains.is_empty() {
            println!("No domains configured.");
            return Ok(());
        }
        
        println!("🔧 Available Domains:");
        println!("─────────────────────────────");
        
        for domain in domains {
            let status = if domain.enabled { "✅" } else { "⏸️" };
            
            println!("{} {} - {}", status, domain.name, domain.description);
            println!("   Module: {}", domain.module_path);
            
            if !domain.dependencies.is_empty() {
                println!("   Dependencies: {}", domain.dependencies.join(", "));
            }
            
            if !domain.dependents.is_empty() {
                println!("   Used by: {}", domain.dependents.join(", "));
            }
        }
        
        // Show relationships
        if !self.relationships.is_empty() {
            println!("\n🔗 Relationships:");
            for rel in &self.relationships {
                let arrow = if rel.bidirectional { "↔️" } else { "→" };
                println!("   {} {} {} ({})", 
                    rel.source, 
                    arrow, 
                    rel.target, 
                    rel.relationship_type
                );
            }
        }
        
        Ok(())
    }
    
    async fn show_tree_cli(&self, root: Option<String>) -> Result<()> {
        let tree = self.show_hierarchy(root).await?;
        
        println!("🌳 Domain Hierarchy:");
        println!("─────────────────────────────");
        println!("{}", tree);
        
        Ok(())
    }
    
    async fn show_domain_cli(&self, name: String) -> Result<()> {
        let domain = self.domains.get(&name)
            .ok_or_else(|| anyhow::anyhow!("Domain not found: {}", name))?;
        
        println!("📦 Domain: {}", domain.name);
        println!("─────────────────────────────");
        println!("Description: {}", domain.description);
        println!("Module Path: {}", domain.module_path);
        println!("Status: {}", if domain.enabled { "Enabled" } else { "Disabled" });
        
        if !domain.dependencies.is_empty() {
            println!("\n⬇️  Dependencies:");
            for dep in &domain.dependencies {
                if let Some(dep_domain) = self.domains.get(dep) {
                    let status = if dep_domain.enabled { "✅" } else { "⚠️" };
                    println!("   {} {} - {}", status, dep, dep_domain.description);
                }
            }
        }
        
        if !domain.dependents.is_empty() {
            println!("\n⬆️  Dependents:");
            for dep in &domain.dependents {
                if let Some(dep_domain) = self.domains.get(dep) {
                    let status = if dep_domain.enabled { "✅" } else { "⚠️" };
                    println!("   {} {} - {}", status, dep, dep_domain.description);
                }
            }
        }
        
        // Show relationships involving this domain
        let relationships: Vec<_> = self.relationships
            .iter()
            .filter(|r| r.source == name || r.target == name)
            .collect();
        
        if !relationships.is_empty() {
            println!("\n🔗 Relationships:");
            for rel in relationships {
                if rel.source == name {
                    println!("   → {} ({})", rel.target, rel.relationship_type);
                } else {
                    println!("   ← {} ({})", rel.source, rel.relationship_type);
                }
            }
        }
        
        Ok(())
    }
    
    async fn show_graph_cli(&self, format: String) -> Result<()> {
        match format.as_str() {
            "mermaid" => {
                println!("```mermaid");
                println!("graph TD");
                
                // Add nodes
                for entry in self.domains.iter() {
                    let domain = entry.value();
                    let style = if domain.enabled {
                        "fill:#90EE90"
                    } else {
                        "fill:#FFB6C1"
                    };
                    
                    println!("    {}[\"{}\\n{}\"]", 
                        domain.name, 
                        domain.name,
                        domain.description
                    );
                    println!("    style {} {}", domain.name, style);
                }
                
                // Add edges from dependencies
                for entry in self.domains.iter() {
                    let domain = entry.value();
                    for dep in &domain.dependencies {
                        println!("    {} --> {}", dep, domain.name);
                    }
                }
                
                // Add relationship edges
                for rel in &self.relationships {
                    let arrow = if rel.bidirectional { "<-->" } else { "-->" };
                    println!("    {} {}|{}| {}", 
                        rel.source, 
                        arrow,
                        rel.relationship_type,
                        rel.target
                    );
                }
                
                println!("```");
            }
            "dot" => {
                println!("digraph Domains {{");
                println!("    rankdir=BT;");
                println!("    node [shape=box, style=rounded];");
                
                // Add nodes
                for entry in self.domains.iter() {
                    let domain = entry.value();
                    let color = if domain.enabled { "lightgreen" } else { "lightpink" };
                    
                    println!("    \"{}\" [label=\"{}\\n{}\", fillcolor={}, style=filled];",
                        domain.name,
                        domain.name,
                        domain.description,
                        color
                    );
                }
                
                // Add edges
                for entry in self.domains.iter() {
                    let domain = entry.value();
                    for dep in &domain.dependencies {
                        println!("    \"{}\" -> \"{}\";", dep, domain.name);
                    }
                }
                
                // Add relationship edges
                for rel in &self.relationships {
                    let attrs = format!("label=\"{}\", style=dashed", rel.relationship_type);
                    if rel.bidirectional {
                        println!("    \"{}\" -> \"{}\" [{}];", rel.source, rel.target, attrs);
                        println!("    \"{}\" -> \"{}\" [{}];", rel.target, rel.source, attrs);
                    } else {
                        println!("    \"{}\" -> \"{}\" [{}];", rel.source, rel.target, attrs);
                    }
                }
                
                println!("}}");
            }
            "json" => {
                let graph_data = GraphData {
                    domains: self.domains
                        .iter()
                        .map(|entry| {
                            let d = entry.value();
                            GraphDomain {
                                name: d.name.clone(),
                                description: d.description.clone(),
                                enabled: d.enabled,
                                dependencies: d.dependencies.clone(),
                                dependents: d.dependents.clone(),
                            }
                        })
                        .collect(),
                    relationships: self.relationships.clone(),
                };
                
                println!("{}", serde_json::to_string_pretty(&graph_data)?);
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported format: {}. Use 'mermaid', 'dot', or 'json'", format));
            }
        }
        
        Ok(())
    }
    
    fn build_tree(
        &self,
        output: &mut String,
        domain: &Domain,
        prefix: &str,
        is_last: bool,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(&domain.name) {
            output.push_str(&format!("{}{}[circular: {}]\n", prefix, if is_last { "└── " } else { "├── " }, domain.name));
            return;
        }
        
        visited.insert(domain.name.clone());
        
        let status = if domain.enabled { "✅" } else { "⏸️" };
        let connector = if is_last { "└── " } else { "├── " };
        
        output.push_str(&format!("{}{}{} {} - {}\n", 
            prefix, 
            connector,
            status,
            domain.name, 
            domain.description
        ));
        
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        
        let mut deps: Vec<_> = domain.dependents
            .iter()
            .filter_map(|name| self.domains.get(name).map(|d| d.clone()))
            .collect();
        
        deps.sort_by(|a, b| a.name.cmp(&b.name));
        
        for (idx, dep) in deps.iter().enumerate() {
            let is_last_dep = idx == deps.len() - 1;
            self.build_tree(output, &dep, &new_prefix, is_last_dep, visited);
        }
        
        visited.remove(&domain.name);
    }
}

#[derive(Debug, Serialize)]
struct GraphData {
    domains: Vec<GraphDomain>,
    relationships: Vec<DomainRelationship>,
}

#[derive(Debug, Serialize)]
struct GraphDomain {
    name: String,
    description: String,
    enabled: bool,
    dependencies: Vec<String>,
    dependents: Vec<String>,
}