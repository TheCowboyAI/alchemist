//! Performance benchmarks for Alchemist
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use alchemist::{
    renderer_events::*,
    dashboard::DashboardData,
    policy_engine::{PolicyEngine, Policy, PolicyRule, Condition, Action},
    event_monitor::MonitoredEvent,
};
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark event creation and serialization
fn benchmark_event_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_creation");
    
    group.bench_function("create_dashboard_update", |b| {
        b.iter(|| {
            let data = DashboardData::example();
            let event = EventBuilder::dashboard_update(black_box(data));
            black_box(event);
        });
    });
    
    group.bench_function("create_dialog_events", |b| {
        b.iter(|| {
            let dialog_id = "test123".to_string();
            let message = "Hello world".to_string();
            
            let events = vec![
                EventBuilder::dialog_started(black_box(dialog_id.clone())),
                EventBuilder::user_message(black_box(dialog_id.clone()), black_box(message.clone())),
                EventBuilder::ai_thinking(black_box(dialog_id.clone())),
                EventBuilder::ai_response_chunk(black_box(dialog_id.clone()), black_box("Response".to_string())),
                EventBuilder::ai_response_complete(black_box(dialog_id)),
            ];
            black_box(events);
        });
    });
    
    group.bench_function("serialize_event", |b| {
        let event = EventBuilder::dashboard_update(DashboardData::example());
        b.iter(|| {
            let json = serde_json::to_string(&black_box(&event)).unwrap();
            black_box(json);
        });
    });
    
    group.finish();
}

