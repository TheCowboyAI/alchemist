//! Infrastructure Layer 1.3: Message Routing Tests for cim-ipld
//! 
//! User Story: As a content system, I need to route content events to appropriate handlers
//!
//! Test Requirements:
//! - Verify content event routing based on type
//! - Verify codec selection for content processing
//! - Verify event ordering and delivery guarantees
//! - Verify error handling in routing failures
//!
//! Event Sequence:
//! 1. RouterInitialized
//! 2. HandlerRegistered { content_type, handler }
//! 3. ContentEventReceived { cid, content_type }
//! 4. EventRouted { handler, cid }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Initialize Router]
//!     B --> C[RouterInitialized]
//!     C --> D[Register Content Handlers]
//!     D --> E[HandlerRegistered]
//!     E --> F[Receive Content Event]
//!     F --> G[ContentEventReceived]
//!     G --> H[Route to Handler]
//!     H --> I[EventRouted]
//!     I --> J[Process Content]
//!     J --> K[Test Success]
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// Content event for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEvent {
    pub cid: String,
    pub content_type: String,
    pub operation: ContentOperation,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentOperation {
    Store,
    Retrieve,
    Transform,
    Delete,
}

/// Handler trait for content processing
pub trait ContentHandler: Send + Sync {
    fn handle(&self, event: &ContentEvent) -> Result<HandlerResult, String>;
    fn supported_types(&self) -> Vec<String>;
    fn clone_box(&self) -> Box<dyn ContentHandler>;
}

impl Clone for Box<dyn ContentHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct HandlerResult {
    pub success: bool,
    pub output_cid: Option<String>,
    pub message: String,
}

/// Document content handler
#[derive(Clone)]
pub struct DocumentHandler {
    pub processed_count: Arc<Mutex<usize>>,
}

impl ContentHandler for DocumentHandler {
    fn handle(&self, event: &ContentEvent) -> Result<HandlerResult, String> {
        let mut count = self.processed_count.lock().unwrap();
        *count += 1;
        
        Ok(HandlerResult {
            success: true,
            output_cid: Some(format!("{event.cid}_processed")),
            message: format!("Document {event.cid} processed"),
        })
    }
    
    fn supported_types(&self) -> Vec<String> {
        vec![
            "text/plain".to_string(),
            "text/markdown".to_string(),
            "application/pdf".to_string(),
        ]
    }
    
    fn clone_box(&self) -> Box<dyn ContentHandler> {
        Box::new(self.clone())
    }
}

/// Image content handler
#[derive(Clone)]
pub struct ImageHandler {
    pub processed_count: Arc<Mutex<usize>>,
}

impl ContentHandler for ImageHandler {
    fn handle(&self, event: &ContentEvent) -> Result<HandlerResult, String> {
        let mut count = self.processed_count.lock().unwrap();
        *count += 1;
        
        Ok(HandlerResult {
            success: true,
            output_cid: Some(format!("{event.cid}_image_processed")),
            message: format!("Image {event.cid} processed"),
        })
    }
    
    fn supported_types(&self) -> Vec<String> {
        vec![
            "image/jpeg".to_string(),
            "image/png".to_string(),
            "image/gif".to_string(),
        ]
    }
    
    fn clone_box(&self) -> Box<dyn ContentHandler> {
        Box::new(self.clone())
    }
}

/// Content event router
pub struct ContentRouter {
    handlers: HashMap<String, Box<dyn ContentHandler>>,
    routing_log: Vec<RoutingEvent>,
    fallback_handler: Option<Box<dyn ContentHandler>>,
}

#[derive(Debug, Clone)]
pub struct RoutingEvent {
    pub event_cid: String,
    pub handler_type: String,
    pub result: bool,
}

impl ContentRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            routing_log: Vec::new(),
            fallback_handler: None,
        }
    }
    
    pub fn register_handler(&mut self, handler: Box<dyn ContentHandler>) {
        for content_type in handler.supported_types() {
            self.handlers.insert(content_type, handler.clone());
        }
    }
    
    pub fn set_fallback_handler(&mut self, handler: Box<dyn ContentHandler>) {
        self.fallback_handler = Some(handler);
    }
    
    pub fn route_event(&mut self, event: ContentEvent) -> Result<HandlerResult, String> {
        let handler = self.handlers.get(&event.content_type)
            .or(self.fallback_handler.as_ref())
            .ok_or_else(|| format!("No handler for content type: {event.content_type}"))?;
        
        let result = handler.handle(&event)?;
        
        self.routing_log.push(RoutingEvent {
            event_cid: event.cid.clone(),
            handler_type: event.content_type.clone(),
            result: result.success,
        });
        
        Ok(result)
    }
    
    pub fn get_routing_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        for event in &self.routing_log {
            *stats.entry(event.handler_type.clone()).or_insert(0) += 1;
        }
        
        stats
    }
    
    pub fn clear_routing_log(&mut self) {
        self.routing_log.clear();
    }
}

/// Default fallback handler
#[derive(Clone)]
pub struct FallbackHandler;

impl ContentHandler for FallbackHandler {
    fn handle(&self, event: &ContentEvent) -> Result<HandlerResult, String> {
        Ok(HandlerResult {
            success: true,
            output_cid: Some(format!("{event.cid}_fallback")),
            message: format!("Handled by fallback: {event.content_type}"),
        })
    }
    
    fn supported_types(&self) -> Vec<String> {
        vec!["*/*".to_string()]
    }
    
    fn clone_box(&self) -> Box<dyn ContentHandler> {
        Box::new(self.clone())
    }
}

