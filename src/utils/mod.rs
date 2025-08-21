//! Utility functions and helpers for the Nix parser

pub mod text;
pub mod position;
pub mod validation;
pub mod conversion;

pub use self::text::{TextUtils, LineInfo};
pub use self::position::{Position, Range, SourceLocation};
pub use self::validation::{Validator, ValidationRule};
pub use self::conversion::{TreeSitterExt, NodeExt};
pub use self::perf::{Timer, TimingResult, MemoryStats};

/// Common constants used throughout the parser
pub mod constants {
    /// Maximum reasonable nesting depth for Nix expressions
    pub const MAX_NESTING_DEPTH: usize = 1000;
    
    /// Maximum size for cached parse results (in bytes)
    pub const MAX_CACHE_SIZE: usize = 100 * 1024 * 1024; // 100MB
    
    /// Default timeout for parsing operations (in milliseconds)
    pub const DEFAULT_TIMEOUT_MS: u64 = 30_000; // 30 seconds
    
    /// Supported Nix file extensions
    pub const NIX_EXTENSIONS: &[&str] = &[".nix"];
    
    /// Keywords in the Nix language
    pub const NIX_KEYWORDS: &[&str] = &[
        "assert", "else", "if", "in", "inherit", "let", "or", "rec", "then", "with"
    ];
    
    /// Built-in operators
    pub const NIX_OPERATORS: &[&str] = &[
        "+", "-", "*", "/", "++", "//", "==", "!=", "<", ">", "<=", ">=", 
        "&&", "||", "->", "!", "?"
    ];
}

/// Memory and performance utilities
pub mod perf {
    use std::time::{Duration, Instant};
    
    /// Simple timer for measuring parsing performance
    #[derive(Debug)]
    pub struct Timer {
        start: Instant,
        label: String,
    }
    
    impl Timer {
        /// Start a new timer with a label
        pub fn start(label: impl Into<String>) -> Self {
            Self {
                start: Instant::now(),
                label: label.into(),
            }
        }
        
        /// Get elapsed time without stopping the timer
        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }
        
        /// Get the timer's label
        pub fn label(&self) -> &str {
            &self.label
        }
        
        /// Stop the timer and return elapsed time with timing info
        pub fn stop(self) -> TimingResult {
            let elapsed = self.elapsed();
            TimingResult {
                label: self.label,
                duration: elapsed,
            }
        }
        
        /// Stop the timer and return just the elapsed time
        pub fn stop_simple(self) -> Duration {
            self.elapsed()
        }
    }
    
    /// Result of a timing measurement
    #[derive(Debug, Clone)]
    pub struct TimingResult {
        /// Label identifying what was timed
        pub label: String,
        /// Duration of the timed operation
        pub duration: Duration,
    }
    
    impl TimingResult {
        /// Format as a human-readable string
        pub fn format(&self) -> String {
            format!("Timer '{}': {:?}", self.label, self.duration)
        }
        
        /// Get duration in milliseconds
        pub fn duration_ms(&self) -> u64 {
            self.duration.as_millis() as u64
        }
    }
    
    impl std::fmt::Display for TimingResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.format())
        }
    }
    
    /// Memory usage statistics
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MemoryStats {
        /// Peak memory usage in bytes
        pub peak_bytes: usize,
        /// Current memory usage in bytes
        pub current_bytes: usize,
        /// Number of allocations
        pub allocations: usize,
    }
    
    impl MemoryStats {
        /// Create empty memory stats
        pub const fn new() -> Self {
            Self {
                peak_bytes: 0,
                current_bytes: 0,
                allocations: 0,
            }
        }
    }
    
    impl Default for MemoryStats {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// String manipulation utilities specific to Nix
pub mod string {
    /// Check if a string is a valid Nix identifier
    pub fn is_valid_identifier(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        
        let mut chars = s.chars();
        let first = chars.next().unwrap();
        
        // First character must be letter or underscore
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
        
        // Remaining characters can be letters, digits, underscore, apostrophe, or hyphen
        chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '\'' | '-'))
    }
    
    /// Escape a string for use in Nix code
    pub fn escape_nix_string(s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 20);
        
        for c in s.chars() {
            match c {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '$' => result.push_str("\\$"),
                c => result.push(c),
            }
        }
        
        result
    }
    
    /// Unescape a Nix string literal
    pub fn unescape_nix_string(s: &str) -> Result<String, String> {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars();
        
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('$') => result.push('$'),
                    Some(other) => return Err(format!("Invalid escape sequence: \\{}", other)),
                    None => return Err("Unterminated escape sequence".to_string()),
                }
            } else {
                result.push(c);
            }
        }
        
        Ok(result)
    }
    
    /// Check if a string needs to be quoted as a Nix string
    pub fn needs_quoting(s: &str) -> bool {
        !is_valid_identifier(s) || crate::utils::constants::NIX_KEYWORDS.contains(&s)
    }
}

