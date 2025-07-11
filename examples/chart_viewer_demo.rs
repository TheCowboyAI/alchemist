//! Chart Viewer Demo
//! 
//! This example demonstrates various chart types including line, bar, scatter, and area charts.

use alchemist::{
    config::AlchemistConfig,
    renderer::RendererManager,
};
use anyhow::Result;
use serde_json::json;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Chart Viewer Demo");

    // Create renderer manager
    let renderer_manager = RendererManager::new()?;
    
    // Example 1: Line Chart - Time Series Data
    let line_chart_data = json!({
        "series": [
            {
                "name": "Revenue",
                "color": [0.2, 0.6, 1.0],
                "data": [
                    {"x": 1.0, "y": 150.0},
                    {"x": 2.0, "y": 230.0},
                    {"x": 3.0, "y": 180.0},
                    {"x": 4.0, "y": 290.0},
                    {"x": 5.0, "y": 320.0},
                    {"x": 6.0, "y": 280.0},
                    {"x": 7.0, "y": 350.0},
                    {"x": 8.0, "y": 380.0},
                    {"x": 9.0, "y": 420.0},
                    {"x": 10.0, "y": 450.0},
                    {"x": 11.0, "y": 480.0},
                    {"x": 12.0, "y": 510.0},
                ]
            },
            {
                "name": "Expenses",
                "color": [1.0, 0.4, 0.2],
                "data": [
                    {"x": 1.0, "y": 120.0},
                    {"x": 2.0, "y": 140.0},
                    {"x": 3.0, "y": 160.0},
                    {"x": 4.0, "y": 180.0},
                    {"x": 5.0, "y": 200.0},
                    {"x": 6.0, "y": 190.0},
                    {"x": 7.0, "y": 210.0},
                    {"x": 8.0, "y": 220.0},
                    {"x": 9.0, "y": 240.0},
                    {"x": 10.0, "y": 250.0},
                    {"x": 11.0, "y": 260.0},
                    {"x": 12.0, "y": 270.0},
                ]
            }
        ],
        "title": "Monthly Revenue vs Expenses",
        "x_label": "Month",
        "y_label": "Amount ($K)"
    });
    
    info!("Launching line chart");
    let window_id = renderer_manager.spawn_chart(
        "Financial Overview",
        line_chart_data,
        "line",
        json!({}),
    ).await?;
    info!("Launched line chart: {}", window_id);
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Example 2: Bar Chart - Category Comparison
    let bar_chart_data = json!({
        "series": [
            {
                "name": "Q1 Sales",
                "color": [0.2, 0.8, 0.4],
                "data": [
                    {"x": 1.0, "y": 45.0, "label": "Product A"},
                    {"x": 2.0, "y": 38.0, "label": "Product B"},
                    {"x": 3.0, "y": 52.0, "label": "Product C"},
                    {"x": 4.0, "y": 41.0, "label": "Product D"},
                    {"x": 5.0, "y": 35.0, "label": "Product E"},
                ]
            },
            {
                "name": "Q2 Sales",
                "color": [0.2, 0.6, 1.0],
                "data": [
                    {"x": 1.0, "y": 50.0},
                    {"x": 2.0, "y": 42.0},
                    {"x": 3.0, "y": 58.0},
                    {"x": 4.0, "y": 46.0},
                    {"x": 5.0, "y": 40.0},
                ]
            }
        ],
        "title": "Product Sales Comparison",
        "x_label": "Products",
        "y_label": "Sales (Units)"
    });
    
    info!("Launching bar chart");
    let window_id = renderer_manager.spawn_chart(
        "Sales Analysis",
        bar_chart_data,
        "bar",
        json!({}),
    ).await?;
    info!("Launched bar chart: {}", window_id);
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Example 3: Scatter Plot - Correlation Analysis
    let scatter_data = json!({
        "series": [
            {
                "name": "Dataset A",
                "color": [0.8, 0.2, 0.8],
                "data": [
                    {"x": 1.2, "y": 2.4},
                    {"x": 2.1, "y": 3.8},
                    {"x": 3.3, "y": 5.2},
                    {"x": 4.0, "y": 7.1},
                    {"x": 5.2, "y": 8.9},
                    {"x": 6.1, "y": 10.2},
                    {"x": 7.3, "y": 12.1},
                    {"x": 8.0, "y": 13.8},
                    {"x": 9.1, "y": 15.2},
                    {"x": 10.2, "y": 16.9},
                ]
            },
            {
                "name": "Dataset B",
                "color": [1.0, 0.8, 0.2],
                "data": [
                    {"x": 1.5, "y": 3.2},
                    {"x": 2.3, "y": 4.1},
                    {"x": 3.1, "y": 4.8},
                    {"x": 4.2, "y": 6.2},
                    {"x": 5.0, "y": 7.8},
                    {"x": 6.3, "y": 9.1},
                    {"x": 7.1, "y": 11.2},
                    {"x": 8.2, "y": 12.8},
                    {"x": 9.0, "y": 14.1},
                    {"x": 10.1, "y": 15.8},
                ]
            }
        ],
        "title": "Correlation Analysis",
        "x_label": "Variable X",
        "y_label": "Variable Y"
    });
    
    info!("Launching scatter plot");
    let window_id = renderer_manager.spawn_chart(
        "Data Correlation",
        scatter_data,
        "scatter",
        json!({}),
    ).await?;
    info!("Launched scatter plot: {}", window_id);
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Example 4: Area Chart - Cumulative Data
    let area_chart_data = json!({
        "series": [
            {
                "name": "Memory Usage",
                "color": [0.2, 0.8, 0.8],
                "data": [
                    {"x": 0.0, "y": 20.0},
                    {"x": 1.0, "y": 25.0},
                    {"x": 2.0, "y": 30.0},
                    {"x": 3.0, "y": 35.0},
                    {"x": 4.0, "y": 45.0},
                    {"x": 5.0, "y": 55.0},
                    {"x": 6.0, "y": 60.0},
                    {"x": 7.0, "y": 58.0},
                    {"x": 8.0, "y": 55.0},
                    {"x": 9.0, "y": 50.0},
                    {"x": 10.0, "y": 45.0},
                ]
            },
            {
                "name": "CPU Usage",
                "color": [1.0, 0.4, 0.4],
                "data": [
                    {"x": 0.0, "y": 10.0},
                    {"x": 1.0, "y": 15.0},
                    {"x": 2.0, "y": 20.0},
                    {"x": 3.0, "y": 25.0},
                    {"x": 4.0, "y": 35.0},
                    {"x": 5.0, "y": 40.0},
                    {"x": 6.0, "y": 38.0},
                    {"x": 7.0, "y": 35.0},
                    {"x": 8.0, "y": 30.0},
                    {"x": 9.0, "y": 25.0},
                    {"x": 10.0, "y": 20.0},
                ]
            }
        ],
        "title": "System Resource Usage",
        "x_label": "Time (minutes)",
        "y_label": "Usage (%)"
    });
    
    info!("Launching area chart");
    let window_id = renderer_manager.spawn_chart(
        "System Monitor",
        area_chart_data,
        "area",
        json!({}),
    ).await?;
    info!("Launched area chart: {}", window_id);
    
    // Keep the main thread alive
    info!("");
    info!("Chart viewers launched. Each window supports:");
    info!("  - Toggle between chart types (Line, Bar, Scatter, Area)");
    info!("  - Toggle grid on/off");
    info!("  - Toggle legend visibility");
    info!("");
    info!("Press Ctrl+C to exit all windows.");
    
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}