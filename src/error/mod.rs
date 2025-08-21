//! Error handling for the Nix parser
//!
//! This module provides comprehensive error types and utilities for handling
//! parsing errors, validation failures, and recovery strategies.

mod recovery;
mod diagnostic;
mod context;

pub use self::recovery::{RecoveryStrategy, ErrorRecovery};
pub use self::diagnostic::{Diagnostic, DiagnosticBuilder, Severity};
pub use self::context::{ErrorContext, ErrorSpan};

use thiserror::Error;

/// Result type for parser operations
pub type Result<T> = std::result::Result<T, ParseError>;

/// Main error type for parsing operations
///
/// This enum covers all possible error conditions that can occur
/// during parsing, validation, and post-processing operations.
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    /// Error during Tree-sitter language setup
    #[error("Language setup error: {0}")]
    LanguageError(String),
    
    /// General parsing failure
    #[error("Parse failed: {0}")]
    ParseFailed(String),
    
    /// Invalid or unexpected node structure
    #[error("Invalid node: {0}")]
    InvalidNode(String),
    
    /// Unknown or unsupported node type
    #[error("Unknown node type: {0}")]
    UnknownNodeType(String),
    
    /// UTF-8 encoding error
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    
    /// I/O operation error
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Plugin processing error
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    /// Validation error after parsing
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Syntax error with location information
    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        /// Line number (1-based)
        line: usize,
        /// Column number (1-based)
        column: usize,
        /// Error message
        message: String,
        /// Optional error context
        context: Option<ErrorContext>,
    },
    
    /// Semantic error (valid syntax, invalid semantics)
    #[error("Semantic error: {message}")]
    SemanticError {
        /// Error message
        message: String,
        /// Location where the error occurred
        span: Option<ErrorSpan>,
        /// Additional context
        context: Option<ErrorContext>,
    },
    
    /// Timeout during parsing
    #[error("Parsing timed out after {timeout_ms}ms")]
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} ({limit})")]
    ResourceLimitExceeded {
        /// Name of the resource
        resource: String,
        /// The limit that was exceeded
        limit: String,
    },
    
    /// Multiple related errors
    #[error("Multiple errors occurred")]
    Multiple(Vec<ParseError>),
    
    /// Feature not supported in current configuration
    #[error("Feature not supported: {feature}")]
    FeatureNotSupported {
        /// Name of the unsupported feature
        feature: String,
        /// Suggestion for enabling the feature
        suggestion: Option<String>,
    },
}

impl ParseError {
    /// Create a syntax error with location information
    pub fn syntax_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        ParseError::SyntaxError {
            line,
            column,
            message: message.into(),
            context: None,
        }
    }
    
    /// Create a syntax error with additional context
    pub fn syntax_error_with_context(
        line: usize, 
        column: usize, 
        message: impl Into<String>,
        context: ErrorContext
    ) -> Self {
        ParseError::SyntaxError {
            line,
            column,
            message: message.into(),
            context: Some(context),
        }
    }
    
    /// Create a semantic error
    pub fn semantic_error(message: impl Into<String>) -> Self {
        ParseError::SemanticError {
            message: message.into(),
            span: None,
            context: None,
        }
    }
    
    /// Create a semantic error with location
    pub fn semantic_error_at(
        message: impl Into<String>,
        span: ErrorSpan
    ) -> Self {
        ParseError::SemanticError {
            message: message.into(),
            span: Some(span),
            context: None,
        }
    }
    
    /// Create a timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        ParseError::Timeout { timeout_ms }
    }
    
    /// Create a resource limit error
    pub fn resource_limit(resource: impl Into<String>, limit: impl Into<String>) -> Self {
        ParseError::ResourceLimitExceeded {
            resource: resource.into(),
            limit: limit.into(),
        }
    }
    
    /// Create a feature not supported error
    pub fn feature_not_supported(feature: impl Into<String>) -> Self {
        ParseError::FeatureNotSupported {
            feature: feature.into(),
            suggestion: None,
        }
    }
    
    /// Create a feature not supported error with suggestion
    pub fn feature_not_supported_with_suggestion(
        feature: impl Into<String>,
        suggestion: impl Into<String>
    ) -> Self {
        ParseError::FeatureNotSupported {
            feature: feature.into(),
            suggestion: Some(suggestion.into()),
        }
    }
    
    /// Combine multiple errors into one
    pub fn combine(errors: Vec<ParseError>) -> Self {
        match errors.len() {
            0 => ParseError::ParseFailed("No specific error".to_string()),
            1 => errors.into_iter().next().unwrap(),
            _ => ParseError::Multiple(errors),
        }
    }
    
    /// Check if this is a syntax error
    pub const fn is_syntax_error(&self) -> bool {
        matches!(self, ParseError::SyntaxError { .. })
    }
    
    /// Check if this is a semantic error
    pub const fn is_semantic_error(&self) -> bool {
        matches!(self, ParseError::SemanticError { .. })
    }
    
    /// Check if this error indicates a timeout
    pub const fn is_timeout(&self) -> bool {
        matches!(self, ParseError::Timeout { .. })
    }
    
    /// Get the primary error message
    pub fn primary_message(&self) -> String {
        match self {
            ParseError::SyntaxError { message, .. } => message.clone(),
            ParseError::SemanticError { message, .. } => message.clone(),
            _ => self.to_string(),
        }
    }
    
    /// Get error location if available
    pub fn location(&self) -> Option<(usize, usize)> {
        match self {
            ParseError::SyntaxError { line, column, .. } => Some((*line, *column)),
            ParseError::SemanticError { span: Some(span), .. } => {
                Some((span.start.line, span.start.column))
            }
            _ => None,
        }
    }
    
    /// Add context to this error
    pub fn with_context(self, context: ErrorContext) -> Self {
        match self {
            ParseError::SyntaxError { line, column, message, .. } => {
                ParseError::SyntaxError {
                    line,
                    column,
                    message,
                    context: Some(context),
                }
            }
            ParseError::SemanticError { message, span, .. } => {
                ParseError::SemanticError {
                    message,
                    span,
                    context: Some(context),
                }
            }
            other => other,
        }
    }
    
    /// Convert to a diagnostic
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            ParseError::SyntaxError { line, column, message, context } => {
                let mut diag = Diagnostic::error(*line, *column, message.clone());
                if let Some(ctx) = context {
                    diag = diag.with_context(ctx.clone());
                }
                diag
            }
            ParseError::SemanticError { message, span, context } => {
                let mut diag = if let Some(s) = span {
                    Diagnostic::error(s.start.line, s.start.column, message.clone())
                } else {
                    Diagnostic::error(0, 0, message.clone())
                };
                if let Some(ctx) = context {
                    diag = diag.with_context(ctx.clone());
                }
                diag
            }
            _ => Diagnostic::error(0, 0, self.to_string()),
        }
    }
}

