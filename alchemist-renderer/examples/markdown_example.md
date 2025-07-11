# Markdown Renderer Example

This is a demonstration of the Alchemist markdown renderer with various formatting options.

## Headers

### Level 3 Header
#### Level 4 Header
##### Level 5 Header
###### Level 6 Header

## Text Formatting

This is **bold text** and this is *italic text* and this is ***bold italic text***.

You can also use `inline code` within paragraphs.

## Lists

### Unordered List
- First item
- Second item
  - Nested item 1
  - Nested item 2
- Third item

### Ordered List
1. First item
2. Second item
3. Third item

## Code Blocks

```rust
fn main() {
    println!("Hello, Alchemist!");
    let x = 42;
    let y = x * 2;
    println!("Result: {}", y);
}
```

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

print(f"Fibonacci(10) = {fibonacci(10)}")
```

## Blockquotes

> This is a blockquote. It can contain multiple lines
> and even include **formatted text** and `code`.
>
> It can also have multiple paragraphs.

## Tables

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
| Cell 7   | Cell 8   | Cell 9   |

## Links and Images

[Visit the Alchemist repository](https://github.com/thecowboyai/alchemist)

## Horizontal Rule

---

## Mixed Content

Here's a paragraph with various elements: **bold text**, *italic text*, `inline code`, and a [link](https://example.com).

> A blockquote with a code block inside:
> ```javascript
> console.log("Hello from a blockquote!");
> ```

That's the end of our markdown example!