//! Event sequencing for reliable ordering

use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::AggregateId;

/// Event sequencer ensures ordered delivery of events
#[derive(Resource)]
pub struct EventSequencer {
    /// Per-aggregate event buffers
    aggregate_buffers: Arc<RwLock<HashMap<AggregateId, AggregateBuffer>>>,

    /// Global event buffer for ordering
    global_buffer: Arc<RwLock<GlobalBuffer>>,

    /// Configuration
    config: SequencerConfig,
}

#[derive(Debug, Clone)]
pub struct SequencerConfig {
    /// Maximum events to buffer per aggregate
    pub max_buffer_size: usize,

    /// Maximum out-of-order tolerance
    pub max_sequence_gap: u64,

    /// Timeout for missing sequences
    pub sequence_timeout: std::time::Duration,
}

impl Default for SequencerConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1000,
            max_sequence_gap: 100,
            sequence_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Buffer for a single aggregate
struct AggregateBuffer {
    /// Expected next sequence
    next_sequence: u64,

    /// Buffered out-of-order events
    pending: BTreeMap<u64, BufferedEvent>,

    /// Last processed timestamp
    last_processed: std::time::Instant,
}

/// Global event buffer
struct GlobalBuffer {
    /// Expected next global sequence
    next_sequence: u64,

    /// Buffered events by sequence
    pending: BTreeMap<u64, BufferedEvent>,

    /// Ready events queue
    ready_queue: VecDeque<DomainEvent>,
}

#[derive(Clone)]
struct BufferedEvent {
    event: DomainEvent,
    sequence: u64,
    aggregate_sequence: u64,
    received_at: std::time::Instant,
}

impl EventSequencer {
    pub fn new(config: SequencerConfig) -> Self {
        Self {
            aggregate_buffers: Arc::new(RwLock::new(HashMap::new())),
            global_buffer: Arc::new(RwLock::new(GlobalBuffer {
                next_sequence: 1,
                pending: BTreeMap::new(),
                ready_queue: VecDeque::new(),
            })),
            config,
        }
    }

    /// Process an event with sequence numbers
    pub fn process_event(
        &self,
        event: DomainEvent,
        global_sequence: u64,
        aggregate_sequence: u64,
    ) -> Result<Vec<DomainEvent>, String> {
        // Process at aggregate level first
        let aggregate_ready = self.process_aggregate_sequence(
            event.aggregate_id(),
            &event,
            aggregate_sequence,
        )?;

        if !aggregate_ready {
            // Event is out of order for this aggregate
            return Ok(vec![]);
        }

        // Process at global level
        self.process_global_sequence(event, global_sequence)
    }

    fn process_aggregate_sequence(
        &self,
        aggregate_id: AggregateId,
        event: &DomainEvent,
        sequence: u64,
    ) -> Result<bool, String> {
        let mut buffers = self.aggregate_buffers.write()
            .map_err(|e| format!("Failed to acquire buffer lock: {}", e))?;

        let buffer = buffers.entry(aggregate_id).or_insert_with(|| AggregateBuffer {
            next_sequence: 1,
            pending: BTreeMap::new(),
            last_processed: std::time::Instant::now(),
        });

        if sequence == buffer.next_sequence {
            // This is the expected sequence
            buffer.next_sequence += 1;
            buffer.last_processed = std::time::Instant::now();
            Ok(true)
        } else if sequence < buffer.next_sequence {
            // Duplicate or old event
            warn!("Received old sequence {} (expected {})", sequence, buffer.next_sequence);
            Ok(false)
        } else {
            // Out of order - buffer it
            let gap = sequence - buffer.next_sequence;
            if gap > self.config.max_sequence_gap {
                error!("Sequence gap too large: {} (max {})", gap, self.config.max_sequence_gap);
                return Err("Sequence gap too large".to_string());
            }

            buffer.pending.insert(sequence, BufferedEvent {
                event: event.clone(),
                sequence,
                aggregate_sequence: sequence,
                received_at: std::time::Instant::now(),
            });

            Ok(false)
        }
    }

