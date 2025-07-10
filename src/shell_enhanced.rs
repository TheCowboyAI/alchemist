//! Enhanced interactive shell with tab completion and syntax highlighting

use anyhow::Result;
use console::{Style, Term};
use std::io::{self, Write};
use std::collections::HashMap;

pub(crate) struct CompletionEngine {
    commands: HashMap<String, Vec<String>>,
    current_input: String,
    cursor_position: usize,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl CompletionEngine {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        
        // Define command hierarchy for completion
        commands.insert("".to_string(), vec![
            "help".to_string(),
            "status".to_string(),
            "progress".to_string(),
            "ai".to_string(),
            "dialog".to_string(),
            "policy".to_string(),
            "domain".to_string(),
            "deploy".to_string(),
            "render".to_string(),
            "dashboard".to_string(),
            "clear".to_string(),
            "exit".to_string(),
            "quit".to_string(),
        ]);
        
        commands.insert("ai".to_string(), vec![
            "list".to_string(),
            "add".to_string(),
            "remove".to_string(),
            "test".to_string(),
            "config".to_string(),
        ]);
        
        commands.insert("dialog".to_string(), vec![
            "new".to_string(),
            "list".to_string(),
            "continue".to_string(),
            "export".to_string(),
            "delete".to_string(),
        ]);
        
        commands.insert("policy".to_string(), vec![
            "list".to_string(),
            "show".to_string(),
            "edit".to_string(),
            "new".to_string(),
            "evaluate".to_string(),
            "export".to_string(),
        ]);
        
        commands.insert("domain".to_string(), vec![
            "list".to_string(),
            "tree".to_string(),
            "show".to_string(),
            "graph".to_string(),
            "enable".to_string(),
            "disable".to_string(),
        ]);
        
        commands.insert("deploy".to_string(), vec![
            "list".to_string(),
            "deploy".to_string(),
            "status".to_string(),
            "rollback".to_string(),
            "logs".to_string(),
        ]);
        
        commands.insert("render".to_string(), vec![
            "graph".to_string(),
            "document".to_string(),
            "edit".to_string(),
            "list".to_string(),
            "close".to_string(),
            "demo".to_string(),
        ]);
        
        Self {
            commands,
            current_input: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            history_index: None,
        }
    }
    
    pub fn get_completions(&self, input: &str) -> Vec<String> {
        // Check if input ends with a space - this indicates we want subcommand completion
        let ends_with_space = input.ends_with(' ');
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() && !ends_with_space {
            // Complete root commands
            return self.commands.get("")
                .map(|cmds| cmds.clone())
                .unwrap_or_default();
        }
        
        if parts.len() == 1 && !ends_with_space {
            // Complete root command
            let prefix = parts[0];
            return self.commands.get("")
                .map(|cmds| {
                    cmds.iter()
                        .filter(|cmd| cmd.starts_with(prefix))
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
        }
        
        if parts.len() == 1 && ends_with_space {
            // Show subcommands for the root command
            let root_cmd = parts[0];
            return self.commands.get(root_cmd)
                .map(|cmds| {
                    cmds.iter()
                        .map(|cmd| format!("{} {}", root_cmd, cmd))
                        .collect()
                })
                .unwrap_or_default();
        }
        
        if parts.len() == 2 {
            // Complete subcommand
            let root_cmd = parts[0];
            let prefix = parts[1];
            
            return self.commands.get(root_cmd)
                .map(|cmds| {
                    cmds.iter()
                        .filter(|cmd| cmd.starts_with(prefix))
                        .map(|cmd| format!("{} {}", root_cmd, cmd))
                        .collect()
                })
                .unwrap_or_default();
        }
        
        Vec::new()
    }
    
    pub fn add_to_history(&mut self, command: String) {
        if !command.trim().is_empty() && 
           (self.history.is_empty() || self.history.last() != Some(&command)) {
            self.history.push(command);
        }
        self.history_index = None;
    }
    
    pub fn previous_history(&mut self) -> Option<&str> {
        if self.history.is_empty() {
            return None;
        }
        
        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
            }
            _ => {}
        }
        
        self.history_index.and_then(|idx| self.history.get(idx).map(|s| s.as_str()))
    }
    
    pub fn next_history(&mut self) -> Option<&str> {
        match self.history_index {
            Some(idx) if idx < self.history.len() - 1 => {
                self.history_index = Some(idx + 1);
                self.history.get(idx + 1).map(|s| s.as_str())
            }
            Some(_) => {
                self.history_index = None;
                None
            }
            None => None,
        }
    }
}

