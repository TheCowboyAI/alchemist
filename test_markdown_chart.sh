#!/bin/bash
# Test script for markdown and chart rendering

echo "=== Testing Markdown and Chart Rendering ==="
echo

# Create test markdown file
cat > test_document.md << 'EOF'
# Test Markdown Document

This is a test of the Alchemist markdown renderer.

## Features

- Bullet points
- **Bold text**
- *Italic text*
- `Code snippets`

### Code Block

```rust
fn main() {
    println!("Hello from Alchemist!");
}
```

### Table

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Data 1   | Data 2   | Data 3   |
| Data 4   | Data 5   | Data 6   |

> This is a blockquote
> with multiple lines
EOF

# Create test chart data
cat > test_chart.json << 'EOF'
{
  "data": [
    {
      "name": "Series 1",
      "data": [
        {"x": 0, "y": 10},
        {"x": 1, "y": 20},
        {"x": 2, "y": 15},
        {"x": 3, "y": 25},
        {"x": 4, "y": 30}
      ],
      "color": [0.2, 0.5, 1.0, 1.0]
    },
    {
      "name": "Series 2", 
      "data": [
        {"x": 0, "y": 5},
        {"x": 1, "y": 15},
        {"x": 2, "y": 25},
        {"x": 3, "y": 20},
        {"x": 4, "y": 18}
      ],
      "color": [1.0, 0.2, 0.2, 1.0]
    }
  ],
  "options": {
    "title": "Test Chart",
    "x_label": "X Axis",
    "y_label": "Y Axis",
    "show_grid": true,
    "show_legend": true
  }
}
EOF

echo "Test files created:"
echo "  - test_document.md"
echo "  - test_chart.json"
echo

# Test markdown rendering
echo "1. Testing Markdown rendering (light theme)..."
cargo run --bin alchemist -- render markdown test_document.md --theme light &
MARKDOWN_PID=$!
sleep 2

echo
echo "2. Testing Markdown rendering (dark theme)..."
cargo run --bin alchemist -- render markdown test_document.md --theme dark &
MARKDOWN_DARK_PID=$!
sleep 2

echo
echo "3. Testing Line Chart..."
cargo run --bin alchemist -- render chart test_chart.json --chart-type line --title "Line Chart Test" &
LINE_PID=$!
sleep 2

echo
echo "4. Testing Bar Chart..."
cargo run --bin alchemist -- render chart test_chart.json --chart-type bar --title "Bar Chart Test" &
BAR_PID=$!
sleep 2

echo
echo "5. Testing Area Chart..."
cargo run --bin alchemist -- render chart test_chart.json --chart-type area --title "Area Chart Test" &
AREA_PID=$!
sleep 2

echo
echo "All renderer windows should now be open!"
echo "Press Enter to close all windows and clean up..."
read

# Clean up
kill $MARKDOWN_PID $MARKDOWN_DARK_PID $LINE_PID $BAR_PID $AREA_PID 2>/dev/null
rm -f test_document.md test_chart.json

echo "Test complete!"