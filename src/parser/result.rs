//! Parser result types and diagnostic information

use std::fmt;
use tree_sitter::{Tree, Node};

use crate::ast::{Expression, SourceLocation};
use crate::error::Result;

/// Result of a parsing operation
///
/// Contains the parsed tree, source information, and any diagnostics
/// generated during parsing.
#[derive(Debug, Clone)]
pub struct ParseResult {
    tree: Tree,
    source: String,
    diagnostics: Vec<ParseDiagnostic>,
    statistics: Option<ParseStats>,
}

impl ParseResult {
    /// Create a new `ParseResult` from a Tree-sitter tree
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the tree cannot be processed or contains
    /// critical errors.
    pub fn from_tree(tree: Tree, source: String) -> Result<Self> {
        let mut diagnostics = Vec::new();
        
        // Collect syntax errors from the tree
        if tree.root_node().has_error() {
            Self::collect_errors(&tree.root_node(), &source, &mut diagnostics);
        }
        
        Ok(Self {
            tree,
            source,
            diagnostics,
            statistics: None,
        })
    }
    
    /// Get the underlying Tree-sitter tree
    pub const fn tree(&self) -> &Tree {
        &self.tree
    }
    
    /// Get the source code that was parsed
    pub fn source(&self) -> &str {
        &self.source
    }
    
    /// Get all parse diagnostics
    pub fn diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }
    
    /// Check if parsing resulted in any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Error)
    }
    
    /// Check if parsing resulted in any warnings
    pub fn has_warnings(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Warning)
    }
    
    /// Get the root expression from the parse tree
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the tree structure is invalid or the
    /// root expression cannot be converted to the AST representation.
    pub fn expression(&self) -> Result<Option<Expression>> {
        let root = self.tree.root_node();
        
        // Look for the expression field in the source_file node
        if let Some(expr_node) = root.child_by_field_name("expression") {
            Expression::from_tree_sitter_node(expr_node, &self.source)
                .map(Some)
        } else {
            Ok(None)
        }
    }
    
    /// Get detailed error information
    pub fn error_summary(&self) -> Option<String> {
        if !self.has_errors() {
            return None;
        }
        
        let errors: Vec<_> = self.diagnostics.iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect();
            
        if errors.is_empty() {
            return None;
        }
        
        let mut summary = String::new();
        summary.push_str(&format!("Found {} parse error(s):\n", errors.len()));
        
        for error in errors {
            summary.push_str(&format!("  - Line {}, Column {}: {}\n", 
                                     error.location.line, 
                                     error.location.column, 
                                     error.message));
        }
        
        Some(summary)
    }
    
    /// Add a diagnostic to the result
    pub fn add_diagnostic(&mut self, diagnostic: ParseDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
    
    /// Get parsing statistics if available
    pub fn statistics(&self) -> Option<&ParseStats> {
        self.statistics.as_ref()
    }
    
    /// Set parsing statistics
    pub fn set_statistics(&mut self, statistics: Option<ParseStats>) {
        self.statistics = statistics;
    }
    
    // Private helper methods
    
    fn collect_errors(node: &Node, source: &str, diagnostics: &mut Vec<ParseDiagnostic>) {
        if node.is_error() {
            let location = SourceLocation::from_tree_sitter_node(node);
            let text = node.utf8_text(source.as_bytes())
                .unwrap_or("<invalid UTF-8>")
                .to_string();
                
            diagnostics.push(ParseDiagnostic {
                severity: DiagnosticSeverity::Error,
                location,
                message: format!("Syntax error near: '{}'", text),
                code: Some("syntax_error".to_string()),
                source: Some("nix-parser".to_string()),
            });
        }
        
        // Check for missing nodes (Tree-sitter represents these specially)
        if node.is_missing() {
            let location = SourceLocation::from_tree_sitter_node(node);
            diagnostics.push(ParseDiagnostic {
                severity: DiagnosticSeverity::Error,
                location,
                message: format!("Missing: {}", node.kind()),
                code: Some("missing_node".to_string()),
                source: Some("nix-parser".to_string()),
            });
        }
        
        // Recursively check children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                Self::collect_errors(&child, source, diagnostics);
            }
        }
    }
}