pub(crate) struct SyntaxHighlighter {
    keyword_style: Style,
    command_style: Style,
    string_style: Style,
    number_style: Style,
    error_style: Style,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            keyword_style: Style::new().cyan().bold(),
            command_style: Style::new().green().bold(),
            string_style: Style::new().yellow(),
            number_style: Style::new().magenta(),
            error_style: Style::new().red(),
        }
    }
    
    pub fn highlight(&self, input: &str) -> String {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return input.to_string();
        }
        
        let mut result = Vec::new();
        
        // First part is always a command
        if let Some(cmd) = parts.first() {
            let styled = if self.is_valid_command(cmd) {
                self.command_style.apply_to(cmd).to_string()
            } else {
                self.error_style.apply_to(cmd).to_string()
            };
            result.push(styled);
        }
        
        // Process remaining parts
        for part in parts.iter().skip(1) {
            let styled = if part.starts_with('"') && part.ends_with('"') {
                self.string_style.apply_to(part).to_string()
            } else if part.starts_with('\'') && part.ends_with('\'') {
                self.string_style.apply_to(part).to_string()
            } else if part.parse::<i64>().is_ok() || part.parse::<f64>().is_ok() {
                self.number_style.apply_to(part).to_string()
            } else if self.is_keyword(part) {
                self.keyword_style.apply_to(part).to_string()
            } else {
                part.to_string()
            };
            result.push(styled);
        }
        
        result.join(" ")
    }
    
    fn is_valid_command(&self, cmd: &str) -> bool {
        matches!(cmd, 
            "help" | "?" | "status" | "progress" | "ai" | "dialog" | 
            "policy" | "domain" | "deploy" | "render" | "dashboard" | 
            "clear" | "exit" | "quit"
        )
    }
    
    fn is_keyword(&self, word: &str) -> bool {
        matches!(word,
            "list" | "add" | "remove" | "test" | "new" | "show" | 
            "edit" | "tree" | "graph" | "enable" | "disable" | 
            "deploy" | "rollback" | "logs" | "demo" | "export" |
            "evaluate" | "continue" | "delete" | "config"
        )
    }
}

pub(crate) struct EnhancedShell {
    completion_engine: CompletionEngine,
    syntax_highlighter: SyntaxHighlighter,
    term: Term,
    prompt: String,
}

impl EnhancedShell {
    pub fn new() -> Self {
        Self {
            completion_engine: CompletionEngine::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
            term: Term::stdout(),
            prompt: "alchemist> ".to_string(),
        }
    }
    
