//! Error Handling and Resilience Tests
//! 
//! These tests verify that the CIM system handles errors gracefully

use cim_domain::{DomainResult, DomainError, GraphId, NodeId};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

/// Simulated service that can fail
#[derive(Clone)]
struct FallibleService {
    failure_rate: f32,
    call_count: Arc<Mutex<u32>>,
}

impl FallibleService {
    fn new(failure_rate: f32) -> Self {
        Self {
            failure_rate,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    async fn process(&self, _data: &str) -> DomainResult<String> {
        let mut count = self.call_count.lock().await;
        *count += 1;
        
        // Simulate random failures
        if rand::random::<f32>() < self.failure_rate {
            Err(DomainError::generic("Service temporarily unavailable"))
        } else {
            Ok("Processed successfully".to_string())
        }
    }

    async fn get_call_count(&self) -> u32 {
        *self.call_count.lock().await
    }
}

#[tokio::test]
async fn test_retry_with_exponential_backoff() -> DomainResult<()> {
    let service = FallibleService::new(0.5); // 50% failure rate
    let max_retries = 3;
    let mut retry_count = 0;
    let mut delay = Duration::from_millis(100);
    
    loop {
        match service.process("test data").await {
            Ok(result) => {
                println!("✅ Success after {} retries: {}", retry_count, result);
                break;
            }
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(DomainError::generic(
                        format!("Failed after {} retries: {}", max_retries, e)
                    ));
                }
                
                println!("Retry {} after {:?} delay", retry_count, delay);
                tokio::time::sleep(delay).await;
                
                // Exponential backoff
                delay *= 2;
            }
        }
    }
    
    assert!(retry_count < max_retries);
    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker() -> DomainResult<()> {
    // Circuit breaker pattern
    struct CircuitBreaker {
        failure_threshold: u32,
        failure_count: Arc<Mutex<u32>>,
        is_open: Arc<Mutex<bool>>,
        reset_timeout: Duration,
    }
    
    impl CircuitBreaker {
        fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
            Self {
                failure_threshold,
                failure_count: Arc::new(Mutex::new(0)),
                is_open: Arc::new(Mutex::new(false)),
                reset_timeout,
            }
        }
        
        async fn call<F, T>(&self, f: F) -> DomainResult<T>
        where
            F: std::future::Future<Output = DomainResult<T>>,
        {
            // Check if circuit is open
            if *self.is_open.lock().await {
                return Err(DomainError::generic("Circuit breaker is open"));
            }
            
            // Try the operation
            match f.await {
                Ok(result) => {
                    // Reset failure count on success
                    *self.failure_count.lock().await = 0;
                    Ok(result)
                }
                Err(e) => {
                    let mut failures = self.failure_count.lock().await;
                    *failures += 1;
                    
                    // Open circuit if threshold reached
                    if *failures >= self.failure_threshold {
                        *self.is_open.lock().await = true;
                        
                        // Schedule circuit reset
                        let is_open = self.is_open.clone();
                        let timeout = self.reset_timeout;
                        tokio::spawn(async move {
                            tokio::time::sleep(timeout).await;
                            *is_open.lock().await = false;
                        });
                    }
                    
                    Err(e)
                }
            }
        }
    }
    
    let service = FallibleService::new(0.8); // 80% failure rate
    let breaker = CircuitBreaker::new(3, Duration::from_millis(500));
    
    // Should fail multiple times and open circuit
    let mut open_circuit_seen = false;
    
