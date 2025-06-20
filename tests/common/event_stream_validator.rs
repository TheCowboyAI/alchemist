//! Event Stream Validator
//! 
//! Utility for validating event sequences in tests

use async_nats::jetstream;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ExpectedEvent {
    pub event_type: String,
    pub causation_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CapturedEvent {
    pub event_type: String,
    pub event_id: Uuid,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub sequence: u64,
    pub payload: serde_json::Value,
}

#[derive(Debug)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub event_count: usize,
    pub sequence_errors: Vec<String>,
    pub correlation_errors: Vec<String>,
    pub missing_events: Vec<String>,
    pub unexpected_events: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
    
    pub fn event_count(&self) -> usize {
        self.event_count
    }
}

pub struct EventStreamValidator {
    expected_events: Vec<ExpectedEvent>,
    captured_events: Vec<CapturedEvent>,
    jetstream: Option<jetstream::Context>,
}

impl EventStreamValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
            jetstream: None,
        }
    }
    
    pub async fn with_nats(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let client = async_nats::connect("nats://localhost:4222").await?;
        self.jetstream = Some(jetstream::new(client));
        Ok(self)
    }
    
    pub fn expect_sequence(mut self, events: Vec<&str>) -> Self {
        self.expected_events = events
            .into_iter()
            .map(|e| ExpectedEvent {
                event_type: e.to_string(),
                causation_id: None,
            })
            .collect();
        self
    }
    
    pub fn expect_correlation_chain(mut self, events: Vec<(&str, Option<&str>)>) -> Self {
        self.expected_events = events
            .into_iter()
            .map(|(event_type, causation)| ExpectedEvent {
                event_type: event_type.to_string(),
                causation_id: causation.map(|s| s.to_string()),
            })
            .collect();
        self
    }
    
    pub async fn capture_from_nats(
        &mut self,
        subject: &str,
        max_messages: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let jetstream = self.jetstream.as_ref()
            .ok_or("NATS not initialized")?;
        
        // Get stream by subject
        let stream_name = jetstream.stream_by_subject(subject).await?;
        let stream = jetstream.get_stream(&stream_name).await?;
        
        // Create consumer
        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                durable_name: Some(format!("validator-{}", Uuid::new_v4())),
                deliver_policy: jetstream::consumer::DeliverPolicy::All,
                ..Default::default()
            })
            .await?;
        
        // Fetch messages
        let mut messages = consumer
            .fetch()
            .max_messages(max_messages)
            .messages()
            .await?;
        
        self.captured_events.clear();
        let mut seq = 0;
        
        while let Some(msg) = messages.next().await {
            if let Ok(msg) = msg {
                if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&msg.payload) {
                    seq += 1;
                    
                    // Extract event info from payload
                    let event_type = payload["event_type"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    let event_id = payload["event_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .unwrap_or_else(Uuid::new_v4);
                    
                    let correlation_id = payload["correlation_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok());
                    
                    let causation_id = payload["causation_id"]
                        .as_str()
                        .and_then(|s| Uuid::parse_str(s).ok());
                    
                    self.captured_events.push(CapturedEvent {
                        event_type,
                        event_id,
                        correlation_id,
                        causation_id,
                        sequence: seq,
                        payload,
                    });
                }
            }
        }
        
        // Cleanup
        consumer.delete().await.ok();
        
        Ok(())
    }
    
    pub fn capture_manual(&mut self, events: Vec<CapturedEvent>) {
        self.captured_events = events;
    }
    
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport {
            is_valid: true,
            event_count: self.captured_events.len(),
            sequence_errors: Vec::new(),
            correlation_errors: Vec::new(),
            missing_events: Vec::new(),
            unexpected_events: Vec::new(),
        };
        
        // Check event count
        if self.expected_events.len() != self.captured_events.len() {
            report.is_valid = false;
            report.sequence_errors.push(format!(
                "Expected {} events, but captured {}",
                self.expected_events.len(),
                self.captured_events.len()
            ));
        }
        
        // Check event types match in order
        for (i, (expected, captured)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected.event_type != captured.event_type {
                report.is_valid = false;
                report.sequence_errors.push(format!(
                    "Event {} mismatch: expected '{}', got '{}'",
                    i + 1,
                    expected.event_type,
                    captured.event_type
                ));
            }
        }
        
        // Check for missing events
        let captured_types: Vec<_> = self.captured_events
            .iter()
            .map(|e| e.event_type.as_str())
            .collect();
        
        for expected in &self.expected_events {
            if !captured_types.contains(&expected.event_type.as_str()) {
                report.is_valid = false;
                report.missing_events.push(expected.event_type.clone());
            }
        }
        
        // Check for unexpected events
        let expected_types: Vec<_> = self.expected_events
            .iter()
            .map(|e| e.event_type.as_str())
            .collect();
        
        for captured in &self.captured_events {
            if !expected_types.contains(&captured.event_type.as_str()) {
                report.unexpected_events.push(captured.event_type.clone());
            }
        }
        
        // Validate correlation chains if specified
        let mut correlation_map: HashMap<Uuid, &CapturedEvent> = HashMap::new();
        
        for event in &self.captured_events {
            if let Some(corr_id) = event.correlation_id {
                correlation_map.insert(corr_id, event);
            }
        }
        
        for (expected, captured) in self.expected_events.iter()
            .zip(self.captured_events.iter())
        {
            if let Some(expected_causation) = &expected.causation_id {
                if captured.causation_id.is_none() {
                    report.is_valid = false;
                    report.correlation_errors.push(format!(
                        "Event '{}' missing expected causation_id",
                        captured.event_type
                    ));
                }
            }
        }
        
        report
    }
    
    pub fn print_report(&self, report: &ValidationReport) {
        println!("\n📊 Event Stream Validation Report");
        println!("================================");
        println!("Valid: {}", if report.is_valid { "✅" } else { "❌" });
        println!("Event Count: {}", report.event_count);
        
        if !report.sequence_errors.is_empty() {
            println!("\n❌ Sequence Errors:");
            for error in &report.sequence_errors {
                println!("  - {}", error);
            }
        }
        
        if !report.missing_events.is_empty() {
            println!("\n❌ Missing Events:");
            for event in &report.missing_events {
                println!("  - {}", event);
            }
        }
        
        if !report.unexpected_events.is_empty() {
            println!("\n⚠️  Unexpected Events:");
            for event in &report.unexpected_events {
                println!("  - {}", event);
            }
        }
        
        if !report.correlation_errors.is_empty() {
            println!("\n❌ Correlation Errors:");
            for error in &report.correlation_errors {
                println!("  - {}", error);
            }
        }
        
        if report.is_valid {
            println!("\n✅ All validations passed!");
        }
    }
} 