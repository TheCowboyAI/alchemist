//! CIM Event Monitor
//! 
//! Subscribes to NATS event streams and exposes Prometheus metrics

use prometheus::{
    register_counter_vec, register_histogram_vec, register_gauge_vec,
    CounterVec, HistogramVec, GaugeVec, TextEncoder, Encoder,
};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::Filter;
use futures::StreamExt;

/// Metrics exposed by the event monitor
#[derive(Clone)]
struct EventMetrics {
    /// Total events by type and domain
    events_total: CounterVec,
    /// Failed events by type and domain
    events_failed: CounterVec,
    /// Event processing latency
    event_latency: HistogramVec,
    /// Current event processing lag
    event_lag: GaugeVec,
    /// Domain errors by domain and error type
    domain_errors: CounterVec,
    /// Active event streams
    active_streams: GaugeVec,
}

impl EventMetrics {
    fn new() -> Result<Self, prometheus::Error> {
        Ok(Self {
            events_total: register_counter_vec!(
                "cim_events_total",
                "Total events processed",
                &["event_type", "domain", "aggregate_type"]
            )?,
            events_failed: register_counter_vec!(
                "cim_events_failed_total",
                "Total events that failed processing",
                &["event_type", "domain", "error_type"]
            )?,
            event_latency: register_histogram_vec!(
                "cim_event_processing_duration_seconds",
                "Event processing latency",
                &["event_type", "domain"],
                vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0]
            )?,
            event_lag: register_gauge_vec!(
                "cim_event_processing_lag_seconds",
                "Current event processing lag",
                &["domain", "stream"]
            )?,
            domain_errors: register_counter_vec!(
                "cim_domain_errors_total",
                "Domain errors by type",
                &["domain", "error_type", "aggregate_type"]
            )?,
            active_streams: register_gauge_vec!(
                "cim_active_event_streams",
                "Number of active event stream subscriptions",
                &["stream_type"]
            )?,
        })
    }

    /// Update metrics based on event
    fn process_event(&self, subject: &str, payload: &[u8]) {
        // Parse subject to extract domain and event type
        let parts: Vec<&str> = subject.split('.').collect();
        if parts.len() < 3 {
            return;
        }

        let domain = parts[1];
        let event_type = parts[2];

        // Try to parse event payload
        if let Ok(event) = serde_json::from_slice::<Value>(payload) {
            // Extract aggregate type if present
            let aggregate_type = event.get("aggregate_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // Update event counter
            self.events_total
                .with_label_values(&[event_type, domain, aggregate_type])
                .inc();

            // Check for error events
            if event_type.contains("error") || event_type.contains("failed") {
                let error_type = event.get("error_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                self.domain_errors
                    .with_label_values(&[domain, error_type, aggregate_type])
                    .inc();
            }

            // Calculate processing lag if timestamp is present
            if let Some(timestamp) = event.get("timestamp")
                .and_then(|v| v.as_i64()) {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                
                let lag = (now - timestamp).abs() as f64;
                self.event_lag
                    .with_label_values(&[domain, "default"])
                    .set(lag);
            }
        }
    }
}

/// Main event monitor service
pub async fn run_event_monitor() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize metrics
    let metrics = EventMetrics::new()?;

    // Connect to NATS
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let client = async_nats::connect(&nats_url).await?;

    // Get metrics port
    let metrics_port: u16 = std::env::var("METRICS_PORT")
        .unwrap_or_else(|_| "9091".to_string())
        .parse()?;

    // Subscribe to event subjects
    let subjects = std::env::var("EVENT_SUBJECTS").unwrap_or_else(|_| "cim.>".to_string());
    println!("Event monitor started on port {}", metrics_port);
    println!("Subscribing to: {}", &subjects);
    
    let mut subscriber = client.subscribe(subjects).await?;

    // Track active subscription
    metrics.active_streams
        .with_label_values(&["cim_events"])
        .set(1.0);

    // Start metrics HTTP server
    let metrics_for_server = metrics.clone();
    tokio::spawn(async move {
        let metrics_route = warp::path("metrics")
            .map(move || {
                let encoder = TextEncoder::new();
                let metric_families = prometheus::gather();
                let mut buffer = Vec::new();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                String::from_utf8(buffer).unwrap()
            });

        warp::serve(metrics_route)
            .run(([0, 0, 0, 0], metrics_port))
            .await;
    });

    // Process events
    while let Some(msg) = subscriber.next().await {
        metrics.process_event(&msg.subject, &msg.payload);
    }

    Ok(())
}

/// Binary entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_event_monitor().await
} 