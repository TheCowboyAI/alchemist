//! Mock AI provider for testing

use super::*;
use crate::value_objects::analysis_result::{
    Recommendation, EffortLevel, RecommendedAction, AnalysisResult,
    Insight, Impact, Priority
};

/// Mock AI provider that returns predetermined responses
pub struct MockAIProvider {
    delay_ms: u64,
}

impl MockAIProvider {
    pub fn new() -> Self {
        Self { delay_ms: 100 }
    }
    
    pub fn with_delay(delay_ms: u64) -> Self {
        Self { delay_ms }
    }
}

#[async_trait]
impl GraphAnalysisProvider for MockAIProvider {
    async fn analyze_graph(
        &self,
        graph_data: GraphData,
        analysis_type: AnalysisCapability,
        _parameters: HashMap<String, Value>,
    ) -> AIProviderResult<AnalysisResult> {
        // Simulate processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        
        // Generate mock insights based on graph size
        let mut insights = vec![];
        
        if graph_data.nodes.len() > 5 {
            insights.push(Insight {
                id: uuid::Uuid::new_v4(),
                category: "complexity".to_string(),
                description: format!(
                    "Graph has {} nodes, which may indicate high complexity",
                    graph_data.nodes.len()
                ),
                evidence: vec![
                    format!("Node count: {}", graph_data.nodes.len()),
                    format!("Edge count: {}", graph_data.edges.len()),
                ],
                confidence: 0.6,
                impact: Impact::Medium,
            });
        }
        
        // Generate mock recommendations
        let recommendations = vec![
            Recommendation {
                id: uuid::Uuid::new_v4(),
                title: "Optimize Node Layout".to_string(),
                description: "Reorganize nodes for better visibility".to_string(),
                priority: Priority::High,
                expected_impact: "Reduce processing time by 30%".to_string(),
                effort_level: EffortLevel::Medium,
                actions: vec![
                    RecommendedAction {
                        id: uuid::Uuid::new_v4(),
                        action_type: "layout_optimization".to_string(),
                        target: "graph_nodes".to_string(),
                        description: "Apply force-directed layout algorithm".to_string(),
                        estimated_duration: std::time::Duration::from_secs(300),
                        parameters: HashMap::new(),
                        dependencies: Vec::new(),
                    }
                ],
                metadata: HashMap::new(),
            },
            Recommendation {
                id: uuid::Uuid::new_v4(),
                title: "Remove Redundant Edges".to_string(),
                description: "Identify and remove unnecessary connections".to_string(),
                priority: Priority::Medium,
                expected_impact: "Improve graph clarity by 20%".to_string(),
                effort_level: EffortLevel::Low,
                actions: vec![
                    RecommendedAction {
                        id: uuid::Uuid::new_v4(),
                        action_type: "edge_removal".to_string(),
                        target: "redundant_edges".to_string(),
                        description: "Remove edges that don't add value".to_string(),
                        estimated_duration: std::time::Duration::from_secs(180),
                        parameters: HashMap::new(),
                        dependencies: Vec::new(),
                    }
                ],
                metadata: HashMap::new(),
            }
        ];
        
        // Add performance insight
        insights.push(Insight {
            id: uuid::Uuid::new_v4(),
            category: "performance".to_string(),
            description: "Graph processing could be optimized".to_string(),
            evidence: vec!["High node count".to_string(), "Complex edge patterns".to_string()],
            confidence: 0.85,
            impact: Impact::High,
        });
        
        Ok(AnalysisResult {
            id: uuid::Uuid::new_v4(),
            confidence_score: 0.75,
            summary: format!("Mock analysis of graph with {} nodes and {} edges", 
                graph_data.nodes.len(), graph_data.edges.len()),
            recommendations,
            insights,
            metadata: HashMap::from([
                ("mock".to_string(), json!(true)),
                ("graph_id".to_string(), json!(graph_data.graph_id.to_string())),
                ("analysis_type".to_string(), json!(format!("{:?}", analysis_type))),
            ]),
            timestamp: std::time::SystemTime::now(),
        })
    }
    
    async fn suggest_transformations(
        &self,
        _graph_data: GraphData,
        optimization_goals: Vec<String>,
        _constraints: HashMap<String, Value>,
    ) -> AIProviderResult<Vec<TransformationSuggestion>> {
        // Simulate processing delay
        tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        
        let suggestions = optimization_goals.iter()
            .enumerate()
            .map(|(i, goal)| TransformationSuggestion {
                id: format!("MOCK-T{:03}", i + 1),
                suggestion_type: "optimization".to_string(),
                description: format!("Optimize graph for: {}", goal),
                rationale: format!("Mock analysis suggests this would improve {}", goal),
                expected_benefit: "20-30% improvement".to_string(),
                transformation_steps: vec![
                    json!({
                        "action": "reorganize",
                        "target": "workflow",
                        "goal": goal,
                    }),
                ],
                risk_assessment: Some(json!({
                    "risk_level": "low",
                    "mitigation": "Create backup before transformation",
                })),
            })
            .collect();
        
        Ok(suggestions)
    }
    
    fn supports_capability(&self, capability: &AnalysisCapability) -> bool {
        // Mock provider supports all capabilities for testing
        match capability {
            AnalysisCapability::GraphAnalysis => true,
            AnalysisCapability::WorkflowOptimization => true,
            AnalysisCapability::SemanticAnalysis => true,
            AnalysisCapability::PatternDetection => true,
            AnalysisCapability::TransformationSuggestion => true,
            AnalysisCapability::Custom(_) => true,
        }
    }
    
    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Mock AI Provider".to_string(),
            version: "1.0.0".to_string(),
            model: "mock-model-v1".to_string(),
            capabilities: vec![
                AnalysisCapability::GraphAnalysis,
                AnalysisCapability::WorkflowOptimization,
                AnalysisCapability::PatternDetection,
                AnalysisCapability::SemanticAnalysis,
                AnalysisCapability::TransformationSuggestion,
            ],
            rate_limits: Some(RateLimits {
                requests_per_minute: 1000,
                tokens_per_minute: 100000,
                concurrent_requests: 10,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_provider_analysis() {
        let provider = MockAIProvider::new();
        let graph_data = GraphData {
            graph_id: uuid::Uuid::new_v4(),
            nodes: vec![
                NodeData {
                    id: "node-1".to_string(),
                    node_type: "process".to_string(),
                    label: "Start".to_string(),
                    properties: HashMap::new(),
                    position: Some((0.0, 0.0, 0.0)),
                },
            ],
            edges: vec![],
            metadata: HashMap::new(),
        };
        
        let result = provider.analyze_graph(
            graph_data,
            AnalysisCapability::GraphAnalysis,
            HashMap::new(),
        ).await.unwrap();
        
        assert_eq!(result.confidence_score, 0.75);
        assert!(!result.metadata.is_empty());
    }
} 