/// A diagnostic message from parsing
///
/// Represents errors, warnings, and informational messages
/// generated during the parsing process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    /// Severity level of the diagnostic
    pub severity: DiagnosticSeverity,
    
    /// Location in the source code
    pub location: SourceLocation,
    
    /// Human-readable message
    pub message: String,
    
    /// Optional diagnostic code
    pub code: Option<String>,
    
    /// Source of the diagnostic (e.g., "nix-parser", "plugin-name")
    pub source: Option<String>,
}

impl ParseDiagnostic {
    /// Create a new error diagnostic
    pub fn error(location: SourceLocation, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            location,
            message: message.into(),
            code: None,
            source: Some("nix-parser".to_string()),
        }
    }
    
    /// Create a new warning diagnostic
    pub fn warning(location: SourceLocation, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            location,
            message: message.into(),
            code: None,
            source: Some("nix-parser".to_string()),
        }
    }
    
    /// Create a new info diagnostic
    pub fn info(location: SourceLocation, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            location,
            message: message.into(),
            code: None,
            source: Some("nix-parser".to_string()),
        }
    }
    
    /// Set the diagnostic code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
    
    /// Set the diagnostic source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

impl fmt::Display for ParseDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} at {}:{}", 
               self.severity,
               self.message,
               self.location.line,
               self.location.column)
    }
}

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    /// Informational message
    Info,
    /// Warning that doesn't prevent parsing
    Warning,
    /// Error that indicates invalid syntax
    Error,
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticSeverity::Info => write!(f, "info"),
            DiagnosticSeverity::Warning => write!(f, "warning"),
            DiagnosticSeverity::Error => write!(f, "error"),
        }
    }
}

/// Statistics about a parse result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseStats {
    /// Number of nodes in the parse tree
    pub node_count: usize,
    
    /// Parse time in milliseconds
    pub parse_time_ms: u64,
    
    /// Number of errors found
    pub error_count: usize,
    
    /// Number of warnings found
    pub warning_count: usize,
    
    /// Size of the source code in bytes
    pub source_size: usize,
    
    /// Whether incremental parsing was used
    pub incremental: bool,
}

impl ParseStats {
    /// Create statistics from a parse result
    pub fn from_result(result: &ParseResult, parse_time_ms: u64, incremental: bool) -> Self {
        let node_count = Self::count_nodes(&result.tree.root_node());
        let error_count = result.diagnostics.iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count();
        let warning_count = result.diagnostics.iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count();
            
        Self {
            node_count,
            parse_time_ms,
            error_count,
            warning_count,
            source_size: result.source.len(),
            incremental,
        }
    }
    
    fn count_nodes(node: &Node) -> usize {
        let mut count = 1;
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                count += Self::count_nodes(&child);
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::{Parser, Language};

    extern "C" {
        fn tree_sitter_nix() -> Language;
    }

    fn create_test_parser() -> Parser {
        let language = unsafe { tree_sitter_nix() };
        let mut parser = Parser::new();
        parser.set_language(&language).unwrap();
        parser
    }

    #[test]
    fn test_successful_parse_result() {
        let mut parser = create_test_parser();
        let tree = parser.parse("42", None).unwrap();
        let result = ParseResult::from_tree(tree, "42".to_string()).unwrap();
        
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
        assert_eq!(result.source(), "42");
    }

    #[test]
    fn test_error_parse_result() {
        let mut parser = create_test_parser();
        let tree = parser.parse("if true then", None).unwrap();
        let result = ParseResult::from_tree(tree, "if true then".to_string()).unwrap();
        
        assert!(result.has_errors());
        assert!(result.error_summary().is_some());
    }

    #[test]
    fn test_diagnostic_creation() {
        let location = SourceLocation::new(1, 5, 0, 5);
        let diag = ParseDiagnostic::error(location, "Test error")
            .with_code("E001")
            .with_source("test");
            
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.message, "Test error");
        assert_eq!(diag.code, Some("E001".to_string()));
        assert_eq!(diag.source, Some("test".to_string()));
    }

    #[test]
    fn test_parse_stats() {
        let mut parser = create_test_parser();
        let tree = parser.parse("{ x = 1; }", None).unwrap();
        let result = ParseResult::from_tree(tree, "{ x = 1; }".to_string()).unwrap();
        
        let stats = ParseStats::from_result(&result, 100, false);
        assert!(stats.node_count > 0);
        assert_eq!(stats.parse_time_ms, 100);
        assert_eq!(stats.source_size, 10);
        assert!(!stats.incremental);
    }
}