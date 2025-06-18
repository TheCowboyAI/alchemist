//! Example demonstrating WorkflowGraph with a document approval process
//!
//! This example shows:
//! - Creating workflow graphs with the new domain model
//! - Adding steps with dependencies
//! - Exporting to JSON and DOT formats
//! - Validating workflow structure
//! - Using ContextGraph projection for visualization

use cim_domain_workflow::value_objects::StepType;
use cim_workflow_graph::{WorkflowGraph, WorkflowGraphError};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Document Approval Workflow Example ===\n");

    // Create workflow graph
    let mut workflow = WorkflowGraph::new(
        "Document Approval Process".to_string(),
        "Complete document approval workflow with review and publishing steps".to_string(),
    )?;

    println!("📝 Creating Document Approval Workflow...");
    println!("   Name: {}", workflow.name());
    println!("   Description: {}", workflow.description());
    println!("   Status: {:?}", workflow.status());

    // Add metadata
    workflow.add_tag("approval".to_string());
    workflow.add_tag("document".to_string());
    workflow.set_property("department".to_string(), serde_json::json!("content"));
    workflow.set_property("priority".to_string(), serde_json::json!("high"));

    println!("\n🏗️  Adding workflow steps...");

    // Step 1: Draft Creation
    let draft_step = workflow.add_step(
        "Create Draft".to_string(),
        "Author creates the initial document draft".to_string(),
        StepType::Manual,
        {
            let mut config = HashMap::new();
            config.insert("template".to_string(), serde_json::json!("standard_doc"));
            config.insert("min_length".to_string(), serde_json::json!(500));
            config
        },
        Vec::new(), // No dependencies
        Some(120),  // 2 hours
        Some("content-author".to_string()),
    )?;

    println!("   ✓ Added: Create Draft (Manual, 2 hours)");

    // Step 2: Technical Review
    let tech_review_step = workflow.add_step(
        "Technical Review".to_string(),
        "Technical expert reviews document for accuracy".to_string(),
        StepType::Manual,
        {
            let mut config = HashMap::new();
            config.insert(
                "review_checklist".to_string(),
                serde_json::json!([
                    "Technical accuracy",
                    "Code examples work",
                    "Links are valid"
                ]),
            );
            config.insert("required_score".to_string(), serde_json::json!(8));
            config
        },
        vec![draft_step], // Depends on draft
        Some(60),         // 1 hour
        Some("tech-reviewer".to_string()),
    )?;

    println!("   ✓ Added: Technical Review (Manual, 1 hour)");

    // Step 3: Editorial Review
    let editorial_step = workflow.add_step(
        "Editorial Review".to_string(),
        "Editor reviews document for style and clarity".to_string(),
        StepType::Manual,
        {
            let mut config = HashMap::new();
            config.insert(
                "style_guide".to_string(),
                serde_json::json!("company_style_v2"),
            );
            config.insert("grammar_check".to_string(), serde_json::json!(true));
            config
        },
        vec![draft_step], // Also depends on draft (parallel with tech review)
        Some(45),         // 45 minutes
        Some("editor".to_string()),
    )?;

    println!("   ✓ Added: Editorial Review (Manual, 45 minutes)");

    // Step 4: Final Approval
    let approval_step = workflow.add_step(
        "Manager Approval".to_string(),
        "Department manager gives final approval for publication".to_string(),
        StepType::Approval,
        {
            let mut config = HashMap::new();
            config.insert(
                "approval_criteria".to_string(),
                serde_json::json!([
                    "Technical review passed",
                    "Editorial review passed",
                    "Aligns with business goals"
                ]),
            );
            config.insert("escalation_hours".to_string(), serde_json::json!(24));
            config
        },
        vec![tech_review_step, editorial_step], // Depends on both reviews
        Some(30),                               // 30 minutes
        Some("department-manager".to_string()),
    )?;

    println!("   ✓ Added: Manager Approval (Approval, 30 minutes)");

    // Step 5: Publication
    let publish_step = workflow.add_step(
        "Publish Document".to_string(),
        "Publish the approved document to the company portal".to_string(),
        StepType::Automated,
        {
            let mut config = HashMap::new();
            config.insert(
                "target_platform".to_string(),
                serde_json::json!("company_portal"),
            );
            config.insert(
                "notification_list".to_string(),
                serde_json::json!(["all-staff@company.com", "content-team@company.com"]),
            );
            config.insert("auto_index".to_string(), serde_json::json!(true));
            config
        },
        vec![approval_step], // Depends on approval
        Some(5),             // 5 minutes automated
        Some("publishing-system".to_string()),
    )?;

    println!("   ✓ Added: Publish Document (Automated, 5 minutes)");

    // Validate workflow
    println!("\n🔍 Validating workflow structure...");
    match workflow.validate() {
        Ok(()) => println!("   ✅ Workflow validation passed!"),
        Err(e) => {
            println!("   ❌ Workflow validation failed: {}", e);
            return Err(e.into());
        }
    }

    // Display workflow statistics
    let stats = workflow.statistics();
    println!("\n📊 Workflow Statistics:");
    println!("   • Total nodes: {}", stats.total_nodes);
    println!("   • Step nodes: {}", stats.step_nodes);
    println!("   • Total edges: {}", stats.total_edges);
    println!("   • Dependency edges: {}", stats.dependency_edges);
    println!("   • Max depth: {}", stats.max_depth);
    println!("   • Is cyclic: {}", stats.is_cyclic);

    // Show step analysis
    println!("\n📋 Step Analysis:");
    for node in workflow.get_step_nodes() {
        if let cim_workflow_graph::ContextGraphNodeValue::Step {
            name,
            step_type,
            status,
            estimated_duration_minutes,
            assigned_to,
            ..
        } = &node.value
        {
            println!("   • {} ({:?})", name, step_type);
            println!("     Status: {:?}", status);
            if let Some(duration) = estimated_duration_minutes {
                println!("     Duration: {} minutes", duration);
            }
            if let Some(assignee) = assigned_to {
                println!("     Assigned to: {}", assignee);
            }
        }
    }

    // Show dependency analysis
    println!("\n🔗 Dependency Analysis:");
    for edge in workflow.get_dependency_edges() {
        println!(
            "   • {} → {} ({})",
            edge.source, edge.target, edge.edge_type
        );
    }

    // Export to JSON
    println!("\n📄 Exporting to JSON format...");
    let json = workflow.to_json()?;

    // Show a snippet of the JSON
    let json_lines: Vec<&str> = json.lines().take(10).collect();
    println!("   JSON Preview (first 10 lines):");
    for line in json_lines {
        println!("   {}", line);
    }
    println!("   ... (truncated)");

    // Show JSON size
    println!("   📏 JSON size: {} bytes", json.len());

    // Export to DOT format
    println!("\n🎨 Exporting to DOT format (for Graphviz)...");
    let dot = workflow.to_dot();

    // Show DOT content
    println!("   DOT content:");
    for line in dot.lines().take(15) {
        println!("   {}", line);
    }
    if dot.lines().count() > 15 {
        println!("   ... (truncated)");
    }

    // Simulate workflow execution
    println!("\n🚀 Simulating workflow execution...");

    // Start the workflow
    let mut context = HashMap::new();
    context.insert("initiator".to_string(), serde_json::json!("john.doe"));
    context.insert(
        "document_type".to_string(),
        serde_json::json!("technical_guide"),
    );

    workflow.start(context)?;
    println!("   ✅ Workflow started successfully!");
    println!("   Status: {:?}", workflow.status());

    // Find executable steps
    let executable_steps = workflow.get_executable_steps();
    println!(
        "   📝 Executable steps: {} steps ready",
        executable_steps.len()
    );

    // Find steps by type
    let manual_steps = workflow.find_steps_by_type(StepType::Manual);
    let automated_steps = workflow.find_steps_by_type(StepType::Automated);
    let approval_steps = workflow.find_steps_by_type(StepType::Approval);

    println!("\n📊 Steps by Type:");
    println!("   • Manual steps: {}", manual_steps.len());
    println!("   • Automated steps: {}", automated_steps.len());
    println!("   • Approval steps: {}", approval_steps.len());

    // Calculate estimated total time
    let total_time: u32 = workflow
        .get_step_nodes()
        .iter()
        .filter_map(|node| {
            if let cim_workflow_graph::ContextGraphNodeValue::Step {
                estimated_duration_minutes: Some(duration),
                ..
            } = &node.value
            {
                Some(*duration)
            } else {
                None
            }
        })
        .sum();

    println!("\n⏱️  Time Analysis:");
    println!(
        "   • Total estimated time: {} minutes ({:.1} hours)",
        total_time,
        total_time as f32 / 60.0
    );

    // Show critical path (longest dependency chain)
    println!("   • Critical path: Draft → Reviews → Approval → Publishing");

    let critical_path_time = 120 + 60.max(45) + 30 + 5; // Draft + max(reviews) + approval + publish
    println!(
        "   • Critical path time: {} minutes ({:.1} hours)",
        critical_path_time,
        critical_path_time as f32 / 60.0
    );

    // Test JSON round-trip
    println!("\n🔄 Testing JSON round-trip...");
    let reconstructed = WorkflowGraph::from_json(&json)?;
    println!("   ✅ Successfully reconstructed ContextGraph from JSON");

    println!("\n🎯 Business Value Analysis:");
    println!("   • Parallel reviews save time (45 min vs 105 min sequential)");
    println!("   • Automated publishing reduces manual effort");
    println!("   • Clear approval gates ensure quality control");
    println!("   • Estimated ROI: 40% time reduction vs manual process");

    println!("\n✨ Document Approval Workflow Example Complete! ✨");
    println!("\nNext steps:");
    println!("   1. Save JSON to file for integration with other systems");
    println!("   2. Use DOT file with Graphviz: `dot -Tpng workflow.dot -o workflow.png`");
    println!("   3. Integrate with workflow execution engine");
    println!("   4. Add monitoring and metrics collection");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_approval_workflow() {
        // Run the main example as a test
        assert!(main().is_ok());
    }

    #[test]
    fn test_workflow_creation_and_validation() {
        let workflow = WorkflowGraph::new(
            "Test Workflow".to_string(),
            "Test workflow for validation".to_string(),
        )
        .unwrap();

        assert_eq!(workflow.name(), "Test Workflow");
        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_step_dependencies() {
        let mut workflow = WorkflowGraph::new(
            "Dependency Test".to_string(),
            "Testing step dependencies".to_string(),
        )
        .unwrap();

        let step1 = workflow
            .add_step(
                "Step 1".to_string(),
                "First step".to_string(),
                StepType::Manual,
                HashMap::new(),
                Vec::new(),
                Some(30),
                None,
            )
            .unwrap();

        let _step2 = workflow
            .add_step(
                "Step 2".to_string(),
                "Second step".to_string(),
                StepType::Automated,
                HashMap::new(),
                vec![step1],
                Some(15),
                None,
            )
            .unwrap();

        assert!(workflow.validate().is_ok());
        let stats = workflow.statistics();
        assert_eq!(stats.step_nodes, 2);
        assert!(stats.dependency_edges > 0);
    }
}