/// File system utilities
pub mod fs {
    use std::path::{Path, PathBuf};
    use crate::utils::constants::NIX_EXTENSIONS;
    
    /// Check if a file has a Nix extension
    pub fn is_nix_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| NIX_EXTENSIONS.iter().any(|nix_ext| nix_ext == &format!(".{}", ext)))
            .unwrap_or(false)
    }
    
    /// Find all Nix files in a directory recursively
    pub fn find_nix_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        let mut nix_files = Vec::new();
        find_nix_files_recursive(dir, &mut nix_files)?;
        Ok(nix_files)
    }
    
    fn find_nix_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                find_nix_files_recursive(&path, files)?;
            } else if is_nix_file(&path) {
                files.push(path);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identifier_validation() {
        assert!(string::is_valid_identifier("foo"));
        assert!(string::is_valid_identifier("_private"));
        assert!(string::is_valid_identifier("var123"));
        assert!(string::is_valid_identifier("foo-bar"));
        assert!(string::is_valid_identifier("foo'"));
        
        assert!(!string::is_valid_identifier("123var"));
        assert!(!string::is_valid_identifier(""));
        assert!(!string::is_valid_identifier("foo.bar"));
        assert!(!string::is_valid_identifier("foo@bar"));
    }
    
    #[test]
    fn test_string_escaping() {
        assert_eq!(string::escape_nix_string("hello"), "hello");
        assert_eq!(string::escape_nix_string("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(string::escape_nix_string("line1\nline2"), "line1\\nline2");
        assert_eq!(string::escape_nix_string("${var}"), "\\${var}");
    }
    
    #[test]
    fn test_string_unescaping() {
        assert_eq!(string::unescape_nix_string("hello").unwrap(), "hello");
        assert_eq!(string::unescape_nix_string("hello \\\"world\\\"").unwrap(), "hello \"world\"");
        assert_eq!(string::unescape_nix_string("line1\\nline2").unwrap(), "line1\nline2");
        assert_eq!(string::unescape_nix_string("\\${var}").unwrap(), "${var}");
        
        assert!(string::unescape_nix_string("invalid\\x").is_err());
        assert!(string::unescape_nix_string("incomplete\\").is_err());
    }
    
    #[test]
    fn test_needs_quoting() {
        assert!(!string::needs_quoting("foo"));
        assert!(!string::needs_quoting("_private"));
        assert!(string::needs_quoting("let")); // keyword
        assert!(string::needs_quoting("foo.bar")); // contains dot
        assert!(string::needs_quoting("123var")); // starts with digit
    }
    
    #[test]
    fn test_nix_file_detection() {
        use std::path::Path;
        
        assert!(fs::is_nix_file(Path::new("foo.nix")));
        assert!(!fs::is_nix_file(Path::new("foo.txt")));
        assert!(!fs::is_nix_file(Path::new("foo")));
    }
    
    #[test]
    fn test_timer() {
        use std::thread;
        use std::time::Duration;
        
        let timer = perf::Timer::start("test timer");
        assert_eq!(timer.label(), "test timer");
        
        thread::sleep(Duration::from_millis(10));
        let result = timer.stop();
        
        assert_eq!(result.label, "test timer");
        assert!(result.duration >= Duration::from_millis(10));
        assert!(result.duration_ms() >= 10);
        
        // Test display formatting
        let formatted = result.format();
        assert!(formatted.contains("test timer"));
        assert!(formatted.contains("ms") || formatted.contains("µs"));
    }
}