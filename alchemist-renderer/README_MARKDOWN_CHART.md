# Markdown and Chart Renderers for Alchemist

This document describes the new Markdown and Chart renderer implementations for the Alchemist project using the Iced GUI framework.

## Overview

The Alchemist renderer now supports two new view types:
- **Markdown Viewer**: Renders markdown content with full formatting support
- **Chart Viewer**: Displays various chart types with interactive features

## Markdown Viewer

### Features

- Full markdown syntax support including:
  - Headers (H1-H6)
  - Bold, italic, and inline code formatting
  - Code blocks with syntax highlighting
  - Lists (ordered and unordered)
  - Tables
  - Blockquotes
  - Horizontal rules
  - Links (clickable in future versions)
- Theme support (light and dark modes)
- Scrollable content
- Responsive layout

### Usage

#### From the Alchemist Shell

```bash
# View a markdown file
alchemist> render markdown path/to/file.md

# View with dark theme
alchemist> render markdown path/to/file.md --theme dark

# Run the demo
alchemist> render demo markdown
```

#### Programmatic Usage

```rust
use alchemist::renderer::{RendererManager, RenderData};

let renderer_manager = RendererManager::new();

// Spawn a markdown viewer
let id = renderer_manager.spawn_markdown(
    "My Document",
    markdown_content,
    Some("light") // or "dark"
).await?;
```

### Theme Configuration

The markdown viewer supports two built-in themes:

```rust
// Light theme (default)
MarkdownTheme::default()

// Dark theme
MarkdownTheme::dark()
```

Custom themes can be created by implementing the `MarkdownTheme` struct.

## Chart Viewer

### Features

- Multiple chart types:
  - Line charts
  - Bar charts
  - Scatter plots
  - Pie charts
  - Area charts
- Interactive controls:
  - Zoom and pan
  - Toggle grid and legend
  - Export to PNG/SVG (planned)
- Real-time data updates
- Customizable colors and styles
- Responsive design

### Usage

#### From the Alchemist Shell

```bash
# View a chart from JSON data
alchemist> render chart path/to/data.json

# Specify chart type
alchemist> render chart data.json --chart-type bar

# With custom title
alchemist> render chart data.json --title "Sales Report"

# Run the demo
alchemist> render demo chart
```

#### Programmatic Usage

```rust
use alchemist::renderer::{RendererManager, RenderData};
use serde_json::json;

let renderer_manager = RendererManager::new();

// Prepare chart data
let data = json!([
    {
        "name": "Series 1",
        "data": [
            {"x": 0, "y": 10, "label": "Jan"},
            {"x": 1, "y": 20, "label": "Feb"},
            {"x": 2, "y": 15, "label": "Mar"}
        ],
        "color": [0.12, 0.47, 0.71, 1.0]
    }
]);

let options = json!({
    "title": "Monthly Sales",
    "x_label": "Month",
    "y_label": "Sales ($)",
    "show_grid": true,
    "show_legend": true
});

// Spawn a chart viewer
let id = renderer_manager.spawn_chart(
    "Sales Chart",
    data,
    "line",
    options
).await?;
```

### Chart Data Format

Chart data should be provided in JSON format:

```json
{
  "data": [
    {
      "name": "Series Name",
      "data": [
        {"x": 0, "y": 10, "label": "Point 1"},
        {"x": 1, "y": 20, "label": "Point 2"}
      ],
      "color": [0.12, 0.47, 0.71, 1.0]  // RGBA
    }
  ],
  "chart_type": "line",
  "options": {
    "title": "Chart Title",
    "x_label": "X Axis",
    "y_label": "Y Axis",
    "show_grid": true,
    "show_legend": true,
    "interactive": true,
    "animation_duration": 0.5
  }
}
```

### Chart Types

1. **Line Chart**: Connect data points with lines
2. **Bar Chart**: Display data as vertical bars
3. **Scatter Plot**: Show individual data points
4. **Pie Chart**: Display data as pie slices (uses y-values as slice sizes)
5. **Area Chart**: Fill area under line chart

## Examples

Example files are provided in `alchemist-renderer/examples/`:

- `markdown_example.md`: Comprehensive markdown formatting examples
- `chart_example.json`: Line chart example
- `bar_chart_example.json`: Bar chart example
- `pie_chart_example.json`: Pie chart example

## Implementation Details

### Architecture

Both renderers are implemented as Iced applications that integrate with the Alchemist renderer infrastructure:

1. **MarkdownView**: Parses markdown using `pulldown-cmark` and renders using Iced widgets
2. **ChartView**: Uses Iced's Canvas API for custom chart rendering

### Dependencies

- `iced`: GUI framework
- `pulldown-cmark`: Markdown parsing
- `serde_json`: JSON data handling

### Future Enhancements

1. **Markdown Viewer**:
   - Syntax highlighting for code blocks
   - Image loading and display
   - Link navigation
   - Export to PDF/HTML
   - Custom CSS styling

2. **Chart Viewer**:
   - More chart types (radar, heatmap, etc.)
   - Animation support
   - Data point tooltips
   - Export functionality
   - Real-time data streaming
   - Advanced interactivity (selection, filtering)

## Testing

Run the test script to verify the renderers:

```bash
./test_renderers.sh
```

This will test:
- Demo modes for both renderers
- File-based rendering
- Different chart types
- Theme switching

## Troubleshooting

### Common Issues

1. **Markdown not rendering properly**:
   - Check file encoding (UTF-8 required)
   - Verify markdown syntax
   - Try with a simple test file first

2. **Chart not displaying**:
   - Validate JSON format
   - Ensure data arrays are not empty
   - Check color values are in [0.0, 1.0] range

3. **Window not opening**:
   - Verify X11/Wayland support
   - Check renderer binary is built
   - Look for error messages in console

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo run -- render markdown file.md
```

## Contributing

When adding new features:

1. Update the respective view implementation
2. Add new message types if needed
3. Update the demo content
4. Add tests and examples
5. Update this documentation

## License

Part of the Alchemist project - see main LICENSE file.