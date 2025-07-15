//! Agent discovery via NATS

use crate::registry::{AgentId, AgentCapabilities};
use async_nats::Client as NatsClient;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, warn, error};

/// Events that occur during agent discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryEvent {
    /// New agent discovered
    AgentDiscovered(AgentCapabilities),
    
    /// Agent updated its capabilities
    AgentUpdated(AgentCapabilities),
    
    /// Agent is leaving
    AgentLeaving(AgentId),
}

/// Agent discovery service using NATS
pub struct AgentDiscovery {
    nats_client: NatsClient,
    discovery_subject: String,
    event_sender: mpsc::Sender<DiscoveryEvent>,
}

impl AgentDiscovery {
    /// Create a new agent discovery service
    pub fn new(
        nats_client: NatsClient,
        discovery_subject: String,
        event_sender: mpsc::Sender<DiscoveryEvent>,
    ) -> Self {
        Self {
            nats_client,
            discovery_subject,
            event_sender,
        }
    }
    
    /// Start listening for agent discovery messages
    pub async fn start(&self) -> anyhow::Result<()> {
        let mut subscriber = self.nats_client.subscribe(&self.discovery_subject).await?;
        let event_sender = self.event_sender.clone();
        
        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                match serde_json::from_slice::<DiscoveryMessage>(&message.payload) {
                    Ok(discovery_msg) => {
                        let event = match discovery_msg {
                            DiscoveryMessage::Announce(agent) => {
                                info!("Agent announced: {} ({})", agent.name, agent.id);
                                DiscoveryEvent::AgentDiscovered(agent)
                            }
                            DiscoveryMessage::Update(agent) => {
                                info!("Agent updated: {} ({})", agent.name, agent.id);
                                DiscoveryEvent::AgentUpdated(agent)
                            }
                            DiscoveryMessage::Leave(agent_id) => {
                                info!("Agent leaving: {}", agent_id);
                                DiscoveryEvent::AgentLeaving(agent_id)
                            }
                        };
                        
                        if let Err(e) = event_sender.send(event).await {
                            error!("Failed to send discovery event: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse discovery message: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Announce an agent to the network
    pub async fn announce_agent(&self, agent: &AgentCapabilities) -> anyhow::Result<()> {
        let message = DiscoveryMessage::Announce(agent.clone());
        let payload = serde_json::to_vec(&message)?;
        
        self.nats_client
            .publish(&self.discovery_subject, payload.into())
            .await?;
        
        Ok(())
    }
    
    /// Update agent capabilities
    pub async fn update_agent(&self, agent: &AgentCapabilities) -> anyhow::Result<()> {
        let message = DiscoveryMessage::Update(agent.clone());
        let payload = serde_json::to_vec(&message)?;
        
        self.nats_client
            .publish(&self.discovery_subject, payload.into())
            .await?;
        
        Ok(())
    }
    
    /// Announce agent departure
    pub async fn announce_departure(&self, agent_id: &AgentId) -> anyhow::Result<()> {
        let message = DiscoveryMessage::Leave(agent_id.clone());
        let payload = serde_json::to_vec(&message)?;
        
        self.nats_client
            .publish(&self.discovery_subject, payload.into())
            .await?;
        
        Ok(())
    }
}

/// Discovery messages sent via NATS
#[derive(Debug, Clone, Serialize, Deserialize)]
enum DiscoveryMessage {
    /// Agent announcement
    Announce(AgentCapabilities),
    
    /// Agent capability update
    Update(AgentCapabilities),
    
    /// Agent departure
    Leave(AgentId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_agent_discovery() {
        let client = async_nats::connect("nats://localhost:4222")
            .await
            .expect("Failed to connect to NATS");
        
        let (tx, mut rx) = mpsc::channel(100);
        let discovery = AgentDiscovery::new(
            client.clone(),
            "agent.discovery".to_string(),
            tx,
        );
        
        // Start discovery
        discovery.start().await.unwrap();
        
        // Announce an agent
        let agent = AgentCapabilities::new(
            AgentId::from_str("test-agent"),
            "Test Agent".to_string(),
            vec!["test".to_string()],
            "agent.test".to_string(),
        );
        
        discovery.announce_agent(&agent).await.unwrap();
        
        // Wait for event
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            if let Some(event) = rx.recv().await {
                match event {
                    DiscoveryEvent::AgentDiscovered(discovered_agent) => {
                        assert_eq!(discovered_agent.id, agent.id);
                    }
                    _ => panic!("Unexpected event"),
                }
            }
        })
        .await
        .expect("Timeout waiting for discovery event");
    }
}