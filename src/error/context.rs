//! Error context and span information

/// Additional context information for error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The file path where the error occurred
    pub file_path: Option<String>,
    /// A snippet of source code around the error location
    pub source_snippet: Option<String>,
    /// Suggested fixes or improvements
    pub suggestions: Vec<String>,
}

/// A span representing a range in the source code
#[derive(Debug, Clone)]
pub struct ErrorSpan {
    /// The starting position of the span
    pub start: Position,
    /// The ending position of the span
    pub end: Position,
}

/// A position in source code with line and column information
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}