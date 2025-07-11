//! CPU monitoring utilities for Linux

use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use tracing::debug;

/// CPU usage statistics
#[derive(Debug, Clone, Default)]
pub struct CpuStats {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
}

impl CpuStats {
    /// Calculate total CPU time
    pub fn total(&self) -> u64 {
        self.user + self.nice + self.system + self.idle + 
        self.iowait + self.irq + self.softirq + self.steal
    }
    
    /// Calculate active CPU time (non-idle)
    pub fn active(&self) -> u64 {
        self.total() - self.idle - self.iowait
    }
}

/// Read CPU stats from /proc/stat
pub fn read_cpu_stats() -> Option<CpuStats> {
    #[cfg(target_os = "linux")]
    {
        let stat_content = fs::read_to_string("/proc/stat").ok()?;
        
        for line in stat_content.lines() {
            if line.starts_with("cpu ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 8 {
                    return Some(CpuStats {
                        user: parts.get(1)?.parse().ok()?,
                        nice: parts.get(2)?.parse().ok()?,
                        system: parts.get(3)?.parse().ok()?,
                        idle: parts.get(4)?.parse().ok()?,
                        iowait: parts.get(5)?.parse().ok()?,
                        irq: parts.get(6)?.parse().ok()?,
                        softirq: parts.get(7)?.parse().ok()?,
                        steal: parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0),
                    });
                }
            }
        }
    }
    
    None
}

/// Calculate CPU usage percentage between two measurements
pub fn calculate_cpu_usage(prev: &CpuStats, curr: &CpuStats) -> f32 {
    let prev_total = prev.total();
    let curr_total = curr.total();
    
    if curr_total <= prev_total {
        return 0.0;
    }
    
    let total_diff = curr_total - prev_total;
    let active_diff = curr.active() - prev.active();
    
    (active_diff as f32 / total_diff as f32) * 100.0
}

/// Get current CPU usage percentage
pub async fn get_cpu_usage() -> f32 {
    #[cfg(target_os = "linux")]
    {
        if let Some(stats1) = read_cpu_stats() {
            sleep(Duration::from_millis(100)).await;
            if let Some(stats2) = read_cpu_stats() {
                return calculate_cpu_usage(&stats1, &stats2);
            }
        }
    }
    
    0.0
}

/// CPU monitor that tracks usage over time
pub struct CpuMonitor {
    last_stats: Option<CpuStats>,
}

impl CpuMonitor {
    pub fn new() -> Self {
        Self {
            last_stats: None,
        }
    }
    
    /// Get current CPU usage percentage
    pub fn get_usage(&mut self) -> f32 {
        #[cfg(target_os = "linux")]
        {
            if let Some(current_stats) = read_cpu_stats() {
                let usage = if let Some(last) = &self.last_stats {
                    calculate_cpu_usage(last, &current_stats)
                } else {
                    0.0
                };
                
                self.last_stats = Some(current_stats);
                debug!("CPU usage: {:.1}%", usage);
                return usage;
            }
        }
        
        0.0
    }
    
    /// Get CPU core count
    pub fn get_core_count() -> usize {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
                return content.lines()
                    .filter(|line| line.starts_with("processor"))
                    .count();
            }
        }
        
        1
    }
}

/// Get system load average
pub fn get_load_average() -> Option<(f32, f32, f32)> {
    #[cfg(target_os = "linux")]
    {
        let content = fs::read_to_string("/proc/loadavg").ok()?;
        let parts: Vec<&str> = content.split_whitespace().collect();
        
        if parts.len() >= 3 {
            let load1 = parts[0].parse().ok()?;
            let load5 = parts[1].parse().ok()?;
            let load15 = parts[2].parse().ok()?;
            return Some((load1, load5, load15));
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_stats_calculation() {
        let prev = CpuStats {
            user: 1000,
            nice: 0,
            system: 500,
            idle: 8000,
            iowait: 500,
            irq: 0,
            softirq: 0,
            steal: 0,
        };
        
        let curr = CpuStats {
            user: 1100,
            nice: 0,
            system: 550,
            idle: 8800,
            iowait: 550,
            irq: 0,
            softirq: 0,
            steal: 0,
        };
        
        let usage = calculate_cpu_usage(&prev, &curr);
        // Active increased by 150, total by 1000, so 15% usage
        assert!((usage - 15.0).abs() < 0.1);
    }
    
    #[tokio::test]
    async fn test_get_cpu_usage() {
        let usage = get_cpu_usage().await;
        // CPU usage should be between 0 and 100
        assert!(usage >= 0.0);
        assert!(usage <= 100.0);
    }
    
    #[test]
    fn test_cpu_monitor() {
        let mut monitor = CpuMonitor::new();
        let usage = monitor.get_usage();
        
        // First call might return 0 as there's no previous stats
        assert!(usage >= 0.0);
        assert!(usage <= 100.0);
        
        // Get core count
        let cores = CpuMonitor::get_core_count();
        assert!(cores >= 1);
    }
}