    pub fn read_line(&mut self) -> Result<String> {
        let mut input = String::new();
        let mut cursor_pos = 0;
        
        // Print prompt
        print!("{}", Style::new().green().apply_to(&self.prompt));
        io::stdout().flush()?;
        
        loop {
            if let Ok(key) = self.term.read_key() {
                use console::Key;
                
                match key {
                    Key::Enter => {
                        println!();
                        self.completion_engine.add_to_history(input.clone());
                        return Ok(input);
                    }
                    Key::Tab => {
                        let completions = self.completion_engine.get_completions(&input);
                        if completions.len() == 1 {
                            // Single completion - use it
                            input = completions[0].clone();
                            cursor_pos = input.len();
                            self.redraw_line(&input, cursor_pos)?;
                        } else if completions.len() > 1 {
                            // Multiple completions - show them
                            println!();
                            for completion in &completions {
                                println!("  {}", completion);
                            }
                            print!("{}", Style::new().green().apply_to(&self.prompt));
                            print!("{}", input);
                            io::stdout().flush()?;
                        }
                    }
                    Key::Char(c) => {
                        input.insert(cursor_pos, c);
                        cursor_pos += 1;
                        self.redraw_line(&input, cursor_pos)?;
                    }
                    Key::Backspace => {
                        if cursor_pos > 0 {
                            input.remove(cursor_pos - 1);
                            cursor_pos -= 1;
                            self.redraw_line(&input, cursor_pos)?;
                        }
                    }
                    Key::ArrowLeft => {
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                            self.term.move_cursor_left(1)?;
                        }
                    }
                    Key::ArrowRight => {
                        if cursor_pos < input.len() {
                            cursor_pos += 1;
                            self.term.move_cursor_right(1)?;
                        }
                    }
                    Key::ArrowUp => {
                        if let Some(prev) = self.completion_engine.previous_history() {
                            input = prev.to_string();
                            cursor_pos = input.len();
                            self.redraw_line(&input, cursor_pos)?;
                        }
                    }
                    Key::ArrowDown => {
                        if let Some(next) = self.completion_engine.next_history() {
                            input = next.to_string();
                            cursor_pos = input.len();
                            self.redraw_line(&input, cursor_pos)?;
                        } else {
                            input.clear();
                            cursor_pos = 0;
                            self.redraw_line(&input, cursor_pos)?;
                        }
                    }
                    Key::Home => {
                        cursor_pos = 0;
                        self.term.move_cursor_left(cursor_pos)?;
                    }
                    Key::End => {
                        let diff = input.len() - cursor_pos;
                        cursor_pos = input.len();
                        self.term.move_cursor_right(diff)?;
                    }
                    Key::CtrlC => {
                        println!("^C");
                        return Ok(String::new());
                    }
                    Key::Escape => {
                        // Clear current input
                        input.clear();
                        cursor_pos = 0;
                        self.redraw_line(&input, cursor_pos)?;
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn redraw_line(&self, input: &str, cursor_pos: usize) -> Result<()> {
        // Clear the current line
        self.term.clear_line()?;
        print!("\r{}", Style::new().green().apply_to(&self.prompt));
        
        // Apply syntax highlighting
        let highlighted = self.syntax_highlighter.highlight(input);
        print!("{}", highlighted);
        
        // Move cursor to correct position
        let chars_from_end = input.len() - cursor_pos;
        if chars_from_end > 0 {
            self.term.move_cursor_left(chars_from_end)?;
        }
        
        io::stdout().flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_completion() {
        let engine = CompletionEngine::new();
        
        // Test root command completion
        let completions = engine.get_completions("he");
        assert_eq!(completions, vec!["help"]);
        
        let completions = engine.get_completions("d");
        assert!(completions.contains(&"dialog".to_string()));
        assert!(completions.contains(&"domain".to_string()));
        assert!(completions.contains(&"deploy".to_string()));
        assert!(completions.contains(&"dashboard".to_string()));
        
        // Test subcommand completion
        let completions = engine.get_completions("ai ");
        assert!(completions.contains(&"ai list".to_string()));
        assert!(completions.contains(&"ai add".to_string()));
        assert!(completions.contains(&"ai test".to_string()));
        
        let completions = engine.get_completions("dialog l");
        assert_eq!(completions, vec!["dialog list"]);
        
        // Test full command - no completions
        let completions = engine.get_completions("ai list");
        // Since "ai list" is parsed as two parts, it will try to complete subcommands
        // starting with "list", which matches "list" itself
        assert_eq!(completions, vec!["ai list"]);
    }
    
    #[test]
    fn test_history_navigation() {
        let mut engine = CompletionEngine::new();
        
        // Add some commands to history
        engine.add_to_history("ai list".to_string());
        engine.add_to_history("dialog new".to_string());
        engine.add_to_history("status".to_string());
        
        // Navigate backwards
        assert_eq!(engine.previous_history(), Some("status"));
        assert_eq!(engine.previous_history(), Some("dialog new"));
        assert_eq!(engine.previous_history(), Some("ai list"));
        assert_eq!(engine.previous_history(), Some("ai list")); // Should stay at beginning
        
        // Navigate forwards
        assert_eq!(engine.next_history(), Some("dialog new"));
        assert_eq!(engine.next_history(), Some("status"));
        assert_eq!(engine.next_history(), None); // Should clear when at end
        
        // Previous after clearing should start from end again
        assert_eq!(engine.previous_history(), Some("status"));
    }
    
    #[test]
    fn test_syntax_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        
        // Test valid command highlighting
        let highlighted = highlighter.highlight("help");
        assert!(highlighted.contains("help")); // Should contain the command
        
        // Test command with arguments
        let highlighted = highlighter.highlight("ai list");
        assert!(highlighted.contains("ai")); // Command should be highlighted
        assert!(highlighted.contains("list")); // Keyword should be highlighted
        
        // Test string highlighting
        let highlighted = highlighter.highlight("dialog new \"My Dialog\"");
        assert!(highlighted.contains("dialog"));
        assert!(highlighted.contains("new"));
        assert!(highlighted.contains("\"My Dialog\"")); // String should be highlighted
        
        // Test number highlighting
        let highlighted = highlighter.highlight("render close 12345");
        assert!(highlighted.contains("render"));
        assert!(highlighted.contains("close"));
        assert!(highlighted.contains("12345")); // Number should be highlighted
    }
    
    #[test]
    fn test_no_duplicate_history() {
        let mut engine = CompletionEngine::new();
        
        engine.add_to_history("status".to_string());
        engine.add_to_history("status".to_string()); // Duplicate
        engine.add_to_history("help".to_string());
        
        // Should only have 2 entries
        assert_eq!(engine.previous_history(), Some("help"));
        assert_eq!(engine.previous_history(), Some("status"));
        assert_eq!(engine.previous_history(), Some("status")); // No duplicate
    }
    
    #[test]
    fn test_empty_command_not_in_history() {
        let mut engine = CompletionEngine::new();
        
        engine.add_to_history("".to_string());
        engine.add_to_history("   ".to_string());
        engine.add_to_history("status".to_string());
        
        // Should only have the actual command
        assert_eq!(engine.previous_history(), Some("status"));
        assert_eq!(engine.previous_history(), Some("status"));
    }
}