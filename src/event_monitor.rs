//! Event filtering and monitoring system for Alchemist
//! 
//! This module provides comprehensive event monitoring, filtering, and analysis capabilities
//! for events flowing through the Alchemist system via NATS.

use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Duration};
use futures::StreamExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Row};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSeverity::Debug => write!(f, "DEBUG"),
            EventSeverity::Info => write!(f, "INFO"),
            EventSeverity::Warning => write!(f, "WARNING"),
            EventSeverity::Error => write!(f, "ERROR"),
            EventSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Monitored event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredEvent {
    /// Unique event ID
    pub id: Uuid,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Domain the event belongs to
    pub domain: String,
    /// Event type
    pub event_type: String,
    /// Event severity
    pub severity: EventSeverity,
    /// NATS subject
    pub subject: String,
    /// Correlation ID for tracking related events
    pub correlation_id: Option<String>,
    /// Event payload
    pub payload: serde_json::Value,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Event filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Filter by domains
    pub domains: Option<Vec<String>>,
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by severity levels (minimum)
    pub min_severity: Option<EventSeverity>,
    /// Filter by time range
    pub time_range: Option<TimeRange>,
    /// Filter by correlation ID
    pub correlation_id: Option<String>,
    /// Filter by subject pattern (regex)
    pub subject_pattern: Option<String>,
    /// Custom metadata filters
    pub metadata_filters: HashMap<String, serde_json::Value>,
}

/// Time range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Filter expression AST for DSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterExpression {
    And(Box<FilterExpression>, Box<FilterExpression>),
    Or(Box<FilterExpression>, Box<FilterExpression>),
    Not(Box<FilterExpression>),
    Domain(String),
    EventType(String),
    Severity(EventSeverity),
    TimeAfter(DateTime<Utc>),
    TimeBefore(DateTime<Utc>),
    Correlation(String),
    Metadata(String, serde_json::Value),
    Regex(String, String), // field, pattern
}

/// Event statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStatistics {
    /// Total event count
    pub total_count: u64,
    /// Count by domain
    pub by_domain: HashMap<String, u64>,
    /// Count by event type
    pub by_type: HashMap<String, u64>,
    /// Count by severity
    pub by_severity: HashMap<EventSeverity, u64>,
    /// Events per minute (last 60 minutes)
    pub events_per_minute: VecDeque<u64>,
    /// Average processing time
    pub avg_processing_time_ms: f64,
}

/// Event monitor that subscribes to NATS and filters events
pub struct EventMonitor {
    /// NATS client
    nats_client: Arc<async_nats::Client>,
    /// SQLite connection pool
    db_pool: SqlitePool,
    /// Active filters
    filters: Arc<RwLock<Vec<EventFilter>>>,
    /// Event statistics
    statistics: Arc<RwLock<EventStatistics>>,
    /// Alert rules
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    /// Event buffer for real-time monitoring
    event_buffer: Arc<Mutex<VecDeque<MonitoredEvent>>>,
    /// Maximum buffer size
    max_buffer_size: usize,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule ID
    pub id: Uuid,
    /// Rule name
    pub name: String,
    /// Filter expression
    pub filter: FilterExpression,
    /// Action to take when triggered
    pub action: AlertAction,
    /// Throttle duration
    pub throttle: Option<Duration>,
    /// Last triggered timestamp
    #[serde(skip)]
    pub last_triggered: Option<DateTime<Utc>>,
}

/// Alert action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    Log(EventSeverity),
    Email(String),
    Webhook(String),
    Command(String),
}

impl EventMonitor {
    /// Create a new event monitor
    pub async fn new(
        nats_client: Arc<async_nats::Client>,
        db_path: &str,
        max_buffer_size: usize,
    ) -> Result<Self> {
        // Create database connection pool
        let db_pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", db_path))
            .await
            .context("Failed to create database pool")?;

        // Initialize database schema
        Self::init_database(&db_pool).await?;

        Ok(Self {
            nats_client,
            db_pool,
            filters: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(EventStatistics {
                total_count: 0,
                by_domain: HashMap::new(),
                by_type: HashMap::new(),
                by_severity: HashMap::new(),
                events_per_minute: VecDeque::with_capacity(60),
                avg_processing_time_ms: 0.0,
            })),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            event_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(max_buffer_size))),
            max_buffer_size,
        })
    }

    /// Initialize database schema
    async fn init_database(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                domain TEXT NOT NULL,
                event_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                subject TEXT NOT NULL,
                correlation_id TEXT,
                payload TEXT NOT NULL,
                metadata TEXT NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_timestamp ON events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_domain ON events(domain);
            CREATE INDEX IF NOT EXISTS idx_event_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_severity ON events(severity);
            CREATE INDEX IF NOT EXISTS idx_correlation_id ON events(correlation_id);
            "#
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Start monitoring events
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting event monitoring...");

        // Subscribe to all events
        let mut subscriber = self.nats_client
            .subscribe("alchemist.>")
            .await
            .context("Failed to subscribe to events")?;

        // Start statistics updater
        let stats_handle = self.start_statistics_updater();

        // Start retention policy enforcer
        let retention_handle = self.start_retention_enforcer();

        // Process incoming events
        while let Some(msg) = subscriber.next().await {
            let start_time = std::time::Instant::now();

            // Parse event
            if let Ok(event) = self.parse_nats_message(&msg).await {
                // Store in database
                if let Err(e) = self.store_event(&event).await {
                    error!("Failed to store event: {}", e);
                }

                // Update statistics
                self.update_statistics(&event, start_time.elapsed().as_millis() as f64).await;

                // Check alert rules
                self.check_alert_rules(&event).await;

                // Add to buffer for real-time monitoring
                self.add_to_buffer(event).await;
            }
        }

        Ok(())
    }

    /// Parse NATS message into MonitoredEvent
    async fn parse_nats_message(&self, msg: &async_nats::Message) -> Result<MonitoredEvent> {
        let subject = msg.subject.to_string();
        let parts: Vec<&str> = subject.split('.').collect();
        
        let domain = parts.get(1).unwrap_or(&"unknown").to_string();
        let event_type = parts.get(2).unwrap_or(&"unknown").to_string();

        let payload: serde_json::Value = serde_json::from_slice(&msg.payload)
            .unwrap_or_else(|_| serde_json::json!({
                "raw": String::from_utf8_lossy(&msg.payload).to_string()
            }));

        let correlation_id = payload.get("correlation_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let severity = payload.get("severity")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok())
            .unwrap_or(EventSeverity::Info);

        Ok(MonitoredEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            domain,
            event_type,
            severity,
            subject,
            correlation_id,
            payload,
            metadata: HashMap::new(),
        })
    }

    /// Store event in database
    async fn store_event(&self, event: &MonitoredEvent) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO events (
                id, timestamp, domain, event_type, severity, 
                subject, correlation_id, payload, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event.id.to_string())
        .bind(event.timestamp.timestamp())
        .bind(&event.domain)
        .bind(&event.event_type)
        .bind(event.severity.to_string())
        .bind(&event.subject)
        .bind(&event.correlation_id)
        .bind(serde_json::to_string(&event.payload)?)
        .bind(serde_json::to_string(&event.metadata)?)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Update event statistics
    async fn update_statistics(&self, event: &MonitoredEvent, processing_time_ms: f64) {
        let mut stats = self.statistics.write().await;
        
        stats.total_count += 1;
        *stats.by_domain.entry(event.domain.clone()).or_insert(0) += 1;
        *stats.by_type.entry(event.event_type.clone()).or_insert(0) += 1;
        *stats.by_severity.entry(event.severity).or_insert(0) += 1;

        // Update average processing time
        let alpha = 0.1; // Exponential moving average factor
        stats.avg_processing_time_ms = 
            (1.0 - alpha) * stats.avg_processing_time_ms + alpha * processing_time_ms;
    }

    /// Check alert rules against event
    async fn check_alert_rules(&self, event: &MonitoredEvent) {
        let mut rules = self.alert_rules.write().await;
        
        for rule in rules.iter_mut() {
            if self.evaluate_filter_expression(&rule.filter, event) {
                // Check throttle
                if let Some(throttle) = rule.throttle {
                    if let Some(last) = rule.last_triggered {
                        if Utc::now() - last < throttle {
                            continue;
                        }
                    }
                }

                // Trigger alert
                rule.last_triggered = Some(Utc::now());
                self.trigger_alert(&rule.action, event).await;
            }
        }
    }

    /// Evaluate filter expression against event
    fn evaluate_filter_expression(&self, expr: &FilterExpression, event: &MonitoredEvent) -> bool {
        match expr {
            FilterExpression::And(left, right) => {
                self.evaluate_filter_expression(left, event) && 
                self.evaluate_filter_expression(right, event)
            }
            FilterExpression::Or(left, right) => {
                self.evaluate_filter_expression(left, event) || 
                self.evaluate_filter_expression(right, event)
            }
            FilterExpression::Not(inner) => {
                !self.evaluate_filter_expression(inner, event)
            }
            FilterExpression::Domain(domain) => event.domain == *domain,
            FilterExpression::EventType(event_type) => event.event_type == *event_type,
            FilterExpression::Severity(severity) => event.severity >= *severity,
            FilterExpression::TimeAfter(time) => event.timestamp >= *time,
            FilterExpression::TimeBefore(time) => event.timestamp <= *time,
            FilterExpression::Correlation(id) => {
                event.correlation_id.as_ref() == Some(id)
            }
            FilterExpression::Metadata(key, value) => {
                event.metadata.get(key) == Some(value)
            }
            FilterExpression::Regex(field, pattern) => {
                if let Ok(re) = Regex::new(pattern) {
                    match field.as_str() {
                        "domain" => re.is_match(&event.domain),
                        "type" => re.is_match(&event.event_type),
                        "subject" => re.is_match(&event.subject),
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Trigger alert action
    async fn trigger_alert(&self, action: &AlertAction, event: &MonitoredEvent) {
        match action {
            AlertAction::Log(severity) => {
                match severity {
                    EventSeverity::Debug => debug!("Alert: {:?}", event),
                    EventSeverity::Info => info!("Alert: {:?}", event),
                    EventSeverity::Warning => warn!("Alert: {:?}", event),
                    EventSeverity::Error => error!("Alert: {:?}", event),
                    EventSeverity::Critical => error!("CRITICAL Alert: {:?}", event),
                }
            }
            AlertAction::Email(address) => {
                // TODO: Implement email sending
                info!("Would send email to {} for event: {:?}", address, event);
            }
            AlertAction::Webhook(url) => {
                // TODO: Implement webhook calling
                info!("Would call webhook {} for event: {:?}", url, event);
            }
            AlertAction::Command(cmd) => {
                // TODO: Implement command execution
                info!("Would execute command {} for event: {:?}", cmd, event);
            }
        }
    }

    /// Add event to buffer for real-time monitoring
    async fn add_to_buffer(&self, event: MonitoredEvent) {
        let mut buffer = self.event_buffer.lock().await;
        
        if buffer.len() >= self.max_buffer_size {
            buffer.pop_front();
        }
        
        buffer.push_back(event);
    }

    /// Query historical events
    pub async fn query_events(&self, filter: &EventFilter) -> Result<Vec<MonitoredEvent>> {
        let mut query = String::from(
            "SELECT id, timestamp, domain, event_type, severity, subject, correlation_id, payload, metadata 
             FROM events WHERE 1=1"
        );
        let mut bindings = Vec::new();

        // Build query based on filter
        if let Some(domains) = &filter.domains {
            query.push_str(" AND domain IN (");
            for (i, domain) in domains.iter().enumerate() {
                if i > 0 { query.push_str(", "); }
                query.push_str("?");
                bindings.push(domain.clone());
            }
            query.push_str(")");
        }

        if let Some(types) = &filter.event_types {
            query.push_str(" AND event_type IN (");
            for (i, event_type) in types.iter().enumerate() {
                if i > 0 { query.push_str(", "); }
                query.push_str("?");
                bindings.push(event_type.clone());
            }
            query.push_str(")");
        }

        if let Some(severity) = &filter.min_severity {
            query.push_str(" AND severity >= ?");
            bindings.push(severity.to_string());
        }

        if let Some(range) = &filter.time_range {
            query.push_str(" AND timestamp >= ? AND timestamp <= ?");
            bindings.push(range.start.timestamp().to_string());
            bindings.push(range.end.timestamp().to_string());
        }

        if let Some(corr_id) = &filter.correlation_id {
            query.push_str(" AND correlation_id = ?");
            bindings.push(corr_id.clone());
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT 1000");

        // Execute query
        let mut rows = sqlx::query(&query);
        for binding in bindings {
            rows = rows.bind(binding);
        }

        let results = rows
            .fetch_all(&self.db_pool)
            .await?;

        // Convert rows to events
        let mut events = Vec::new();
        for row in results {
            let event = MonitoredEvent {
                id: Uuid::parse_str(row.get::<String, _>("id").as_str())?,
                timestamp: DateTime::from_timestamp(row.get::<i64, _>("timestamp"), 0)
                    .unwrap_or_else(|| Utc::now()),
                domain: row.get("domain"),
                event_type: row.get("event_type"),
                severity: serde_json::from_value(
                    serde_json::Value::String(row.get::<String, _>("severity"))
                )?,
                subject: row.get("subject"),
                correlation_id: row.get("correlation_id"),
                payload: serde_json::from_str(&row.get::<String, _>("payload"))?,
                metadata: serde_json::from_str(&row.get::<String, _>("metadata"))?,
            };
            events.push(event);
        }

        Ok(events)
    }

    /// Get event statistics
    pub async fn get_statistics(&self) -> EventStatistics {
        self.statistics.read().await.clone()
    }

    /// Export events to file
    pub async fn export_events(
        &self,
        filter: &EventFilter,
        format: ExportFormat,
        output_path: &str,
    ) -> Result<()> {
        let events = self.query_events(filter).await?;

        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&events)?;
                tokio::fs::write(output_path, json).await?;
            }
            ExportFormat::Csv => {
                let mut wtr = csv::Writer::from_path(output_path)?;
                
                // Write header
                wtr.write_record(&[
                    "id", "timestamp", "domain", "event_type", "severity",
                    "subject", "correlation_id", "payload"
                ])?;

                // Write records
                for event in events {
                    wtr.write_record(&[
                        event.id.to_string(),
                        event.timestamp.to_rfc3339(),
                        event.domain,
                        event.event_type,
                        event.severity.to_string(),
                        event.subject,
                        event.correlation_id.unwrap_or_default(),
                        serde_json::to_string(&event.payload)?,
                    ])?;
                }

                wtr.flush()?;
            }
            ExportFormat::Yaml => {
                let yaml = serde_yaml::to_string(&events)?;
                tokio::fs::write(output_path, yaml).await?;
            }
        }

        Ok(())
    }

    /// Start statistics updater task
    fn start_statistics_updater(&self) -> tokio::task::JoinHandle<()> {
        let stats = self.statistics.clone();
        
        tokio::spawn(async move {
            let mut ticker = interval(tokio::time::Duration::from_secs(60));
            let mut last_count = 0u64;

            loop {
                ticker.tick().await;
                
                let mut stats = stats.write().await;
                let current_count = stats.total_count;
                let events_this_minute = current_count - last_count;
                
                if stats.events_per_minute.len() >= 60 {
                    stats.events_per_minute.pop_front();
                }
                stats.events_per_minute.push_back(events_this_minute);
                
                last_count = current_count;
            }
        })
    }

    /// Start retention policy enforcer
    fn start_retention_enforcer(&self) -> tokio::task::JoinHandle<()> {
        let pool = self.db_pool.clone();
        
        tokio::spawn(async move {
            let mut ticker = interval(tokio::time::Duration::from_secs(3600)); // Every hour

            loop {
                ticker.tick().await;
                
                // Delete events older than 30 days
                let cutoff = Utc::now() - Duration::days(30);
                
                if let Err(e) = sqlx::query("DELETE FROM events WHERE timestamp < ?")
                    .bind(cutoff.timestamp())
                    .execute(&pool)
                    .await
                {
                    error!("Failed to enforce retention policy: {}", e);
                }
            }
        })
    }
}

/// Export format options
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
    Yaml,
}

impl std::str::FromStr for ExportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "yaml" | "yml" => Ok(ExportFormat::Yaml),
            _ => Err(anyhow::anyhow!("Invalid export format: {}", s)),
        }
    }
}

/// Parse filter DSL expressions
pub fn parse_filter_dsl(input: &str) -> Result<FilterExpression> {
    // Simple parser for filter DSL
    // Examples:
    // - domain:workflow AND type:started
    // - severity:error OR severity:warning
    // - timestamp>2024-01-01 AND domain:ai
    
    let input = input.trim();
    
    // Handle AND/OR operators
    if let Some(pos) = input.find(" AND ") {
        let left = parse_filter_dsl(&input[..pos])?;
        let right = parse_filter_dsl(&input[pos + 5..])?;
        return Ok(FilterExpression::And(Box::new(left), Box::new(right)));
    }
    
    if let Some(pos) = input.find(" OR ") {
        let left = parse_filter_dsl(&input[..pos])?;
        let right = parse_filter_dsl(&input[pos + 4..])?;
        return Ok(FilterExpression::Or(Box::new(left), Box::new(right)));
    }
    
    // Handle NOT operator
    if input.starts_with("NOT ") {
        let inner = parse_filter_dsl(&input[4..])?;
        return Ok(FilterExpression::Not(Box::new(inner)));
    }
    
    // Handle basic conditions
    if let Some(pos) = input.find(':') {
        let field = &input[..pos];
        let value = &input[pos + 1..];
        
        match field {
            "domain" => Ok(FilterExpression::Domain(value.to_string())),
            "type" => Ok(FilterExpression::EventType(value.to_string())),
            "severity" => {
                let severity = match value.to_lowercase().as_str() {
                    "debug" => EventSeverity::Debug,
                    "info" => EventSeverity::Info,
                    "warning" => EventSeverity::Warning,
                    "error" => EventSeverity::Error,
                    "critical" => EventSeverity::Critical,
                    _ => return Err(anyhow::anyhow!("Invalid severity: {}", value)),
                };
                Ok(FilterExpression::Severity(severity))
            }
            "correlation" => Ok(FilterExpression::Correlation(value.to_string())),
            _ => {
                // Assume it's a metadata field
                let json_value = serde_json::Value::String(value.to_string());
                Ok(FilterExpression::Metadata(field.to_string(), json_value))
            }
        }
    } else if let Some(pos) = input.find('>') {
        let field = &input[..pos];
        let value = &input[pos + 1..];
        
        if field == "timestamp" {
            let dt = DateTime::parse_from_rfc3339(value)
                .or_else(|_| DateTime::parse_from_str(value, "%Y-%m-%d"))
                .map(|dt| dt.with_timezone(&Utc))?;
            Ok(FilterExpression::TimeAfter(dt))
        } else {
            Err(anyhow::anyhow!("Invalid filter expression: {}", input))
        }
    } else if let Some(pos) = input.find('<') {
        let field = &input[..pos];
        let value = &input[pos + 1..];
        
        if field == "timestamp" {
            let dt = DateTime::parse_from_rfc3339(value)
                .or_else(|_| DateTime::parse_from_str(value, "%Y-%m-%d"))
                .map(|dt| dt.with_timezone(&Utc))?;
            Ok(FilterExpression::TimeBefore(dt))
        } else {
            Err(anyhow::anyhow!("Invalid filter expression: {}", input))
        }
    } else if let Some(pos) = input.find('~') {
        let field = &input[..pos];
        let pattern = &input[pos + 1..];
        Ok(FilterExpression::Regex(field.to_string(), pattern.to_string()))
    } else {
        Err(anyhow::anyhow!("Invalid filter expression: {}", input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filter_dsl() {
        // Test simple domain filter
        let expr = parse_filter_dsl("domain:workflow").unwrap();
        match expr {
            FilterExpression::Domain(d) => assert_eq!(d, "workflow"),
            _ => panic!("Expected Domain filter"),
        }

        // Test AND expression
        let expr = parse_filter_dsl("domain:workflow AND type:started").unwrap();
        match expr {
            FilterExpression::And(left, right) => {
                match (*left, *right) {
                    (FilterExpression::Domain(d), FilterExpression::EventType(t)) => {
                        assert_eq!(d, "workflow");
                        assert_eq!(t, "started");
                    }
                    _ => panic!("Expected Domain AND EventType"),
                }
            }
            _ => panic!("Expected AND expression"),
        }

        // Test regex filter
        let expr = parse_filter_dsl("subject~alchemist.*workflow").unwrap();
        match expr {
            FilterExpression::Regex(field, pattern) => {
                assert_eq!(field, "subject");
                assert_eq!(pattern, "alchemist.*workflow");
            }
            _ => panic!("Expected Regex filter"),
        }
    }

    #[test]
    fn test_event_severity_ordering() {
        assert!(EventSeverity::Debug < EventSeverity::Info);
        assert!(EventSeverity::Info < EventSeverity::Warning);
        assert!(EventSeverity::Warning < EventSeverity::Error);
        assert!(EventSeverity::Error < EventSeverity::Critical);
    }
}