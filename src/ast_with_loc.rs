use serde::{Deserialize, Serialize};
use crate::ast::*;

/// Expression with source location information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocatedExpression {
    pub expr: Expression,
    pub loc: SourceLocation,
}

impl LocatedExpression {
    pub fn new(expr: Expression, loc: SourceLocation) -> Self {
        Self { expr, loc }
    }
    
    pub fn from_node(expr: Expression, node: tree_sitter::Node, source: &str) -> Self {
        let loc = SourceLocation {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_position: (node.start_position().row, node.start_position().column),
            end_position: (node.end_position().row, node.end_position().column),
        };
        Self { expr, loc }
    }
    
    /// Get the source text for this expression
    pub fn source_text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.loc.start_byte..self.loc.end_byte]
    }
}

/// Enhanced parser result with location information
pub struct ParseResultWithLocation {
    pub tree: tree_sitter::Tree,
    pub ast: Option<LocatedExpression>,
    pub source: String,
    pub diagnostics: Vec<Diagnostic>,
}

/// Diagnostic with location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: SourceLocation,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            location,
            suggestion: None,
        }
    }
    
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}