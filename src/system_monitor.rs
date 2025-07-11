//! System monitoring utilities for dashboard

use std::time::{Duration, Instant};

/// Get current memory usage in MB
pub fn get_memory_usage_mb() -> f32 {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        
        // Try to read from /proc/self/status
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<f32>() {
                            return kb / 1024.0; // Convert KB to MB
                        }
                    }
                }
            }
        }
    }
    
    // Fallback for other platforms or if reading fails
    0.0
}

/// System uptime tracker
pub struct UptimeTracker {
    start_time: Instant,
}

impl UptimeTracker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
    
    pub fn get_uptime_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
    
    pub fn get_uptime_formatted(&self) -> String {
        let total_seconds = self.start_time.elapsed().as_secs();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

impl Default for UptimeTracker {
    fn default() -> Self {
        Self::new()
    }
}