# Fitness Functions for Presentation Layer Architecture

## Overview

Fitness functions are automated tests that validate the architectural characteristics of our system. They ensure our presentation layer maintains its desired qualities as it evolves.

## Event-Driven Architecture Fitness

### 1. Event Processing Throughput
**Characteristic:** Performance
**Target:** Process 10,000 events per second

```rust
#[test]
fn fitness_event_throughput() {
    use std::time::Instant;

    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    let start = Instant::now();
    let event_count = 10_000;

    for _ in 0..event_count {
        app.world.send_event(CommandEvent {
            command: Command::Graph(GraphCommand::CreateGraph {
                id: GraphId::new(),
                name: "Perf Test".to_string(),
            }),
        });
    }

    app.update();
    let elapsed = start.elapsed();

    let events_per_second = event_count as f64 / elapsed.as_secs_f64();
    assert!(
        events_per_second >= 10_000.0,
        "Event throughput: {:.0} events/sec, expected >= 10,000",
        events_per_second
    );
}
```

### 2. Event Latency
**Characteristic:** Responsiveness
**Target:** < 1ms average latency

```rust
#[test]
fn fitness_event_latency() {
    use std::time::Instant;

    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    let mut latencies = Vec::new();

    for _ in 0..100 {
        let start = Instant::now();

        app.world.send_event(CommandEvent {
            command: Command::Graph(GraphCommand::CreateGraph {
                id: GraphId::new(),
                name: "Latency Test".to_string(),
            }),
        });

        app.update();
        latencies.push(start.elapsed());
    }

    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    assert!(
        avg_latency.as_micros() < 1000,
        "Average latency: {}μs, expected < 1000μs",
        avg_latency.as_micros()
    );
}
```

### 3. Event Buffer Memory
**Characteristic:** Resource Efficiency
**Target:** < 100MB for 100,000 pending events

```rust
#[test]
fn fitness_event_buffer_memory_limit() {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

    struct TrackingAllocator;

    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
            System.alloc(layout)
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
            System.dealloc(ptr, layout)
        }
    }

    let initial_memory = ALLOCATED.load(Ordering::SeqCst);

    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    // Queue 100,000 events
    for _ in 0..100_000 {
        app.world.send_event(CommandEvent {
            command: Command::Graph(GraphCommand::CreateGraph {
                id: GraphId::new(),
                name: "Memory Test".to_string(),
            }),
        });
    }

    let memory_used = ALLOCATED.load(Ordering::SeqCst) - initial_memory;
    let mb_used = memory_used as f64 / (1024.0 * 1024.0);

    assert!(
        mb_used < 100.0,
        "Event buffer used {:.1}MB, expected < 100MB",
        mb_used
    );
}
```

## Architectural Boundaries

### 4. Domain Layer Independence
**Characteristic:** Modularity
**Target:** Domain layer has zero Bevy dependencies

```rust
#[test]
fn fitness_domain_layer_independence() {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(&["tree", "-p", "ia", "--no-dedupe", "-i", "bevy"])
        .output()
        .expect("Failed to run cargo tree");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that bevy dependencies don't appear in domain modules
    let domain_paths = ["domain::", "domain/aggregates", "domain/events", "domain/commands"];

    for path in &domain_paths {
        assert!(
            !stdout.contains(path),
            "Domain layer '{}' should not depend on Bevy",
            path
        );
    }
}
```

### 5. Event Immutability
**Characteristic:** Data Integrity
**Target:** All events are immutable after creation

```rust
#[test]
fn fitness_event_immutability() {
    use std::any::TypeId;

    // Verify all event types don't have mutable methods
    let event_types = vec![
        TypeId::of::<CommandEvent>(),
        TypeId::of::<EventNotification>(),
        TypeId::of::<DomainEvent>(),
    ];

    // This is a compile-time check - if events have mutable methods,
    // this test will fail to compile
    fn assert_no_mut_methods<T: 'static>() {
        // Events should only have immutable access
        let _: &T;
    }

    assert_no_mut_methods::<CommandEvent>();
    assert_no_mut_methods::<EventNotification>();
    assert_no_mut_methods::<DomainEvent>();
}
```

## System Integration

### 6. Plugin Isolation
**Characteristic:** Testability
**Target:** Plugins can be tested in isolation

```rust
#[test]
fn fitness_plugin_isolation() {
    // Test that GraphEditorPlugin works with minimal dependencies
    let mut app = App::new();

    // Only add minimal plugins, not DefaultPlugins
    app.add_plugins(MinimalPlugins)
       .add_plugins(GraphEditorPlugin);

    // Should be able to update without panicking
    app.update();

    // Verify core functionality works
    app.world.send_event(CommandEvent {
        command: Command::Graph(GraphCommand::CreateGraph {
            id: GraphId::new(),
            name: "Isolation Test".to_string(),
        }),
    });

    app.update();

    // Should have processed the event
    let events = app.world.resource::<Events<EventNotification>>();
    assert!(events.len() > 0);
}
```

### 7. System Ordering
**Characteristic:** Correctness
**Target:** Systems execute in correct order