/// Router event types
#[derive(Debug, Clone, PartialEq)]
pub enum RouterEvent {
    RouterInitialized,
    HandlerRegistered { content_type: String, handler: String },
    ContentEventReceived { cid: String, content_type: String },
    EventRouted { handler: String, cid: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_initialization() {
        // Arrange & Act
        let router = ContentRouter::new();
        
        // Assert
        assert_eq!(router.handlers.len(), 0);
        assert_eq!(router.routing_log.len(), 0);
        assert!(router.fallback_handler.is_none());
    }
    
    #[test]
    fn test_handler_registration() {
        // Arrange
        let mut router = ContentRouter::new();
        let doc_handler = Box::new(DocumentHandler {
            processed_count: Arc::new(Mutex::new(0)),
        });
        
        // Act
        router.register_handler(doc_handler);
        
        // Assert
        assert!(router.handlers.contains_key("text/plain"));
        assert!(router.handlers.contains_key("text/markdown"));
        assert!(router.handlers.contains_key("application/pdf"));
    }
    
    #[test]
    fn test_content_routing_success() {
        // Arrange
        let mut router = ContentRouter::new();
        let doc_handler = Box::new(DocumentHandler {
            processed_count: Arc::new(Mutex::new(0)),
        });
        router.register_handler(doc_handler);
        
        let event = ContentEvent {
            cid: "bafybeig123".to_string(),
            content_type: "text/plain".to_string(),
            operation: ContentOperation::Store,
            metadata: HashMap::new(),
        };
        
        // Act
        let result = router.route_event(event).unwrap();
        
        // Assert
        assert!(result.success);
        assert_eq!(result.output_cid, Some("bafybeig123_processed".to_string()));
        assert_eq!(router.routing_log.len(), 1);
    }
    
    #[test]
    fn test_multiple_content_types() {
        // Arrange
        let mut router = ContentRouter::new();
        
        let doc_count = Arc::new(Mutex::new(0));
        let img_count = Arc::new(Mutex::new(0));
        
        router.register_handler(Box::new(DocumentHandler {
            processed_count: doc_count.clone(),
        }));
        router.register_handler(Box::new(ImageHandler {
            processed_count: img_count.clone(),
        }));
        
        let events = vec![
            ContentEvent {
                cid: "doc1".to_string(),
                content_type: "text/plain".to_string(),
                operation: ContentOperation::Store,
                metadata: HashMap::new(),
            },
            ContentEvent {
                cid: "img1".to_string(),
                content_type: "image/jpeg".to_string(),
                operation: ContentOperation::Store,
                metadata: HashMap::new(),
            },
            ContentEvent {
                cid: "doc2".to_string(),
                content_type: "text/markdown".to_string(),
                operation: ContentOperation::Transform,
                metadata: HashMap::new(),
            },
        ];
        
        // Act
        for event in events {
            router.route_event(event).unwrap();
        }
        
        // Assert
        assert_eq!(*doc_count.lock().unwrap(), 2);
        assert_eq!(*img_count.lock().unwrap(), 1);
        
        let stats = router.get_routing_stats();
        assert_eq!(stats.get("text/plain"), Some(&1));
        assert_eq!(stats.get("image/jpeg"), Some(&1));
        assert_eq!(stats.get("text/markdown"), Some(&1));
    }
    
    #[test]
    fn test_fallback_handler() {
        // Arrange
        let mut router = ContentRouter::new();
        router.set_fallback_handler(Box::new(FallbackHandler));
        
        let event = ContentEvent {
            cid: "unknown1".to_string(),
            content_type: "application/x-unknown".to_string(),
            operation: ContentOperation::Store,
            metadata: HashMap::new(),
        };
        
        // Act
        let result = router.route_event(event).unwrap();
        
        // Assert
        assert!(result.success);
        assert_eq!(result.output_cid, Some("unknown1_fallback".to_string()));
        assert!(result.message.contains("fallback"));
    }
    
    #[test]
    fn test_no_handler_error() {
        // Arrange
        let mut router = ContentRouter::new();
        
        let event = ContentEvent {
            cid: "test1".to_string(),
            content_type: "application/x-unhandled".to_string(),
            operation: ContentOperation::Store,
            metadata: HashMap::new(),
        };
        
        // Act
        let result = router.route_event(event);
        
        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No handler for content type"));
    }
    
    #[test]
    fn test_routing_log_and_stats() {
        // Arrange
        let mut router = ContentRouter::new();
        router.register_handler(Box::new(DocumentHandler {
            processed_count: Arc::new(Mutex::new(0)),
        }));
        
        // Route multiple events
        for i in 0..5 {
            let event = ContentEvent {
                cid: format!("doc{i}"),
                content_type: "text/plain".to_string(),
                operation: ContentOperation::Store,
                metadata: HashMap::new(),
            };
            router.route_event(event).unwrap();
        }
        
        // Act
        let stats = router.get_routing_stats();
        
        // Assert
        assert_eq!(router.routing_log.len(), 5);
        assert_eq!(stats.get("text/plain"), Some(&5));
        
        // Test log clearing
        router.clear_routing_log();
        assert_eq!(router.routing_log.len(), 0);
    }
    
    #[test]
    fn test_content_operations() {
        // Arrange
        let mut router = ContentRouter::new();
        router.register_handler(Box::new(DocumentHandler {
            processed_count: Arc::new(Mutex::new(0)),
        }));
        
        let operations = vec![
            ContentOperation::Store,
            ContentOperation::Retrieve,
            ContentOperation::Transform,
            ContentOperation::Delete,
        ];
        
        // Act & Assert
        for (i, op) in operations.into_iter().enumerate() {
            let event = ContentEvent {
                cid: format!("doc{i}"),
                content_type: "text/plain".to_string(),
                operation: op,
                metadata: HashMap::new(),
            };
            
            let result = router.route_event(event).unwrap();
            assert!(result.success);
        }
        
        assert_eq!(router.routing_log.len(), 4);
    }
} 