/// Specialized error types for different parsing phases

/// Lexical analysis errors
#[derive(Error, Debug)]
pub enum LexError {
    #[error("Invalid character: {0}")]
    InvalidCharacter(char),
    
    #[error("Unterminated string")]
    UnterminatedString,
    
    #[error("Invalid escape sequence: {0}")]
    InvalidEscape(String),
    
    #[error("Invalid number format: {0}")]
    InvalidNumber(String),
}

/// Syntax analysis errors
#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unexpected token: {found}, expected: {expected}")]
    UnexpectedToken {
        found: String,
        expected: String,
    },
    
    #[error("Missing closing delimiter: {delimiter}")]
    MissingClosingDelimiter {
        delimiter: String,
    },
    
    #[error("Invalid expression")]
    InvalidExpression,
    
    #[error("Maximum nesting depth exceeded")]
    MaxNestingDepthExceeded,
}

/// Semantic analysis errors
#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Undefined variable: {name}")]
    UndefinedVariable {
        name: String,
    },
    
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: String,
        found: String,
    },
    
    #[error("Duplicate attribute: {name}")]
    DuplicateAttribute {
        name: String,
    },
    
    #[error("Invalid function application")]
    InvalidFunctionApplication,
}

// Implement conversions from specialized errors to ParseError

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        ParseError::ParseFailed(format!("Lexical error: {}", err))
    }
}

impl From<SyntaxError> for ParseError {
    fn from(err: SyntaxError) -> Self {
        ParseError::ParseFailed(format!("Syntax error: {}", err))
    }
}

impl From<SemanticError> for ParseError {
    fn from(err: SemanticError) -> Self {
        ParseError::SemanticError {
            message: err.to_string(),
            span: None,
            context: None,
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ParseError::syntax_error(10, 5, "Unexpected token");
        
        assert!(err.is_syntax_error());
        assert_eq!(err.location(), Some((10, 5)));
        assert_eq!(err.primary_message(), "Unexpected token");
    }

    #[test]
    fn test_error_combination() {
        let errors = vec![
            ParseError::ParseFailed("Error 1".to_string()),
            ParseError::ParseFailed("Error 2".to_string()),
        ];
        
        let combined = ParseError::combine(errors);
        match combined {
            ParseError::Multiple(errs) => assert_eq!(errs.len(), 2),
            _ => panic!("Expected Multiple error"),
        }
    }

    #[test]
    fn test_timeout_error() {
        let err = ParseError::timeout(5000);
        assert!(err.is_timeout());
        
        match err {
            ParseError::Timeout { timeout_ms } => assert_eq!(timeout_ms, 5000),
            _ => panic!("Expected Timeout error"),
        }
    }

    #[test]
    fn test_feature_not_supported() {
        let err = ParseError::feature_not_supported_with_suggestion(
            "experimental_syntax",
            "Enable with --experimental-features"
        );
        
        match err {
            ParseError::FeatureNotSupported { feature, suggestion } => {
                assert_eq!(feature, "experimental_syntax");
                assert!(suggestion.is_some());
            }
            _ => panic!("Expected FeatureNotSupported error"),
        }
    }

    #[test]
    fn test_specialized_errors() {
        let lex_err = LexError::InvalidCharacter('€');
        let parse_err: ParseError = lex_err.into();
        assert!(!parse_err.is_syntax_error());
        
        let syntax_err = SyntaxError::UnexpectedToken {
            found: "if".to_string(),
            expected: "identifier".to_string(),
        };
        let parse_err: ParseError = syntax_err.into();
        assert!(!parse_err.is_syntax_error()); // It's a ParseFailed variant
        
        let semantic_err = SemanticError::UndefinedVariable {
            name: "foo".to_string(),
        };
        let parse_err: ParseError = semantic_err.into();
        assert!(parse_err.is_semantic_error());
    }
}