//! Core parsing functionality for the Nix language
//!
//! This module provides the main parser interface and related types.

mod config;
mod result;
mod incremental;

pub use self::config::{ParserConfig, LanguageVersion};
pub use self::result::{ParseResult, ParseDiagnostic};
pub use self::incremental::IncrementalParser;

use std::sync::Arc;
use tree_sitter::{Parser, Tree, Language};

// use crate::ast::Expression; // Not needed for this module interface
use crate::error::{ParseError, Result};

#[cfg(feature = "cache")]
use crate::cache::ParseCache;

#[cfg(feature = "plugins")]
use crate::plugins::Plugin;

extern "C" {
    fn tree_sitter_nix() -> Language;
}

/// Main parser for Nix source code
///
/// The `NixParser` provides a high-level interface for parsing Nix expressions
/// and files. It supports incremental parsing, caching, and plugin integration.
///
/// # Examples
///
/// ```rust
/// use nix_parser::prelude::*;
///
/// let mut parser = NixParser::new()?;
/// let result = parser.parse("{ x = 1; y = 2; }")?;
/// 
/// match result.expression() {
///     Some(Expression::AttrSet(_)) => println!("Parsed attribute set"),
///     _ => println!("Unexpected expression type"),
/// }
/// # Ok::<(), ParseError>(())
/// ```
pub struct NixParser {
    inner: Parser,
    language: Language,
    config: ParserConfig,
    
    #[cfg(feature = "cache")]
    cache: Option<Arc<ParseCache>>,
    
    #[cfg(feature = "plugins")]
    plugins: Vec<Box<dyn Plugin>>,
}

impl NixParser {
    /// Create a new parser with default configuration
    ///
    /// # Errors
    ///
    /// Returns `ParseError::LanguageError` if the Tree-sitter language
    /// cannot be loaded or is incompatible.
    pub fn new() -> Result<Self> {
        Self::with_config(ParserConfig::default())
    }

    /// Create a new parser with custom configuration
    ///
    /// # Errors
    ///
    /// Returns `ParseError::LanguageError` if the Tree-sitter language
    /// cannot be loaded or is incompatible.
    pub fn with_config(config: ParserConfig) -> Result<Self> {
        let language = unsafe { tree_sitter_nix() };
        let mut inner = Parser::new();
        
        inner.set_language(&language)
            .map_err(|e| ParseError::LanguageError(format!("Failed to set language: {}", e)))?;

        // Validate ABI compatibility
        if language.abi_version() < crate::MIN_TREE_SITTER_ABI as usize {
            return Err(ParseError::LanguageError(
                format!("Incompatible Tree-sitter ABI version: {} < {}", 
                       language.abi_version(), crate::MIN_TREE_SITTER_ABI)
            ));
        }

        Ok(Self {
            inner,
            language,
            config,
            
            #[cfg(feature = "cache")]
            cache: None,
            
            #[cfg(feature = "plugins")]
            plugins: Vec::new(),
        })
    }

    /// Parse Nix source code
    ///
    /// # Arguments
    ///
    /// * `source` - The Nix source code to parse
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing the parsed tree and any diagnostics.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if parsing fails due to syntax errors or
    /// internal parser issues.
    pub fn parse(&mut self, source: &str) -> Result<ParseResult> {
        self.parse_with_context(source, None)
    }

    /// Parse Nix source code with an existing tree for incremental parsing
    ///
    /// # Arguments
    ///
    /// * `source` - The Nix source code to parse
    /// * `old_tree` - Optional previous parse tree for incremental parsing
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing the parsed tree and any diagnostics.
    pub fn parse_with_context(&mut self, source: &str, old_tree: Option<&Tree>) -> Result<ParseResult> {
        // Check cache first
        #[cfg(feature = "cache")]
        if let Some(ref cache) = self.cache {
            if let Some(cached_result) = cache.get(source) {
                return Ok(cached_result.clone());
            }
        }

        // Apply plugins before parsing
        #[cfg(feature = "plugins")]
        let processed_source = self.apply_preprocessing_plugins(source)?;
        #[cfg(not(feature = "plugins"))]
        let processed_source = source;

        // Parse the source
        let tree = self.inner.parse(processed_source, old_tree)
            .ok_or_else(|| ParseError::ParseFailed("Tree-sitter parse returned None".to_string()))?;

        let mut result = ParseResult::from_tree(tree, processed_source.to_string())?;

        // Apply plugins after parsing
        #[cfg(feature = "plugins")]
        self.apply_postprocessing_plugins(&mut result)?;

        // Add parsing statistics if enabled
        if self.config.collect_statistics {
            self.add_parse_statistics(&mut result, processed_source);
        }

        // Validate result if enabled
        if self.config.validate_output {
            self.validate_result(&result)?;
        }

        // Cache the result
        #[cfg(feature = "cache")]
        if let Some(ref cache) = self.cache {
            cache.insert(source.to_string(), result.clone());
        }

        Ok(result)
    }