    fn process_global_sequence(
        &self,
        event: DomainEvent,
        sequence: u64,
    ) -> Result<Vec<DomainEvent>, String> {
        let mut buffer = self.global_buffer.write()
            .map_err(|e| format!("Failed to acquire global buffer lock: {}", e))?;

        let mut ready_events = Vec::new();

        if sequence == buffer.next_sequence {
            // This event is ready
            ready_events.push(event);
            buffer.next_sequence += 1;

            // Check if any buffered events are now ready
            while let Some((&seq, _)) = buffer.pending.first_key_value() {
                if seq == buffer.next_sequence {
                    if let Some(buffered) = buffer.pending.remove(&seq) {
                        ready_events.push(buffered.event);
                        buffer.next_sequence += 1;
                    }
                } else {
                    break;
                }
            }
        } else if sequence > buffer.next_sequence {
            // Buffer for later
            buffer.pending.insert(sequence, BufferedEvent {
                event,
                sequence,
                aggregate_sequence: 0, // Not used at global level
                received_at: std::time::Instant::now(),
            });
        }
        // Ignore if sequence < next_sequence (duplicate)

        Ok(ready_events)
    }

    /// Check for timed-out sequences and force progression
    pub fn check_timeouts(&self) -> Vec<DomainEvent> {
        let mut forced_events = Vec::new();

        // Check aggregate buffers
        if let Ok(mut buffers) = self.aggregate_buffers.write() {
            for (aggregate_id, buffer) in buffers.iter_mut() {
                if buffer.last_processed.elapsed() > self.config.sequence_timeout {
                    // Force progression
                    if let Some((&next_seq, _)) = buffer.pending.first_key_value() {
                        warn!(
                            "Forcing sequence progression for aggregate {:?} from {} to {}",
                            aggregate_id, buffer.next_sequence, next_seq
                        );
                        buffer.next_sequence = next_seq;
                    }
                }
            }
        }

        // Check global buffer
        if let Ok(mut buffer) = self.global_buffer.write() {
            let now = std::time::Instant::now();
            let timeout = self.config.sequence_timeout;

            // Find timed-out sequences
            let timed_out: Vec<_> = buffer.pending
                .iter()
                .filter(|(_, event)| now.duration_since(event.received_at) > timeout)
                .map(|(seq, _)| *seq)
                .collect();

            if !timed_out.is_empty() {
                warn!("Found {} timed-out sequences, forcing progression", timed_out.len());

                // Skip to the first timed-out sequence
                if let Some(&first_timeout) = timed_out.first() {
                    buffer.next_sequence = first_timeout;

                    // Process all events that are now ready
                    for seq in timed_out {
                        if let Some(buffered) = buffer.pending.remove(&seq) {
                            forced_events.push(buffered.event);
                        }
                    }
                }
            }
        }

        forced_events
    }

    /// Get statistics about buffered events
    pub fn get_stats(&self) -> SequencerStats {
        let aggregate_stats = self.aggregate_buffers.read()
            .ok()
            .map(|buffers| {
                buffers.iter()
                    .map(|(id, buffer)| (id.clone(), AggregateStats {
                        next_sequence: buffer.next_sequence,
                        pending_count: buffer.pending.len(),
                        oldest_pending: buffer.pending.first_key_value()
                            .map(|(seq, _)| *seq),
                    }))
                    .collect()
            })
            .unwrap_or_default();

        let global_stats = self.global_buffer.read()
            .ok()
            .map(|buffer| GlobalStats {
                next_sequence: buffer.next_sequence,
                pending_count: buffer.pending.len(),
                ready_count: buffer.ready_queue.len(),
            })
            .unwrap_or_default();

        SequencerStats {
            aggregate_stats,
            global_stats,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SequencerStats {
    pub aggregate_stats: HashMap<AggregateId, AggregateStats>,
    pub global_stats: GlobalStats,
}

#[derive(Debug, Clone)]
pub struct AggregateStats {
    pub next_sequence: u64,
    pub pending_count: usize,
    pub oldest_pending: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct GlobalStats {
    pub next_sequence: u64,
    pub pending_count: usize,
    pub ready_count: usize,
}

/// Plugin for event sequencing
pub struct EventSequencerPlugin;

impl Plugin for EventSequencerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EventSequencer::new(SequencerConfig::default()))
            .add_systems(Update, check_sequence_timeouts);
    }
}

/// System to check for sequence timeouts
fn check_sequence_timeouts(
    sequencer: Res<EventSequencer>,
    mut event_writer: EventWriter<crate::application::EventNotification>,
) {
    let forced_events = sequencer.check_timeouts();

    for event in forced_events {
        warn!("Force-processing timed-out event: {:?}", event.event_type());
        event_writer.write(crate::application::EventNotification { event });
    }
}
