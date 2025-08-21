//! Position and location utilities

/// A position in source code with line and column information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

/// A range representing a span in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// Starting position of the range
    pub start: Position,
    /// Ending position of the range
    pub end: Position,
}

// Re-export from ast module to avoid duplication
pub use crate::ast::SourceLocation;