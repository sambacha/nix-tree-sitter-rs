//! Diagnostic formatting and utilities

/// A diagnostic message with severity level and location information
#[derive(Debug, Clone)]
pub struct Diagnostic {}

impl Diagnostic {
    /// Create a new error-level diagnostic
    /// 
    /// # Arguments
    /// 
    /// * `_line` - The line number where the error occurred
    /// * `_column` - The column number where the error occurred  
    /// * `_message` - The diagnostic message
    pub fn error(_line: usize, _column: usize, _message: String) -> Self {
        Self {}
    }
    
    /// Add additional context information to this diagnostic
    /// 
    /// # Arguments
    /// 
    /// * `_context` - Additional error context to attach
    pub fn with_context(self, _context: super::ErrorContext) -> Self {
        self
    }
}

/// Builder for constructing complex diagnostic messages
#[derive(Debug, Clone)]
pub struct DiagnosticBuilder {}

/// Severity levels for diagnostic messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {}