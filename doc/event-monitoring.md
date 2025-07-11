# Event Monitoring System

The Alchemist Event Monitoring System provides comprehensive event tracking, filtering, and analysis capabilities for all events flowing through the system via NATS.

## Features

### Core Capabilities
- **Real-time Event Monitoring**: Subscribe to and monitor events as they flow through NATS
- **Event Persistence**: Store events in SQLite database with automatic indexing
- **Advanced Filtering**: Use DSL expressions to filter events by domain, type, severity, time, etc.
- **Event Statistics**: Track event counts, rates, and patterns
- **Alert System**: Define rules to trigger alerts based on event patterns
- **Export Functionality**: Export events to JSON, CSV, or YAML formats

### Event Structure
Events are captured with the following information:
- **ID**: Unique event identifier
- **Timestamp**: When the event occurred
- **Domain**: System domain (workflow, ai, policy, etc.)
- **Event Type**: Specific event type (started, completed, error, etc.)
- **Severity**: Debug, Info, Warning, Error, or Critical
- **Subject**: NATS subject the event was published to
- **Correlation ID**: For tracking related events
- **Payload**: Event-specific data
- **Metadata**: Additional key-value pairs

## Command Line Interface

### List Recent Events
```bash
# List last 20 events
alchemist event list

# Filter by domain
alchemist event list --domain workflow

# Filter by severity
alchemist event list --severity error

# Show more events
alchemist event list --count 50
```

### Watch Events in Real-Time
```bash
# Watch all events
alchemist event watch

# Watch with filter
alchemist event watch --filter "domain:workflow AND severity:error"

# Custom update interval
alchemist event watch --interval 5
```

### Query Historical Events
```bash
# Query with DSL
alchemist event query "domain:workflow AND type:started"

# Complex queries
alchemist event query "severity:error OR severity:critical"
alchemist event query "timestamp>2024-01-01 AND domain:ai"

# Output formats
alchemist event query "domain:policy" --format json
alchemist event query "type:error" --format yaml --limit 100
```

### Event Statistics
```bash
# Show basic stats
alchemist event stats

# Group by different fields
alchemist event stats --group-by domain
alchemist event stats --group-by type
alchemist event stats --group-by severity

# Different time windows
alchemist event stats --window 1h
alchemist event stats --window 24h
alchemist event stats --window 7d
```

### Export Events
```bash
# Export to JSON
alchemist event export json --output events.json

# Export with filter
alchemist event export csv --output errors.csv --filter "severity:error"

# Export time range
alchemist event export yaml --output daily.yaml \
  --start "2024-01-01T00:00:00Z" \
  --end "2024-01-02T00:00:00Z"
```

### Alert Management
```bash
# List alert rules
alchemist event alert list

# Add alert rule
alchemist event alert add "Critical Errors" \
  --filter "severity:critical" \
  --action log \
  --throttle 300

# Email alert
alchemist event alert add "Workflow Failures" \
  --filter "domain:workflow AND type:failed" \
  --action email \
  --target admin@example.com

# Webhook alert
alchemist event alert add "High Error Rate" \
  --filter "severity:error" \
  --action webhook \
  --target "https://hooks.slack.com/services/..."

# Remove alert
alchemist event alert remove <rule-id>

# Test alert
alchemist event alert test <rule-id>
```

## Filter DSL

The event filtering DSL supports various operators and fields:

### Basic Filters
- `domain:workflow` - Match specific domain
- `type:started` - Match specific event type
- `severity:error` - Match severity level (minimum)
- `correlation:abc123` - Match correlation ID

### Comparison Operators
- `timestamp>2024-01-01` - Events after date
- `timestamp<2024-12-31` - Events before date

### Logical Operators
- `domain:workflow AND type:started` - Both conditions
- `severity:error OR severity:critical` - Either condition
- `NOT domain:test` - Negation

### Regex Patterns
- `subject~alchemist.*workflow` - Match subject with regex
- `domain~work.*` - Match domain with regex

### Complex Examples
```
# All workflow errors in the last hour
domain:workflow AND severity:error AND timestamp>2024-01-01T12:00:00Z

# Critical events from AI or policy domains
(domain:ai OR domain:policy) AND severity:critical

# Non-info events with specific correlation
NOT severity:info AND correlation:abc-123
```

## Event Monitoring UI

Launch the graphical event monitor:
```bash
alchemist render event-monitor
```

Features:
- Real-time event stream visualization
- Interactive filter builder
- Event detail inspection
- Statistics dashboard
- Alert configuration

## Configuration

### Database Settings
Events are stored in SQLite database at `events.db`. Configuration options:
- **Retention Period**: Default 30 days
- **Maximum Events**: Default 1,000,000
- **Index Strategy**: Automatic indexes on timestamp, domain, type, severity

### Performance Tuning
- **Buffer Size**: Maximum 10,000 events in memory
- **Batch Size**: Process events in batches of 100
- **Statistics Update**: Every 60 seconds

## Integration with Policy Engine

The event monitoring system integrates with the policy engine to:
- Trigger policies based on event patterns
- Record policy execution as events
- Monitor policy performance and errors

Example policy trigger:
```yaml
trigger:
  event_pattern: "domain:workflow AND type:failed"
  conditions:
    - severity: error
    - correlation_id: exists
```

## Troubleshooting

### NATS Connection Issues
```
Event monitoring not available (NATS not connected)
```
Solution: Ensure NATS is running:
```bash
docker-compose up -d nats
```

### Database Errors
```
Failed to store event: database is locked
```
Solution: Check database permissions and disk space.

### High Memory Usage
If memory usage is high:
1. Reduce buffer size
2. Enable more aggressive cleanup
3. Export and archive old events

## Testing

Run the test script to generate sample events:
```bash
cargo run --example test_event_monitor
```

This will generate various test events across different domains and severities.