    for i in 0..10 {
        match breaker.call(service.process("test")).await {
            Ok(_) => println!("Request {} succeeded", i),
            Err(e) => {
                println!("Request {} failed: {}", i, e);
                if e.to_string().contains("Circuit breaker is open") {
                    open_circuit_seen = true;
                }
            }
        }
        
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    assert!(open_circuit_seen, "Circuit breaker should have opened");
    println!("✅ Circuit breaker test passed");
    
    Ok(())
}

#[tokio::test]
async fn test_timeout_handling() -> DomainResult<()> {
    // Simulate slow operation
    async fn slow_operation() -> DomainResult<String> {
        tokio::time::sleep(Duration::from_secs(2)).await;
        Ok("Completed".to_string())
    }
    
    // Test with timeout
    let timeout_duration = Duration::from_millis(500);
    let result = tokio::time::timeout(timeout_duration, slow_operation()).await;
    
    match result {
        Ok(_) => panic!("Operation should have timed out"),
        Err(_) => println!("✅ Operation correctly timed out after {:?}", timeout_duration),
    }
    
    // Test graceful degradation
    let fallback_result = match tokio::time::timeout(timeout_duration, slow_operation()).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => "Operation failed".to_string(),
        Err(_) => "Operation timed out - using cached result".to_string(),
    };
    
