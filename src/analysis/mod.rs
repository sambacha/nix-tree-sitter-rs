//! Static analysis and validation for Nix ASTs
//!
//! This module provides tools for analyzing parsed Nix code,
//! including semantic validation, dependency analysis, and code quality checks.

pub mod semantic;
pub mod dependency;
pub mod lint;
pub mod scope;

pub use self::semantic::{SemanticAnalyzer, SemanticError};
pub use self::dependency::{DependencyAnalyzer, Dependency, DependencyGraph};
pub use self::lint::{Linter, LintRule, LintResult};
pub use self::scope::{ScopeAnalyzer, Scope, ScopeType};

use crate::ast::Expression;
use crate::error::{ParseError, Result};

/// Main interface for static analysis
///
/// Provides a unified interface for running various analysis passes
/// on parsed Nix code.
pub struct Analyzer {
    semantic: SemanticAnalyzer,
    dependency: DependencyAnalyzer,
    linter: Linter,
    scope: ScopeAnalyzer,
}

impl Analyzer {
    /// Create a new analyzer with default configuration
    pub fn new() -> Self {
        Self {
            semantic: SemanticAnalyzer::new(),
            dependency: DependencyAnalyzer::new(),
            linter: Linter::new(),
            scope: ScopeAnalyzer::new(),
        }
    }
    
    /// Run all analysis passes on an expression
    pub fn analyze(&mut self, expression: &Expression) -> Result<AnalysisResult> {
        let mut result = AnalysisResult::new();
        
        // Scope analysis (foundation for other analyses)
        let scopes = self.scope.analyze(expression)?;
        result.scopes = scopes;
        
        // Semantic analysis
        match self.semantic.analyze(expression) {
            Ok(semantic_info) => result.semantic = Some(semantic_info),
            Err(error) => result.errors.push(error),
        }
        
        // Dependency analysis
        match self.dependency.analyze(expression) {
            Ok(deps) => result.dependencies = deps,
            Err(e) => result.errors.push(e),
        }
        
        // Linting
        let lint_results = self.linter.lint(expression)?;
        result.lint_results = lint_results;
        
        Ok(result)
    }
    
    /// Configure the analyzer
    pub fn with_config(mut self, config: AnalyzerConfig) -> Self {
        if let Some(semantic_config) = config.semantic {
            self.semantic = self.semantic.with_config(semantic_config);
        }
        if let Some(lint_config) = config.lint {
            self.linter = self.linter.with_config(lint_config);
        }
        self
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the analyzer
#[derive(Debug, Clone, Default)]
pub struct AnalyzerConfig {
    /// Semantic analysis configuration
    pub semantic: Option<semantic::Config>,
    
    /// Dependency analysis configuration  
    pub dependency: Option<dependency::Config>,
    
    /// Linting configuration
    pub lint: Option<lint::Config>,
    
    /// Scope analysis configuration
    pub scope: Option<scope::Config>,
}

/// Result of running analysis on Nix code
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Scope information
    pub scopes: Vec<Scope>,
    
    /// Semantic analysis results
    pub semantic: Option<semantic::SemanticInfo>,
    
    /// Dependency graph
    pub dependencies: DependencyGraph,
    
    /// Lint results
    pub lint_results: Vec<LintResult>,
    
    /// Analysis errors
    pub errors: Vec<ParseError>,
}

impl AnalysisResult {
    /// Create a new empty analysis result
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            semantic: None,
            dependencies: DependencyGraph::new(),
            lint_results: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    /// Check if analysis found any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Get all errors as a combined result
    pub fn into_result(self) -> Result<Self> {
        if self.has_errors() {
            Err(ParseError::combine(self.errors))
        } else {
            Ok(self)
        }
    }
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = Analyzer::new();
        // Basic creation should work
        drop(analyzer);
    }

    #[test]
    fn test_analysis_result() {
        let result = AnalysisResult::new();
        assert!(!result.has_errors());
        assert!(result.scopes.is_empty());
        assert!(result.semantic.is_none());
    }
}