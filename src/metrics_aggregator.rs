//! Metrics aggregation and analysis for Alchemist
//! 
//! Provides advanced metrics aggregation, trend analysis, and alerting

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};
use chrono::{DateTime, Utc};

/// Metrics aggregator for time-series analysis
pub struct MetricsAggregator {
    time_series: Arc<RwLock<HashMap<String, TimeSeries>>>,
    aggregation_window: Duration,
    retention_period: Duration,
}

/// Time series data for a metric
#[derive(Debug, Clone)]
struct TimeSeries {
    name: String,
    data_points: VecDeque<DataPoint>,
    metric_type: MetricType,
    unit: String,
}

/// Single data point in time series
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f64,
    tags: HashMap<String, String>,
}

/// Type of metric for proper aggregation
#[derive(Debug, Clone, Copy, PartialEq)]
enum MetricType {
    Counter,    // Always increasing (e.g., total requests)
    Gauge,      // Can go up or down (e.g., memory usage)
    Histogram,  // Distribution of values (e.g., latencies)
    Rate,       // Rate per time unit (e.g., requests/sec)
}

/// Aggregated metrics over a time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub metric: String,
    pub window: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub count: usize,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub stddev: f64,
    pub rate: f64,
}

/// Trend analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub metric: String,
    pub trend_direction: TrendDirection,
    pub change_percent: f64,
    pub is_anomaly: bool,
    pub forecast_next_value: Option<f64>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrendDirection {
    Rising,
    Falling,
    Stable,
    Volatile,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub metric: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration: Duration,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
    OutsideRange { min: f64, max: f64 },
    RateOfChange { percent: f64 },
    Anomaly { sensitivity: f64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl MetricsAggregator {
    pub fn new(aggregation_window: Duration, retention_period: Duration) -> Self {
        Self {
            time_series: Arc::new(RwLock::new(HashMap::new())),
            aggregation_window,
            retention_period,
        }
    }
    
    /// Record a metric value
    pub async fn record(&self, metric: &str, value: f64, tags: HashMap<String, String>) {
        let mut series_map = self.time_series.write().await;
        
        let series = series_map.entry(metric.to_string()).or_insert_with(|| {
            TimeSeries {
                name: metric.to_string(),
                data_points: VecDeque::new(),
                metric_type: Self::infer_metric_type(metric),
                unit: Self::infer_unit(metric),
            }
        });
        
        let data_point = DataPoint {
            timestamp: Utc::now(),
            value,
            tags,
        };
        
        series.data_points.push_back(data_point);
        
        // Cleanup old data
        let cutoff = Utc::now() - chrono::Duration::from_std(self.retention_period).unwrap();
        while let Some(front) = series.data_points.front() {
            if front.timestamp < cutoff {
                series.data_points.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// Get aggregated metrics for a time window
    pub async fn aggregate(&self, metric: &str, window: Duration) -> Option<AggregatedMetrics> {
        let series_map = self.time_series.read().await;
        let series = series_map.get(metric)?;
        
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::from_std(window).unwrap();
        
        let window_data: Vec<f64> = series.data_points.iter()
            .filter(|dp| dp.timestamp >= start_time && dp.timestamp <= end_time)
            .map(|dp| dp.value)
            .collect();
        
        if window_data.is_empty() {
            return None;
        }
        
        let count = window_data.len();
        let sum: f64 = window_data.iter().sum();
        let min = window_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = window_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = sum / count as f64;
        
        // Calculate percentiles
        let mut sorted = window_data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p50 = Self::percentile(&sorted, 0.50);
        let p95 = Self::percentile(&sorted, 0.95);
        let p99 = Self::percentile(&sorted, 0.99);
        
        // Calculate standard deviation
        let variance = window_data.iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f64>() / count as f64;
        let stddev = variance.sqrt();
        
        // Calculate rate (per second)
        let duration_secs = window.as_secs_f64();
        let rate = match series.metric_type {
            MetricType::Counter => {
                if count >= 2 {
                    (max - min) / duration_secs
                } else {
                    0.0
                }
            }
            _ => count as f64 / duration_secs,
        };
        
        Some(AggregatedMetrics {
            metric: metric.to_string(),
            window: format!("{}s", window.as_secs()),
            start_time,
            end_time,
            count,
            sum,
            min,
            max,
            avg,
            p50,
            p95,
            p99,
            stddev,
            rate,
        })
    }
    
    /// Analyze trend for a metric
    pub async fn analyze_trend(&self, metric: &str) -> Option<TrendAnalysis> {
        let series_map = self.time_series.read().await;
        let series = series_map.get(metric)?;
        
        if series.data_points.len() < 10 {
            return None; // Not enough data
        }
        
        let values: Vec<f64> = series.data_points.iter()
            .map(|dp| dp.value)
            .collect();
        
        // Simple linear regression for trend
        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;
        
        let mut num = 0.0;
        let mut den = 0.0;
        
        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            num += (x - x_mean) * (y - y_mean);
            den += (x - x_mean).powi(2);
        }
        
        let slope = if den != 0.0 { num / den } else { 0.0 };
        let intercept = y_mean - slope * x_mean;
        
        // Determine trend direction
        let recent_avg = values.iter().rev().take(5).sum::<f64>() / 5.0;
        let older_avg = values.iter().skip(values.len() - 10).take(5).sum::<f64>() / 5.0;
        let change_percent = ((recent_avg - older_avg) / older_avg) * 100.0;
        
        let trend_direction = if change_percent.abs() < 1.0 {
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Rising
        } else if slope < 0.0 {
            TrendDirection::Falling
        } else {
            TrendDirection::Volatile
        };
        
        // Simple anomaly detection using z-score
        let is_anomaly = if let Some(&last_value) = values.last() {
            let z_score = (last_value - y_mean) / stddev(&values);
            z_score.abs() > 3.0
        } else {
            false
        };
        
        // Forecast next value
        let forecast_next_value = Some(slope * n + intercept);
        
        // Confidence based on R-squared
        let r_squared = Self::calculate_r_squared(&values, slope, intercept);
        let confidence = r_squared * 100.0;
        
        Some(TrendAnalysis {
            metric: metric.to_string(),
            trend_direction,
            change_percent,
            is_anomaly,
            forecast_next_value,
            confidence,
        })
    }
    
    /// Get metrics summary
    pub async fn get_summary(&self) -> MetricsSummary {
        let series_map = self.time_series.read().await;
        
        let total_metrics = series_map.len();
        let total_data_points: usize = series_map.values()
            .map(|s| s.data_points.len())
            .sum();
        
        let metrics_by_type = series_map.values()
            .fold(HashMap::new(), |mut acc, series| {
                *acc.entry(series.metric_type).or_insert(0) += 1;
                acc
            });
        
        MetricsSummary {
            total_metrics,
            total_data_points,
            oldest_data_point: series_map.values()
                .flat_map(|s| s.data_points.front())
                .map(|dp| dp.timestamp)
                .min(),
            metrics_by_type: metrics_by_type.into_iter()
                .map(|(t, c)| (format!("{:?}", t), c))
                .collect(),
        }
    }
    
    // Helper methods
    
    fn infer_metric_type(metric: &str) -> MetricType {
        if metric.contains("_total") || metric.contains("_count") {
            MetricType::Counter
        } else if metric.contains("_rate") || metric.contains("/s") {
            MetricType::Rate
        } else if metric.contains("latency") || metric.contains("duration") {
            MetricType::Histogram
        } else {
            MetricType::Gauge
        }
    }
    
    fn infer_unit(metric: &str) -> String {
        if metric.contains("_bytes") || metric.contains("memory") {
            "bytes".to_string()
        } else if metric.contains("_ms") || metric.contains("latency") {
            "ms".to_string()
        } else if metric.contains("_percent") || metric.contains("cpu") {
            "%".to_string()
        } else if metric.contains("_count") || metric.contains("_total") {
            "count".to_string()
        } else {
            "unit".to_string()
        }
    }
    
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        let idx = (sorted.len() as f64 * p) as usize;
        sorted.get(idx.saturating_sub(1)).copied().unwrap_or(0.0)
    }
    
    fn calculate_r_squared(values: &[f64], slope: f64, intercept: f64) -> f64 {
        let y_mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;
        
        for (i, &y) in values.iter().enumerate() {
            let y_pred = slope * i as f64 + intercept;
            ss_res += (y - y_pred).powi(2);
            ss_tot += (y - y_mean).powi(2);
        }
        
        if ss_tot == 0.0 {
            0.0
        } else {
            1.0 - (ss_res / ss_tot)
        }
    }
}

/// Metrics summary
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSummary {
    pub total_metrics: usize,
    pub total_data_points: usize,
    pub oldest_data_point: Option<DateTime<Utc>>,
    pub metrics_by_type: HashMap<String, usize>,
}

fn stddev(values: &[f64]) -> f64 {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_aggregation() {
        let aggregator = MetricsAggregator::new(
            Duration::from_secs(60),
            Duration::from_secs(3600),
        );
        
        // Record some metrics
        for i in 0..100 {
            let value = 50.0 + (i as f64).sin() * 10.0;
            aggregator.record(
                "test_metric",
                value,
                HashMap::new(),
            ).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Get aggregated metrics
        let agg = aggregator.aggregate("test_metric", Duration::from_secs(10)).await;
        assert!(agg.is_some());
        
        let agg = agg.unwrap();
        assert!(agg.count > 0);
        assert!(agg.min <= agg.avg);
        assert!(agg.avg <= agg.max);
    }
    
    #[tokio::test]
    async fn test_trend_analysis() {
        let aggregator = MetricsAggregator::new(
            Duration::from_secs(60),
            Duration::from_secs(3600),
        );
        
        // Create upward trend
        for i in 0..20 {
            aggregator.record(
                "trending_metric",
                i as f64 * 2.0 + 10.0,
                HashMap::new(),
            ).await;
        }
        
        let trend = aggregator.analyze_trend("trending_metric").await;
        assert!(trend.is_some());
        
        let trend = trend.unwrap();
        assert!(matches!(trend.trend_direction, TrendDirection::Rising));
        assert!(trend.change_percent > 0.0);
    }
}