/// Benchmark policy engine evaluation
fn benchmark_policy_engine(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("policy_engine");
    
    // Create test policies
    let policies = vec![
        Policy {
            id: "policy1".to_string(),
            name: "Read Policy".to_string(),
            domain: "graph".to_string(),
            rules: vec![
                PolicyRule {
                    conditions: vec![
                        Condition::HasClaim("graph:read".to_string()),
                        Condition::DomainIs("graph".to_string()),
                    ],
                    actions: vec![Action::Allow],
                },
            ],
            priority: 1,
            enabled: true,
        },
        Policy {
            id: "policy2".to_string(),
            name: "Write Policy".to_string(),
            domain: "graph".to_string(),
            rules: vec![
                PolicyRule {
                    conditions: vec![
                        Condition::HasClaim("graph:write".to_string()),
                        Condition::DomainIs("graph".to_string()),
                    ],
                    actions: vec![Action::Allow],
                },
            ],
            priority: 2,
            enabled: true,
        },
    ];
    
    group.bench_function("evaluate_single_policy", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = PolicyEngine::new(vec![], Duration::from_secs(300));
            let claims = vec!["graph:read".to_string()];
            let context = serde_json::json!({ "domain": "graph" });
            
            let result = engine.evaluate(&claims, &context).await.unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("evaluate_multiple_policies", |b| {
        b.to_async(&rt).iter(|| async {
            let engine = PolicyEngine::new(black_box(policies.clone()), Duration::from_secs(300));
            let claims = vec!["graph:read".to_string(), "graph:write".to_string()];
            let context = serde_json::json!({ "domain": "graph", "action": "update" });
            
            let result = engine.evaluate(&claims, &context).await.unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("policy_cache_hit", |b| {
        let engine = PolicyEngine::new(policies.clone(), Duration::from_secs(300));
        let claims = vec!["graph:read".to_string()];
        let context = serde_json::json!({ "domain": "graph" });
        
        // Warm up cache
        rt.block_on(async {
            engine.evaluate(&claims, &context).await.unwrap();
        });
        
        b.to_async(&rt).iter(|| async {
            let result = engine.evaluate(&black_box(&claims), &black_box(&context)).await.unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

/// Benchmark dashboard data operations
fn benchmark_dashboard_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("dashboard");
    
    group.bench_function("create_dashboard_data", |b| {
        b.iter(|| {
            let data = DashboardData::example();
            black_box(data);
        });
    });
    
    group.bench_function("update_dashboard_metrics", |b| {
        let mut data = DashboardData::example();
        b.iter(|| {
            data.total_events += 1;
            data.system_status.memory_usage_mb = 1024;
            data.system_status.cpu_percentage = 45.5;
            data.recent_events.push(EventInfo {
                timestamp: chrono::Utc::now().to_rfc3339(),
                domain: "test".to_string(),
                event_type: "benchmark".to_string(),
                summary: "Benchmark event".to_string(),
            });
            if data.recent_events.len() > 100 {
                data.recent_events.remove(0);
            }
            black_box(&data);
        });
    });
    
    group.finish();
}

/// Benchmark event monitoring
fn benchmark_event_monitoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_monitor");
    
    group.bench_function("create_monitored_event", |b| {
        b.iter(|| {
            let event = MonitoredEvent {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                domain: "benchmark".to_string(),
                event_type: "test".to_string(),
                severity: crate::event_monitor::EventSeverity::Info,
                source: "benchmarks".to_string(),
                data: serde_json::json!({ "test": true }),
                tags: vec!["benchmark".to_string()],
            };
            black_box(event);
        });
    });
    
    group.bench_function("filter_events", |b| {
        let events: Vec<MonitoredEvent> = (0..1000)
            .map(|i| MonitoredEvent {
                id: format!("event_{}", i),
                timestamp: chrono::Utc::now(),
                domain: if i % 2 == 0 { "graph" } else { "workflow" }.to_string(),
                event_type: if i % 3 == 0 { "created" } else { "updated" }.to_string(),
                severity: match i % 4 {
                    0 => crate::event_monitor::EventSeverity::Info,
                    1 => crate::event_monitor::EventSeverity::Warning,
                    2 => crate::event_monitor::EventSeverity::Error,
                    _ => crate::event_monitor::EventSeverity::Debug,
                },
                source: "benchmark".to_string(),
                data: serde_json::json!({ "index": i }),
                tags: vec![],
            })
            .collect();
        
        b.iter(|| {
            let filtered: Vec<_> = events
                .iter()
                .filter(|e| black_box(e.domain == "graph" && e.severity == crate::event_monitor::EventSeverity::Error))
                .collect();
            black_box(filtered);
        });
    });
    
    group.finish();
}

/// Benchmark string operations (for shell command parsing)
fn benchmark_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_ops");
    
    let commands = vec![
        "ai test --model gpt-4",
        "dialog new \"Complex Title with Spaces\" --model claude-3",
        "deploy pipeline \"release-v2\" -e dev -e staging -e prod --canary",
        "policy claims add graph:read --description \"Read access to graph domain\"",
    ];
    
    for (idx, cmd) in commands.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("parse_command", idx),
            cmd,
            |b, cmd| {
                b.iter(|| {
                    let parts: Vec<&str> = black_box(cmd).split_whitespace().collect();
                    let parsed = shellwords::split(black_box(cmd)).unwrap();
                    black_box((parts, parsed));
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark JSON operations
fn benchmark_json_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("json");
    
    let small_json = serde_json::json!({
        "id": "123",
        "name": "Test",
        "value": 42,
    });
    
    let medium_json = serde_json::json!({
        "data": (0..100).map(|i| {
            serde_json::json!({
                "id": i,
                "name": format!("Item {}", i),
                "values": vec![i, i * 2, i * 3],
            })
        }).collect::<Vec<_>>(),
    });
    
    let large_json = DashboardData::example();
    
    group.bench_function("serialize_small", |b| {
        b.iter(|| {
            let json_str = serde_json::to_string(&black_box(&small_json)).unwrap();
            black_box(json_str);
        });
    });
    
    group.bench_function("serialize_medium", |b| {
        b.iter(|| {
            let json_str = serde_json::to_string(&black_box(&medium_json)).unwrap();
            black_box(json_str);
        });
    });
    
    group.bench_function("serialize_large", |b| {
        b.iter(|| {
            let json_str = serde_json::to_string(&black_box(&large_json)).unwrap();
            black_box(json_str);
        });
    });
    
    let small_str = serde_json::to_string(&small_json).unwrap();
    let medium_str = serde_json::to_string(&medium_json).unwrap();
    let large_str = serde_json::to_string(&large_json).unwrap();
    
    group.bench_function("deserialize_small", |b| {
        b.iter(|| {
            let value: serde_json::Value = serde_json::from_str(&black_box(&small_str)).unwrap();
            black_box(value);
        });
    });
    
    group.bench_function("deserialize_medium", |b| {
        b.iter(|| {
            let value: serde_json::Value = serde_json::from_str(&black_box(&medium_str)).unwrap();
            black_box(value);
        });
    });
    
    group.bench_function("deserialize_large", |b| {
        b.iter(|| {
            let data: DashboardData = serde_json::from_str(&black_box(&large_str)).unwrap();
            black_box(data);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_event_creation,
    benchmark_policy_engine,
    benchmark_dashboard_operations,
    benchmark_event_monitoring,
    benchmark_string_operations,
    benchmark_json_operations
);
criterion_main!(benches);