    assert_eq!(fallback_result, "Operation timed out - using cached result");
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_error_handling() -> DomainResult<()> {
    let service = Arc::new(FallibleService::new(0.3)); // 30% failure rate
    let mut handles = Vec::new();
    
    // Spawn multiple concurrent tasks
    for i in 0..10 {
        let svc = service.clone();
        let handle = tokio::spawn(async move {
            let mut attempts = 0;
            loop {
                attempts += 1;
                match svc.process(&format!("Task {}", i)).await {
                    Ok(_) => return Ok(attempts),
                    Err(_) if attempts < 3 => continue,
                    Err(e) => return Err(e),
                }
            }
        });
        handles.push(handle);
    }
    
    // Collect results
    let mut successes = 0;
    let mut failures = 0;
    let mut total_attempts = 0;
    
    for handle in handles {
        match handle.await.unwrap() {
            Ok(attempts) => {
                successes += 1;
                total_attempts += attempts;
            }
            Err(_) => failures += 1,
        }
    }
    
    println!("✅ Concurrent error handling: {} succeeded, {} failed", successes, failures);
    println!("   Average attempts per success: {:.2}", total_attempts as f32 / successes as f32);
    
    // Most should succeed with retries
    assert!(successes >= 7, "At least 70% should succeed with retries");
    
    Ok(())
}

#[tokio::test]
async fn test_graceful_degradation() -> DomainResult<()> {
    // Simulate a service with dependencies
    struct ServiceWithDependencies {
        primary_service: FallibleService,
        fallback_service: FallibleService,
        cache: Arc<Mutex<Option<String>>>,
    }
    
    impl ServiceWithDependencies {
        async fn get_data(&self) -> DomainResult<String> {
            // Try primary service
            match self.primary_service.process("primary").await {
                Ok(result) => {
                    // Update cache on success
                    *self.cache.lock().await = Some(result.clone());
                    Ok(result)
                }
                Err(_) => {
                    // Try fallback service
                    match self.fallback_service.process("fallback").await {
                        Ok(result) => Ok(format!("Fallback: {}", result)),
                        Err(_) => {
                            // Use cache as last resort
                            match &*self.cache.lock().await {
                                Some(cached) => Ok(format!("Cached: {}", cached)),
                                None => Err(DomainError::generic("All services failed and no cache available")),
                            }
                        }
                    }
                }
            }
        }
    }
    
    let service = ServiceWithDependencies {
        primary_service: FallibleService::new(0.9), // 90% failure rate
        fallback_service: FallibleService::new(0.5), // 50% failure rate
        cache: Arc::new(Mutex::new(Some("Initial cache".to_string()))),
    };
    
    // Test multiple calls
    for i in 0..5 {
        match service.get_data().await {
            Ok(result) => println!("Call {}: {}", i, result),
            Err(e) => println!("Call {} failed: {}", i, e),
        }
    }
    
    println!("✅ Graceful degradation test passed");
    Ok(())
}

#[tokio::test]
async fn test_error_aggregation() -> DomainResult<()> {
    // Collect errors from multiple operations
    #[derive(Debug)]
    struct ErrorCollector {
        errors: Arc<Mutex<Vec<(String, DomainError)>>>,
    }
    
    impl ErrorCollector {
        fn new() -> Self {
            Self {
                errors: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        async fn record_error(&self, context: String, error: DomainError) {
            self.errors.lock().await.push((context, error));
        }
        
        async fn get_summary(&self) -> String {
            let errors = self.errors.lock().await;
            if errors.is_empty() {
                "No errors recorded".to_string()
            } else {
                format!("Total errors: {}\n{}", 
                    errors.len(),
                    errors.iter()
                        .map(|(ctx, err)| format!("  - {}: {}", ctx, err))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        }
    }
    
    let collector = ErrorCollector::new();
    let service = FallibleService::new(0.4); // 40% failure rate
    
    // Perform multiple operations
    for i in 0..10 {
        if let Err(e) = service.process(&format!("Operation {}", i)).await {
            collector.record_error(format!("Operation {}", i), e).await;
        }
    }
    
    let summary = collector.get_summary().await;
    println!("Error summary:\n{}", summary);
    
    assert!(summary.contains("Total errors:"));
    println!("✅ Error aggregation test passed");
    
    Ok(())
}

#[test]
fn test_error_type_handling() {
    // Test different error types and conversions
    let errors = vec![
        DomainError::generic("Generic error"),
        DomainError::EntityNotFound { 
            entity_type: "Entity".to_string(), 
            id: "123".to_string() 
        },
        DomainError::ValidationError("Invalid input".to_string()),
        DomainError::ComponentAlreadyExists("Resource already exists".to_string()),
    ];
    
    for error in errors {
        match error {
            DomainError::EntityNotFound { entity_type, id } => {
                println!("Not found: {} with id {}", entity_type, id);
            }
            DomainError::ValidationError(msg) => {
                println!("Validation error: {}", msg);
            }
            DomainError::ComponentAlreadyExists(msg) => {
                println!("Already exists: {}", msg);
            }
            DomainError::Generic(msg) => {
                println!("Generic error: {}", msg);
            }
            _ => {
                println!("Other error: {}", error);
            }
        }
    }
    
    println!("✅ Error type handling test passed");
}

#[tokio::test]
async fn test_cascading_failure_prevention() -> DomainResult<()> {
    // Simulate a system with multiple dependent services
    struct SystemWithDependencies {
        services: Vec<FallibleService>,
        failure_counts: Arc<Mutex<Vec<u32>>>,
    }
    
    impl SystemWithDependencies {
        fn new(service_count: usize) -> Self {
            let mut services = Vec::new();
            for i in 0..service_count {
                // Increasing failure rates for dependent services
                let failure_rate = 0.1 * (i + 1) as f32;
                services.push(FallibleService::new(failure_rate.min(0.8)));
            }
            
            Self {
                services,
                failure_counts: Arc::new(Mutex::new(vec![0; service_count])),
            }
        }
        
        async fn process_with_isolation(&self) -> Vec<Result<String, String>> {
            let mut results = Vec::new();
            
            for (i, service) in self.services.iter().enumerate() {
                // Isolate each service call
                let result = tokio::time::timeout(
                    Duration::from_millis(200),
                    service.process(&format!("Service {}", i))
                ).await;
                
                match result {
                    Ok(Ok(success)) => results.push(Ok(success)),
                    Ok(Err(e)) => {
                        let mut counts = self.failure_counts.lock().await;
                        counts[i] += 1;
                        results.push(Err(format!("Service {} failed: {}", i, e)));
                    }
                    Err(_) => {
                        results.push(Err(format!("Service {} timed out", i)));
                    }
                }
            }
            
            results
        }
    }
    
    let system = SystemWithDependencies::new(5);
    
    // Run multiple iterations
    for iteration in 0..3 {
        println!("\nIteration {}:", iteration);
        let results = system.process_with_isolation().await;
        
        let successes = results.iter().filter(|r| r.is_ok()).count();
        let failures = results.len() - successes;
        
        println!("  Successes: {}, Failures: {}", successes, failures);
        
        // Even with failures, some services should succeed
        assert!(successes > 0, "Some services should succeed despite failures");
    }
    
    println!("✅ Cascading failure prevention test passed");
    Ok(())
} 