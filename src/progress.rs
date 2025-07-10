//! Progress tracking and visualization

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(clap::ValueEnum, Clone)]
pub enum ProgressFormat {
    Tree,
    Json,
    Summary,
    Timeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub project: String,
    pub version: String,
    pub status: String,
    pub completion_percentage: u8,
    pub current_date: String,
    pub current_focus: String,
    pub summary: String,
    pub overall_completion: u8,
    pub last_updated: DateTime<Utc>,
    
    pub domains: HashMap<String, DomainProgress>,
    pub metrics: Metrics,
    pub recent_changes: Vec<RecentChange>,
    pub milestones_achieved: Vec<Milestone>,
    pub architecture_health: ArchitectureHealth,
    
    #[serde(default)]
    pub graph_structure: GraphStructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProgress {
    pub status: String,
    pub completion: u8,
    pub tests_passing: u32,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub total_tests: u32,
    pub passing_tests: u32,
    pub code_files: u32,
    pub documentation_files: u32,
    pub domains_complete: u32,
    pub domains_total: u32,
    pub event_types: u32,
    pub command_types: u32,
    pub query_types: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentChange {
    pub date: String,
    pub change: String,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub date: String,
    pub achievement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureHealth {
    pub event_sourcing: u8,
    pub cqrs_implementation: u8,
    pub domain_isolation: u8,
    pub test_coverage: u8,
    pub documentation: u8,
    pub cross_domain_integration: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphStructure {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

impl Progress {
    /// Load progress from JSON file
    pub async fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let progress: Progress = serde_json::from_str(&content)?;
        Ok(progress)
    }
    
    /// Get overall health status
    pub fn health_status(&self) -> &str {
        match self.overall_completion {
            90..=100 => "EXCELLENT",
            70..=89 => "GOOD",
            50..=69 => "FAIR",
            _ => "NEEDS ATTENTION",
        }
    }
    
    /// Get domain completion summary
    pub fn domain_summary(&self) -> Vec<(String, u8, &str)> {
        let mut summary = Vec::new();
        
        for (name, domain) in &self.domains {
            summary.push((
                name.clone(),
                domain.completion,
                domain.status.as_str(),
            ));
        }
        
        // Sort by completion percentage
        summary.sort_by(|a, b| b.1.cmp(&a.1));
        
        summary
    }
    
    /// Get recent achievements
    pub fn recent_achievements(&self, count: usize) -> Vec<&Milestone> {
        self.milestones_achieved
            .iter()
            .rev()
            .take(count)
            .collect()
    }
    
    /// Format as tree view
    pub fn format_tree(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("🎯 {} - v{}\n", self.project, self.version));
        output.push_str(&format!("   Status: {} ({}%)\n", self.status, self.completion_percentage));
        output.push_str(&format!("   Health: {}\n", self.health_status()));
        output.push_str(&format!("   Focus: {}\n\n", self.current_focus));
        
        output.push_str("📊 Domains:\n");
        for (name, completion, status) in self.domain_summary() {
            let progress_bar = create_progress_bar(completion);
            output.push_str(&format!(
                "   {} {} [{}] {}% - {}\n",
                if completion == 100 { "✅" } else { "🔄" },
                name,
                progress_bar,
                completion,
                status
            ));
        }
        
        output.push_str(&format!("\n📈 Metrics:\n"));
        output.push_str(&format!("   Tests: {}/{} passing\n", self.metrics.passing_tests, self.metrics.total_tests));
        output.push_str(&format!("   Domains: {}/{} complete\n", self.metrics.domains_complete, self.metrics.domains_total));
        output.push_str(&format!("   Events: {} types\n", self.metrics.event_types));
        
        output.push_str(&format!("\n🏆 Recent Achievements:\n"));
        for milestone in self.recent_achievements(5) {
            output.push_str(&format!("   • {} - {}\n", milestone.date, milestone.achievement));
        }
        
        output
    }
    
    /// Format as summary
    pub fn format_summary(&self) -> String {
        format!(
            "{} is {}% complete with {} domains finished. Current status: {}. {}",
            self.project,
            self.completion_percentage,
            self.metrics.domains_complete,
            self.status,
            self.summary
        )
    }
    
    /// Format as timeline
    pub fn format_timeline(&self) -> String {
        let mut output = String::new();
        
        output.push_str("📅 Project Timeline:\n\n");
        
        // Combine milestones and recent changes
        let mut timeline: Vec<(String, String, String)> = Vec::new();
        
        for milestone in &self.milestones_achieved {
            timeline.push((
                milestone.date.clone(),
                "🏆 Milestone".to_string(),
                milestone.achievement.clone(),
            ));
        }
        
        for change in self.recent_changes.iter().take(10) {
            timeline.push((
                change.date.clone(),
                "🔧 Change".to_string(),
                change.change.clone(),
            ));
        }
        
        // Sort by date
        timeline.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (date, event_type, description) in timeline {
            output.push_str(&format!("{} {} {}\n", date, event_type, description));
            
            // Add impact for changes
            if event_type.contains("Change") {
                if let Some(change) = self.recent_changes.iter().find(|c| c.change == description) {
                    output.push_str(&format!("          Impact: {}\n", change.impact));
                }
            }
            output.push_str("\n");
        }
        
        output
    }
}

fn create_progress_bar(percentage: u8) -> String {
    let filled = (percentage as usize / 10).min(10);
    let empty = 10 - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_progress_loading() {
        // Test would load actual progress.json
        // For now, just verify the structure compiles
        let _progress = Progress {
            project: "CIM".to_string(),
            version: "0.4.2".to_string(),
            status: "COMPLETE".to_string(),
            completion_percentage: 100,
            current_date: "2025-01-27".to_string(),
            current_focus: "Production readiness".to_string(),
            summary: "Test summary".to_string(),
            overall_completion: 48,
            last_updated: Utc::now(),
            domains: HashMap::new(),
            metrics: Metrics {
                total_tests: 468,
                passing_tests: 468,
                code_files: 450,
                documentation_files: 125,
                domains_complete: 14,
                domains_total: 14,
                event_types: 87,
                command_types: 64,
                query_types: 42,
            },
            recent_changes: vec![],
            milestones_achieved: vec![],
            architecture_health: ArchitectureHealth {
                event_sourcing: 100,
                cqrs_implementation: 100,
                domain_isolation: 100,
                test_coverage: 95,
                documentation: 90,
                cross_domain_integration: 100,
            },
            graph_structure: GraphStructure::default(),
        };
    }
}