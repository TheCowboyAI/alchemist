//! Main bridge service implementation

use async_nats::{Client, Subscriber};
use chrono::Utc;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

use crate::error::{BridgeError, Result};
use crate::providers::{ollama::OllamaProvider, Provider};
use crate::types::*;

/// Main bridge service - completely standalone
pub struct BridgeService {
    nats_client: Client,
    providers: Arc<RwLock<HashMap<String, Box<dyn Provider>>>>,
    active_provider: Arc<RwLock<Option<String>>>,
    metrics: Arc<RwLock<HashMap<String, ProviderMetrics>>>,
}

impl BridgeService {
    pub async fn new(nats_url: &str) -> Result<Self> {
        info!("Connecting to NATS at {}", nats_url);
        let nats_client = async_nats::connect(nats_url).await
            .map_err(|e| BridgeError::NatsConnect(e.to_string()))?;
        
        let mut providers: HashMap<String, Box<dyn Provider>> = HashMap::new();
        
        // Add Ollama provider by default
        let ollama = OllamaProvider::new(Default::default());
        providers.insert("ollama".to_string(), Box::new(ollama));
        
        Ok(Self {
            nats_client,
            providers: Arc::new(RwLock::new(providers)),
            active_provider: Arc::new(RwLock::new(Some("ollama".to_string()))),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Start the bridge service
    pub async fn start(&self) -> Result<()> {
        info!("Starting CIM Bridge service");
        
        // Subscribe to command subjects
        let command_sub = self.nats_client
            .subscribe("bridge.command.>")
            .await?;
        info!("Subscribed to bridge.command.*");
        
        // Subscribe to query subjects
        let query_sub = self.nats_client
            .subscribe("bridge.query.>")
            .await?;
        info!("Subscribed to bridge.query.*");
        
        // Process messages
        tokio::select! {
            result = self.handle_commands(command_sub) => {
                if let Err(e) = result {
                    error!("Command handler error: {}", e);
                }
            }
            result = self.handle_queries(query_sub) => {
                if let Err(e) = result {
                    error!("Query handler error: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_commands(&self, mut sub: Subscriber) -> Result<()> {
        info!("Command handler started");
        
        while let Some(msg) = sub.next().await {
            let subject = msg.subject.clone();
            info!("Received command on subject: {}", subject);
            
            match serde_json::from_slice::<CommandEnvelope>(&msg.payload) {
                Ok(envelope) => {
                    info!("Processing command: {:?}", envelope.command);
                    
                    if let Err(e) = self.process_command(envelope).await {
                        error!("Failed to process command: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to deserialize command: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_queries(&self, mut sub: Subscriber) -> Result<()> {
        info!("Query handler started");
        
        while let Some(msg) = sub.next().await {
            let subject = msg.subject.clone();
            info!("Received query on subject: {}", subject);
            
            match serde_json::from_slice::<QueryEnvelope>(&msg.payload) {
                Ok(envelope) => {
                    info!("Processing query: {:?}", envelope.query);
                    
                    let reply = msg.reply.map(|s| s.to_string());
                    if let Err(e) = self.process_query(envelope, reply).await {
                        error!("Failed to process query: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to deserialize query: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_command(&self, envelope: CommandEnvelope) -> Result<()> {
        match envelope.command {
            BridgeCommand::Query { model, messages, parameters } => {
                self.process_query_command(
                    envelope.id,
                    envelope.correlation_id,
                    model,
                    messages,
                    parameters,
                ).await?;
            }
            BridgeCommand::StreamQuery { model, messages, parameters } => {
                self.process_stream_query(
                    envelope.id,
                    envelope.correlation_id,
                    model,
                    messages,
                    parameters,
                ).await?;
            }
            BridgeCommand::SwitchProvider { provider, config } => {
                self.switch_provider(provider, config, envelope.correlation_id).await?;
            }
            BridgeCommand::ListProviders => {
                self.list_providers(envelope.correlation_id).await?;
            }
            BridgeCommand::ListModels { provider } => {
                self.list_models(provider, envelope.correlation_id).await?;
            }
            BridgeCommand::HealthCheck { provider } => {
                self.health_check(provider, envelope.correlation_id).await?;
            }
        }
        
        Ok(())
    }
    
    async fn process_query(&self, envelope: QueryEnvelope, reply: Option<String>) -> Result<()> {
        match envelope.query {
            BridgeQuery::Status => {
                // TODO: Implement status query
                info!("Status query received");
            }
            BridgeQuery::ActiveProvider => {
                let provider = self.active_provider.read().await.clone();
                if let Some(reply_subject) = reply {
                    let response = serde_json::json!({
                        "active_provider": provider
                    });
                    self.nats_client
                        .publish(reply_subject, response.to_string().into())
                        .await
                        .map_err(|e| BridgeError::NatsPublish(e.to_string()))?;
                }
            }
            BridgeQuery::Metrics { provider: _, time_range: _ } => {
                // TODO: Implement metrics query
                info!("Metrics query received");
            }
            BridgeQuery::Capabilities { provider: _ } => {
                // TODO: Implement capabilities query
                info!("Capabilities query received");
            }
        }
        
        Ok(())
    }
    
    async fn process_query_command(
        &self,
        query_id: Uuid,
        correlation_id: Uuid,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<()> {
        info!("Processing query for model: {}", model);
        
        // Get active provider
        let provider_name = self.get_active_provider().await?;
        
        // Publish query started event
        self.publish_event(BridgeEvent::QueryStarted {
            query_id,
            provider: provider_name.clone(),
            model: model.clone(),
        }, correlation_id).await?;
        
        // Get provider instance
        let providers = self.providers.read().await;
        let provider = providers
            .get(&provider_name)
            .ok_or(BridgeError::ProviderNotFound(provider_name.clone()))?;
        
        // Execute query
        let start_time = std::time::Instant::now();
        
        match provider.query(model, messages, parameters).await {
            Ok(response) => {
                info!("Query completed successfully");
                
                // Publish success event
                self.publish_event(BridgeEvent::QueryCompleted {
                    query_id,
                    response,
                }, correlation_id).await?;
                
                // Update metrics
                self.update_metrics(&provider_name, true, start_time.elapsed().as_millis() as f64).await?;
            }
            Err(e) => {
                error!("Query failed: {}", e);
                
                // Publish failure event
                self.publish_event(BridgeEvent::QueryFailed {
                    query_id,
                    error: e.to_string(),
                    provider: provider_name.clone(),
                }, correlation_id).await?;
                
                // Update metrics
                self.update_metrics(&provider_name, false, start_time.elapsed().as_millis() as f64).await?;
            }
        }
        
        Ok(())
    }
    
    async fn process_stream_query(
        &self,
        query_id: Uuid,
        correlation_id: Uuid,
        model: String,
        messages: Vec<Message>,
        parameters: ModelParameters,
    ) -> Result<()> {
        info!("Processing streaming query for model: {}", model);
        
        // Get active provider
        let provider_name = self.get_active_provider().await?;
        
        // Publish query started event
        self.publish_event(BridgeEvent::QueryStarted {
            query_id,
            provider: provider_name.clone(),
            model: model.clone(),
        }, correlation_id).await?;
        
        // Get provider instance
        let providers = self.providers.read().await;
        let provider = providers
            .get(&provider_name)
            .ok_or(BridgeError::ProviderNotFound(provider_name.clone()))?;
        
        // Execute streaming query
        let start_time = std::time::Instant::now();
        
        match provider.stream_query(model, messages, parameters).await {
            Ok(mut stream) => {
                info!("Stream started successfully");
                
                // Spawn task to handle stream
                let nats_client = self.nats_client.clone();
                let provider_name_clone = provider_name.clone();
                let metrics = self.metrics.clone();
                
                tokio::spawn(async move {
                    let mut sequence = 0u32;
                    
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                let event = BridgeEvent::StreamChunk {
                                    query_id,
                                    chunk,
                                    sequence,
                                };
                                
                                if let Err(e) = Self::publish_event_static(
                                    &nats_client,
                                    event,
                                    correlation_id,
                                ).await {
                                    error!("Failed to publish stream chunk: {}", e);
                                    break;
                                }
                                
                                sequence += 1;
                            }
                            Err(e) => {
                                error!("Stream error: {}", e);
                                
                                let event = BridgeEvent::QueryFailed {
                                    query_id,
                                    error: e.to_string(),
                                    provider: provider_name_clone.clone(),
                                };
                                
                                let _ = Self::publish_event_static(
                                    &nats_client,
                                    event,
                                    correlation_id,
                                ).await;
                                
                                break;
                            }
                        }
                    }
                    
                    // Update metrics
                    let elapsed = start_time.elapsed().as_millis() as f64;
                    let mut metrics_lock = metrics.write().await;
                    let provider_metrics = metrics_lock
                        .entry(provider_name_clone)
                        .or_insert_with(|| ProviderMetrics {
                            total_requests: 0,
                            successful_requests: 0,
                            failed_requests: 0,
                            average_latency_ms: 0.0,
                            tokens_processed: 0,
                            last_request: None,
                        });
                    
                    provider_metrics.total_requests += 1;
                    provider_metrics.successful_requests += 1;
                    provider_metrics.average_latency_ms = 
                        (provider_metrics.average_latency_ms * (provider_metrics.total_requests - 1) as f64 + elapsed) 
                        / provider_metrics.total_requests as f64;
                    provider_metrics.last_request = Some(Utc::now());
                });
            }
            Err(e) => {
                error!("Failed to start stream: {}", e);
                
                // Publish failure event
                self.publish_event(BridgeEvent::QueryFailed {
                    query_id,
                    error: e.to_string(),
                    provider: provider_name.clone(),
                }, correlation_id).await?;
                
                // Update metrics
                self.update_metrics(&provider_name, false, start_time.elapsed().as_millis() as f64).await?;
            }
        }
        
        Ok(())
    }
    
    async fn switch_provider(
        &self,
        provider: String,
        _config: Option<serde_json::Value>,
        correlation_id: Uuid,
    ) -> Result<()> {
        info!("Switching to provider: {}", provider);
        
        let providers = self.providers.read().await;
        if !providers.contains_key(&provider) {
            return Err(BridgeError::ProviderNotFound(provider));
        }
        
        let old_provider = self.active_provider.read().await.clone();
        *self.active_provider.write().await = Some(provider.clone());
        
        self.publish_event(BridgeEvent::ProviderSwitched {
            old_provider,
            new_provider: provider,
        }, correlation_id).await?;
        
        Ok(())
    }
    
    async fn list_providers(&self, correlation_id: Uuid) -> Result<()> {
        info!("Listing providers");
        
        let providers = self.providers.read().await;
        let active_provider = self.active_provider.read().await.clone();
        
        let provider_infos: Vec<ProviderInfo> = providers
            .iter()
            .map(|(name, provider)| ProviderInfo {
                name: name.clone(),
                description: provider.description(),
                is_active: Some(name) == active_provider.as_ref(),
                is_configured: provider.is_configured(),
                capabilities: provider.capabilities(),
            })
            .collect();
        
        self.publish_event(BridgeEvent::ProvidersListed {
            providers: provider_infos,
        }, correlation_id).await?;
        
        Ok(())
    }
    
    async fn list_models(&self, provider: Option<String>, correlation_id: Uuid) -> Result<()> {
        info!("Listing models for provider: {:?}", provider);
        
        let provider_name = match provider {
            Some(name) => name,
            None => self.get_active_provider().await?,
        };
        
        let providers = self.providers.read().await;
        let provider = providers
            .get(&provider_name)
            .ok_or(BridgeError::ProviderNotFound(provider_name.clone()))?;
        
        match provider.list_models().await {
            Ok(models) => {
                self.publish_event(BridgeEvent::ModelsListed {
                    provider: provider_name,
                    models,
                }, correlation_id).await?;
            }
            Err(e) => {
                error!("Failed to list models: {}", e);
                return Err(e);
            }
        }
        
        Ok(())
    }
    
    async fn health_check(&self, provider: Option<String>, correlation_id: Uuid) -> Result<()> {
        info!("Health check for provider: {:?}", provider);
        
        let provider_name = match provider {
            Some(name) => name,
            None => self.get_active_provider().await?,
        };
        
        let providers = self.providers.read().await;
        let provider = providers
            .get(&provider_name)
            .ok_or(BridgeError::ProviderNotFound(provider_name.clone()))?;
        
        match provider.health_check().await {
            Ok(status) => {
                let details = match &status {
                    HealthStatus::Healthy => Some("Provider is operational".to_string()),
                    HealthStatus::Degraded(msg) => Some(msg.clone()),
                    HealthStatus::Unhealthy(msg) => Some(msg.clone()),
                };
                
                self.publish_event(BridgeEvent::ProviderHealth {
                    provider: provider_name,
                    status,
                    details,
                }, correlation_id).await?;
            }
            Err(e) => {
                error!("Health check failed: {}", e);
                
                self.publish_event(BridgeEvent::ProviderHealth {
                    provider: provider_name,
                    status: HealthStatus::Unhealthy(e.to_string()),
                    details: Some(format!("Health check error: {e}")),
                }, correlation_id).await?;
            }
        }
        
        Ok(())
    }
    
    async fn get_active_provider(&self) -> Result<String> {
        self.active_provider
            .read()
            .await
            .clone()
            .ok_or(BridgeError::NoActiveProvider)
    }
    
    async fn update_metrics(
        &self,
        provider: &str,
        success: bool,
        latency_ms: f64,
    ) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let provider_metrics = metrics
            .entry(provider.to_string())
            .or_insert_with(|| ProviderMetrics {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_latency_ms: 0.0,
                tokens_processed: 0,
                last_request: None,
            });
        
        provider_metrics.total_requests += 1;
        if success {
            provider_metrics.successful_requests += 1;
        } else {
            provider_metrics.failed_requests += 1;
        }
        
        provider_metrics.average_latency_ms = 
            (provider_metrics.average_latency_ms * (provider_metrics.total_requests - 1) as f64 + latency_ms) 
            / provider_metrics.total_requests as f64;
        
        provider_metrics.last_request = Some(Utc::now());
        
        // Publish metrics update
        self.publish_event(BridgeEvent::MetricsUpdated {
            provider: provider.to_string(),
            metrics: provider_metrics.clone(),
        }, Uuid::new_v4()).await?;
        
        Ok(())
    }
    
    async fn publish_event(
        &self,
        event: BridgeEvent,
        correlation_id: Uuid,
    ) -> Result<()> {
        Self::publish_event_static(&self.nats_client, event, correlation_id).await
    }
    
    async fn publish_event_static(
        nats_client: &Client,
        event: BridgeEvent,
        correlation_id: Uuid,
    ) -> Result<()> {
        let envelope = EventEnvelope {
            id: Uuid::new_v4(),
            event: event.clone(),
            correlation_id,
            causation_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        let subject = match &event {
            BridgeEvent::QueryStarted { .. } => "bridge.event.query.started",
            BridgeEvent::QueryCompleted { .. } => "bridge.event.query.completed",
            BridgeEvent::QueryFailed { .. } => "bridge.event.query.failed",
            BridgeEvent::StreamChunk { .. } => "bridge.event.stream.chunk",
            BridgeEvent::ProviderSwitched { .. } => "bridge.event.provider.switched",
            BridgeEvent::ProviderHealth { .. } => "bridge.event.provider.health",
            BridgeEvent::MetricsUpdated { .. } => "bridge.event.metrics.updated",
            BridgeEvent::ProvidersListed { .. } => "bridge.event.providers.listed",
            BridgeEvent::ModelsListed { .. } => "bridge.event.models.listed",
        };
        
        info!("Publishing event to {}: {:?}", subject, event);
        
        nats_client
            .publish(subject, serde_json::to_vec(&envelope)?.into())
            .await
            .map_err(|e| BridgeError::NatsPublish(e.to_string()))?;
        
        Ok(())
    }
} 