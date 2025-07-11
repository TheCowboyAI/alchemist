# Alchemist Rendering Features

## Overview

Alchemist includes a comprehensive rendering system that supports multiple visualization types through both 3D (Bevy) and 2D (Iced) renderers. This document covers all available rendering features and how to use them.

## Supported Render Types

### 1. Dashboard
The main Alchemist dashboard showing system overview, domain status, and real-time events.

```bash
# Launch dashboard (default)
alchemist

# Launch with explicit command
alchemist render dashboard

# Launch in-process dashboard
alchemist dashboard-local
```

### 2. Markdown Documents
Full-featured markdown rendering with syntax highlighting and theme support.

```bash
# Render markdown with light theme
alchemist render markdown document.md --theme light

# Render with dark theme
alchemist render markdown document.md --theme dark
```

**Features:**
- Headers (H1-H6)
- Text formatting (bold, italic, strikethrough)
- Code blocks with syntax highlighting
- Tables with borders
- Blockquotes
- Lists (ordered and unordered)
- Links
- Inline code
- Horizontal rules

### 3. Charts
Interactive chart visualization supporting multiple chart types.

```bash
# Line chart
alchemist render chart data.json --chart-type line --title "Sales Data"

# Bar chart
alchemist render chart data.json --chart-type bar

# Pie chart
alchemist render chart data.json --chart-type pie

# Scatter plot
alchemist render chart data.json --chart-type scatter

# Area chart
alchemist render chart data.json --chart-type area
```

**Chart Data Format:**
```json
{
  "data": [
    {
      "name": "Series Name",
      "data": [
        {"x": 1.0, "y": 100.0, "label": "Jan"},
        {"x": 2.0, "y": 150.0, "label": "Feb"}
      ],
      "color": [0.2, 0.5, 1.0, 1.0]  // RGBA
    }
  ],
  "options": {
    "title": "Chart Title",
    "x_label": "X Axis",
    "y_label": "Y Axis", 
    "show_grid": true,
    "show_legend": true
  }
}
```

**Features:**
- Multiple series support
- Interactive zoom and pan
- Tooltips on hover
- Customizable colors
- Grid lines
- Legends
- Export to PNG/SVG (planned)

### 4. Dialog Windows
AI-powered dialog interface for conversations.

```bash
# Launch dialog window
alchemist render dialog

# Launch with specific dialog
alchemist dialog open dialog_123
```

### 5. 3D Graph Visualization
Interactive 3D graph rendering using Bevy.

```bash
# Render graph from file
alchemist render graph graph_data.json

# Render demo graph
alchemist render demo graph3d
```

### 6. Workflow Editor
Visual workflow creation and editing.

```bash
# Open workflow editor
alchemist render workflow my-workflow

# With 3D visualization
alchemist render workflow my-workflow --three-d
```

### 7. Event Monitor
Real-time event stream visualization.

```bash
# Launch event monitor
alchemist render events
```

### 8. Performance Monitor
System performance metrics visualization.

```bash
# Launch performance monitor
alchemist render performance
```

## Renderer Management

### List Active Renderers
```bash
alchemist render list
```

### Close a Renderer
```bash
alchemist render close <renderer-id>
```

## Architecture

The rendering system uses a hybrid architecture:

1. **Bevy Renderer** - For 3D visualizations
   - Graph visualization
   - 3D scenes
   - Performance-intensive graphics

2. **Iced Renderer** - For 2D UI
   - Dashboard
   - Markdown documents
   - Charts
   - Dialog windows
   - Forms and controls

### Communication

Renderers communicate with the shell through:
- **Direct Channels** - For in-process renderers
- **NATS Bridge** - For distributed renderers
- **Event System** - For real-time updates

## Examples

### Creating a Report with Charts

1. Create your markdown report:
```markdown
# Monthly Report

## Sales Performance

Our sales have increased by 25% this month.

*See the attached charts for details.*
```

2. Create chart data:
```json
{
  "data": [{
    "name": "Monthly Sales",
    "data": [
      {"x": 1, "y": 1000, "label": "Week 1"},
      {"x": 2, "y": 1500, "label": "Week 2"},
      {"x": 3, "y": 2000, "label": "Week 3"},
      {"x": 4, "y": 2500, "label": "Week 4"}
    ]
  }],
  "options": {
    "title": "Sales Growth",
    "y_label": "Revenue ($)"
  }
}
```

3. Render both:
```bash
alchemist render markdown report.md
alchemist render chart sales.json --chart-type line
```

### Real-time Dashboard

The dashboard automatically updates when connected to NATS:

```bash
# Start NATS
nats-server -js

# Launch Alchemist (dashboard starts automatically)
alchemist
```

## Customization

### Themes

Markdown viewer supports light and dark themes:
- Light theme: Clean, bright colors
- Dark theme: Easy on the eyes for night work

### Chart Colors

Charts support custom colors per series:
```json
"color": [0.2, 0.5, 1.0, 1.0]  // RGBA values (0.0-1.0)
```

## Performance Tips

1. **Large Documents**: The markdown renderer handles large documents efficiently with virtual scrolling
2. **Many Data Points**: Charts automatically downsample for performance
3. **Multiple Windows**: Each renderer runs in its own process for isolation

## Troubleshooting

### Renderer Won't Start
- Check if the renderer binary exists in PATH
- Ensure Iced/Bevy dependencies are installed
- Check logs for specific errors

### Charts Not Displaying
- Verify JSON data format is correct
- Ensure data points have valid x,y values
- Check console for parsing errors

### Markdown Rendering Issues
- Verify file encoding is UTF-8
- Check for malformed markdown syntax
- Try with a simple test document first

## Future Enhancements

Planned features include:
- PDF export for markdown and charts
- Live markdown preview with editing
- 3D chart types
- Collaborative editing
- Custom renderer plugins