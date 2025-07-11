//! Tests for dashboard window functionality

#[cfg(test)]
mod tests {
    use alchemist::dashboard::{DashboardData, DomainInfo, DomainHealth, SystemStatus};
    use tokio::sync::mpsc;
    
    #[test]
    fn test_dashboard_data_creation() {
        let data = DashboardData::example();
        
        // Verify we have domains
        assert!(!data.domains.is_empty(), "Dashboard should have domains");
        
        // Verify system status
        assert!(data.system_status.memory_usage_mb >= 0.0);
        
        // Verify at least one domain is healthy
        let healthy_domains = data.domains.iter()
            .filter(|d| matches!(d.health, DomainHealth::Healthy))
            .count();
        assert!(healthy_domains > 0, "Should have at least one healthy domain");
    }
    
    #[tokio::test]
    async fn test_dashboard_channel_updates() {
        let (tx, mut rx) = mpsc::channel::<DashboardData>(10);
        
        // Send initial data
        let data1 = DashboardData::example();
        tx.send(data1.clone()).await.unwrap();
        
        // Verify we can receive it
        let received = rx.try_recv();
        assert!(received.is_ok(), "Should receive dashboard data");
        
        // Send updated data
        let mut data2 = DashboardData::example();
        data2.system_status.total_events += 100;
        tx.send(data2.clone()).await.unwrap();
        
        // Verify we get the update
        let updated = rx.try_recv();
        assert!(updated.is_ok(), "Should receive updated data");
        let updated_data = updated.unwrap();
        assert_eq!(updated_data.system_status.total_events, data2.system_status.total_events);
    }
    
    #[test]
    fn test_domain_info_healthy_flag() {
        let data = DashboardData::example();
        
        for domain in &data.domains {
            let expected_healthy = matches!(domain.health, DomainHealth::Healthy);
            assert_eq!(domain.healthy, expected_healthy, 
                "Domain {} healthy flag should match health status", domain.name);
        }
    }
}