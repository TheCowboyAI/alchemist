//! Markdown Viewer Demo
//! 
//! This example demonstrates the markdown renderer by displaying various
//! markdown documents including the project documentation.

use alchemist::{
    config::AlchemistConfig,
    renderer::RendererManager,
};
use anyhow::Result;
use std::fs;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Markdown Viewer Demo");

    // Create renderer manager
    let renderer_manager = RendererManager::new()?;
    
    // Example 1: Display README
    if let Ok(readme_content) = fs::read_to_string("readme.md") {
        info!("Displaying readme.md");
        let window_id = renderer_manager.spawn_markdown(
            "Project README",
            readme_content,
            Some("dark"),
        ).await?;
        info!("Launched README viewer: {}", window_id);
    }
    
    // Example 2: Display a sample markdown document
    let sample_markdown = r#"# Alchemist Markdown Viewer Demo

This is a demonstration of the **Alchemist Markdown Viewer**.

## Features

The markdown viewer supports:

- **Bold text** and *italic text*
- `Inline code` snippets
- Code blocks with syntax highlighting

```rust
fn main() {
    println!("Hello from Alchemist!");
}
```

### Lists

#### Unordered Lists
- First item
- Second item
  - Nested item
  - Another nested item
- Third item

#### Ordered Lists
1. First step
2. Second step
3. Third step

### Blockquotes

> This is a blockquote. It's useful for highlighting important information
> or quoting external sources.

### Code Examples

Here's a Python example:

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
```

And a JavaScript example:

```javascript
const greet = (name) => {
    console.log(`Hello, ${name}!`);
};
```

---

## Tables (Rendered as text for now)

| Feature | Status | Priority |
|---------|--------|----------|
| Markdown Parsing | ✅ Complete | High |
| Syntax Highlighting | 🚧 In Progress | Medium |
| Live Preview | ⏳ Planned | Low |

## Links and Images

- [Alchemist Documentation](https://example.com/docs)
- [GitHub Repository](https://github.com/example/alchemist)

![Example Image](https://example.com/image.png)

## Advanced Features

### Math Support (Future)
When implemented, we'll support LaTeX math:
- Inline math: $E = mc^2$
- Block math:
$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$

### Mermaid Diagrams (Future)
```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Do Something]
    B -->|No| D[Do Something Else]
    C --> E[End]
    D --> E
```

---

*Thank you for using Alchemist!*
"#;
    
    // Wait a moment for first window to open
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    info!("Displaying sample markdown document");
    let window_id = renderer_manager.spawn_markdown(
        "Markdown Feature Demo",
        sample_markdown.to_string(),
        Some("dark"),
    ).await?;
    info!("Launched sample markdown viewer: {}", window_id);
    
    // Example 3: Display with light theme
    let light_theme_content = r#"# Light Theme Example

This markdown is displayed with a **light theme**.

## Why Light Theme?

Some users prefer light themes for:
- Better readability in bright environments
- Reduced eye strain during daytime
- Traditional document appearance

### Code Example

```rust
// This code block should have light theme styling
struct LightTheme {
    background: Color::WHITE,
    foreground: Color::BLACK,
}
```

> Light themes can be easier to read when printing documents.
"#;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    info!("Displaying light theme example");
    let window_id = renderer_manager.spawn_markdown(
        "Light Theme Demo",
        light_theme_content.to_string(),
        Some("light"),
    ).await?;
    info!("Launched light theme viewer: {}", window_id);
    
    // Keep the main thread alive
    info!("Markdown viewers launched. Press Ctrl+C to exit all windows.");
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}