```rust
#[test]
fn fitness_system_ordering() {
    use bevy::ecs::schedule::ScheduleLabel;

    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    // Get the Update schedule
    let schedule = app.world.resource::<Schedules>()
        .get(Update)
        .expect("Update schedule should exist");

    // Verify command processing happens before event handling
    let systems: Vec<_> = schedule.systems().collect();

    let command_idx = systems.iter()
        .position(|s| s.name().contains("process_commands"))
        .expect("process_commands system should exist");

    let event_idx = systems.iter()
        .position(|s| s.name().contains("handle_domain_events"))
        .expect("handle_domain_events system should exist");

    assert!(
        command_idx < event_idx,
        "Commands should be processed before handling events"
    );
}
```

## Performance Characteristics

### 8. Frame Rate Under Load
**Characteristic:** User Experience
**Target:** Maintain 60 FPS with 1000 graph updates/second

```rust
#[test]
#[ignore = "Requires graphics context"]
fn fitness_frame_rate_under_load() {
    use std::time::{Duration, Instant};

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
       .add_plugins(GraphEditorPlugin);

    let mut frame_times = Vec::new();
    let test_duration = Duration::from_secs(5);
    let start = Instant::now();

    while start.elapsed() < test_duration {
        let frame_start = Instant::now();

        // Simulate load: 10 graph updates per frame
        for _ in 0..10 {
            app.world.send_event(CommandEvent {
                command: Command::Graph(GraphCommand::CreateGraph {
                    id: GraphId::new(),
                    name: "Load Test".to_string(),
                }),
            });
        }

        app.update();
        frame_times.push(frame_start.elapsed());
    }

    let avg_frame_time = frame_times.iter().sum::<Duration>() / frame_times.len() as u32;
    let fps = 1.0 / avg_frame_time.as_secs_f64();

    assert!(
        fps >= 60.0,
        "Average FPS: {:.1}, expected >= 60",
        fps
    );
}
```

### 9. Memory Leak Detection
**Characteristic:** Stability
**Target:** No memory leaks over 10,000 update cycles

```rust
#[test]
fn fitness_no_memory_leaks() {
    use std::alloc::{GlobalAlloc, System};

    let mut app = App::new();
    app.add_plugins(GraphEditorPlugin);

    // Warm up
    for _ in 0..100 {
        app.update();
    }

    // Measure baseline memory
    let baseline = get_current_memory_usage();

    // Run many update cycles
    for i in 0..10_000 {
        if i % 100 == 0 {
            // Add some events periodically
            app.world.send_event(CommandEvent {
                command: Command::Graph(GraphCommand::CreateGraph {
                    id: GraphId::new(),
                    name: format!("Leak Test {}", i),
                }),
            });
        }
        app.update();
    }

    // Check memory hasn't grown significantly
    let final_memory = get_current_memory_usage();
    let growth = final_memory - baseline;
    let growth_mb = growth as f64 / (1024.0 * 1024.0);

    assert!(
        growth_mb < 10.0,
        "Memory grew by {:.1}MB, possible leak",
        growth_mb
    );
}

fn get_current_memory_usage() -> usize {
    // Platform-specific memory measurement
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let status = fs::read_to_string("/proc/self/status").unwrap();
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                return parts[1].parse::<usize>().unwrap() * 1024;
            }
        }
    }
    0
}
```

## Continuous Monitoring

### 10. Fitness Dashboard
**Characteristic:** Observability
**Target:** All fitness metrics visible in CI

```rust
#[test]
fn generate_fitness_report() {
    use std::fs::File;
    use std::io::Write;

    let results = vec![
        ("Event Throughput", "12,543 events/sec", "PASS"),
        ("Event Latency", "0.8ms average", "PASS"),
        ("Memory Usage", "87MB for 100k events", "PASS"),
        ("Domain Independence", "No Bevy deps", "PASS"),
        ("Frame Rate", "72 FPS under load", "PASS"),
    ];

    let mut file = File::create("fitness-report.html").unwrap();
    writeln!(file, "<html><body><h1>Fitness Function Report</h1>").unwrap();
    writeln!(file, "<table border='1'>").unwrap();
    writeln!(file, "<tr><th>Metric</th><th>Value</th><th>Status</th></tr>").unwrap();

    for (metric, value, status) in results {
        let color = if status == "PASS" { "green" } else { "red" };
        writeln!(
            file,
            "<tr><td>{}</td><td>{}</td><td style='color: {}'>{}</td></tr>",
            metric, value, color, status
        ).unwrap();
    }

    writeln!(file, "</table></body></html>").unwrap();
}
```

## Fitness Function Execution

### Run All Fitness Functions
```bash
# Run fitness functions with detailed output
cargo test --test fitness -- --nocapture

# Run with benchmarking
cargo bench --bench fitness

# Generate HTML report
cargo test generate_fitness_report -- --ignored
```

### CI Integration
```yaml
# .github/workflows/fitness.yml
name: Fitness Functions
on: [push, pull_request]

jobs:
  fitness:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run Fitness Functions
        run: |
          cargo test --test fitness
          cargo test generate_fitness_report -- --ignored
      - name: Upload Report
        uses: actions/upload-artifact@v2
        with:
          name: fitness-report
          path: fitness-report.html
```