    /// Get the parser configuration
    pub const fn config(&self) -> &ParserConfig {
        &self.config
    }

    /// Update parser configuration
    pub fn set_config(&mut self, config: ParserConfig) {
        self.config = config;
    }

    /// Get the underlying Tree-sitter language
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Enable caching with the specified cache implementation
    #[cfg(feature = "cache")]
    pub fn enable_cache(&mut self, cache: Arc<ParseCache>) {
        self.cache = Some(cache);
    }

    /// Disable caching
    #[cfg(feature = "cache")]
    pub fn disable_cache(&mut self) {
        self.cache = None;
    }

    /// Add a plugin to the parser
    #[cfg(feature = "plugins")]
    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Remove all plugins
    #[cfg(feature = "plugins")]
    pub fn clear_plugins(&mut self) {
        self.plugins.clear();
    }

    // Private helper methods
    
    #[cfg(feature = "plugins")]
    fn apply_preprocessing_plugins(&self, source: &str) -> Result<String> {
        let mut processed = source.to_string();
        
        for plugin in &self.plugins {
            processed = plugin.preprocess(&processed)
                .map_err(|e| ParseError::PluginError(format!("Preprocessing failed: {}", e)))?;
        }
        
        Ok(processed)
    }

    #[cfg(feature = "plugins")]
    fn apply_postprocessing_plugins(&self, result: &mut ParseResult) -> Result<()> {
        for plugin in &self.plugins {
            plugin.postprocess(result)
                .map_err(|e| ParseError::PluginError(format!("Postprocessing failed: {}", e)))?;
        }
        
        Ok(())
    }

    /// Add parsing statistics to the parse result
    fn add_parse_statistics(&self, result: &mut ParseResult, _source: &str) {
        use crate::parser::result::ParseStats;
        use crate::utils::Timer;
        
        // Start timing the statistics calculation
        let timer = Timer::start("parse_statistics");
        
        // Create parse time statistics (using a simple metric for now)
        let parse_time_ms = 1; // Placeholder - in real usage this would be actual parse time
        
        let stats = ParseStats::from_result(result, parse_time_ms, false);
        result.set_statistics(Some(stats));
        
        // Complete timing measurement
        let timing_result = timer.stop();
        
        // In a real implementation, this timing info could be logged or stored
        let _ = timing_result.format(); // Use the label functionality
    }

    fn validate_result(&self, result: &ParseResult) -> Result<()> {
        if result.has_errors() && !self.config.allow_errors {
            return Err(ParseError::ValidationError(
                "Parse result contains errors but allow_errors is false".to_string()
            ));
        }

        // Additional validation rules can be added here
        
        Ok(())
    }
}

impl Default for NixParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default NixParser")
    }
}

// Ensure NixParser is Send and Sync when appropriate
unsafe impl Send for NixParser {}
unsafe impl Sync for NixParser {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = NixParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_simple_parse() {
        let mut parser = NixParser::new().unwrap();
        let result = parser.parse("42");
        assert!(result.is_ok());
        
        let parse_result = result.unwrap();
        assert!(!parse_result.has_errors());
    }

    #[test]
    fn test_error_handling() {
        let mut parser = NixParser::new().unwrap();
        let result = parser.parse("if true then");
        
        // Should parse but with errors
        assert!(result.is_ok());
        let parse_result = result.unwrap();
        assert!(parse_result.has_errors());
    }

    #[test]
    fn test_config_update() {
        let mut parser = NixParser::new().unwrap();
        let mut config = ParserConfig::default();
        config.allow_errors = false;
        
        parser.set_config(config);
        assert!(!parser.config().allow_errors);
    }
}