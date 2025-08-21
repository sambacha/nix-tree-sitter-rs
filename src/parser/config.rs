//! Parser configuration and language version management

use std::collections::HashMap;

/// Configuration for the Nix parser
///
/// This struct controls various aspects of parser behavior,
/// including error handling, optimization settings, and language features.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserConfig {
    /// Whether to allow parsing with syntax errors
    pub allow_errors: bool,
    
    /// Whether to validate output after parsing
    pub validate_output: bool,
    
    /// Whether to enable incremental parsing
    pub incremental_parsing: bool,
    
    /// Language version to target
    pub language_version: LanguageVersion,
    
    /// Whether to include location information in AST nodes
    pub include_locations: bool,
    
    /// Whether to include comments in the AST
    pub include_comments: bool,
    
    /// Whether to preserve whitespace information
    pub preserve_whitespace: bool,
    
    /// Maximum nesting depth to prevent stack overflow
    pub max_nesting_depth: Option<usize>,
    
    /// Custom feature flags
    pub feature_flags: HashMap<String, bool>,
    
    /// Timeout for parsing operations in milliseconds
    pub timeout_ms: Option<u64>,
    
    /// Whether to collect parsing statistics
    pub collect_statistics: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            allow_errors: true,
            validate_output: false,
            incremental_parsing: true,
            language_version: LanguageVersion::Latest,
            include_locations: true,
            include_comments: false,
            preserve_whitespace: false,
            max_nesting_depth: Some(1000),
            feature_flags: HashMap::new(),
            timeout_ms: None,
            collect_statistics: false,
        }
    }
}

impl ParserConfig {
    /// Create a new configuration builder
    pub fn builder() -> ParserConfigBuilder {
        ParserConfigBuilder::new()
    }
    
    /// Create a strict configuration (no errors allowed)
    pub fn strict() -> Self {
        Self {
            allow_errors: false,
            validate_output: true,
            ..Default::default()
        }
    }
    
    /// Create a lenient configuration (optimized for IDEs)
    pub fn lenient() -> Self {
        Self {
            allow_errors: true,
            validate_output: false,
            include_comments: true,
            preserve_whitespace: true,
            ..Default::default()
        }
    }
    
    /// Create a performance-optimized configuration
    pub fn performance() -> Self {
        Self {
            allow_errors: true,
            validate_output: false,
            include_locations: false,
            include_comments: false,
            preserve_whitespace: false,
            incremental_parsing: true,
            max_nesting_depth: Some(500),
            collect_statistics: true, // Useful for performance monitoring
            ..Default::default()
        }
    }
    
    /// Enable a feature flag
    pub fn enable_feature(&mut self, name: impl Into<String>) {
        self.feature_flags.insert(name.into(), true);
    }
    
    /// Disable a feature flag
    pub fn disable_feature(&mut self, name: impl Into<String>) {
        self.feature_flags.insert(name.into(), false);
    }
    
    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, name: &str) -> bool {
        self.feature_flags.get(name).copied().unwrap_or(false)
    }
}

/// Nix language version targeting
///
/// Different versions of Nix have slightly different language features.
/// This enum allows targeting specific versions for compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageVersion {
    /// Nix 2.3 LTS
    Nix23,
    /// Nix 2.4
    Nix24,
    /// Nix 2.8
    Nix28,
    /// Nix 2.18 (current stable)
    Nix218,
    /// Latest version (may include experimental features)
    Latest,
    /// Experimental features enabled
    Experimental,
}

impl Default for LanguageVersion {
    fn default() -> Self {
        Self::Latest
    }
}

impl LanguageVersion {
    /// Get all supported language versions
    pub const fn all() -> &'static [LanguageVersion] {
        &[
            LanguageVersion::Nix23,
            LanguageVersion::Nix24,
            LanguageVersion::Nix28,
            LanguageVersion::Nix218,
            LanguageVersion::Latest,
            LanguageVersion::Experimental,
        ]
    }
    
    /// Get the version string
    pub const fn as_str(self) -> &'static str {
        match self {
            LanguageVersion::Nix23 => "2.3",
            LanguageVersion::Nix24 => "2.4",
            LanguageVersion::Nix28 => "2.8",
            LanguageVersion::Nix218 => "2.18",
            LanguageVersion::Latest => "latest",
            LanguageVersion::Experimental => "experimental",
        }
    }
    
    /// Check if flakes are supported in this version
    pub const fn supports_flakes(self) -> bool {
        matches!(self, 
                 LanguageVersion::Nix24 | 
                 LanguageVersion::Nix28 | 
                 LanguageVersion::Nix218 | 
                 LanguageVersion::Latest | 
                 LanguageVersion::Experimental)
    }
    
    /// Check if the `or` keyword is supported
    pub const fn supports_or_keyword(self) -> bool {
        // All versions support 'or' keyword for backward compatibility
        true
    }
    
    /// Check if scientific notation is supported for floats
    pub const fn supports_scientific_notation(self) -> bool {
        matches!(self,
                 LanguageVersion::Nix28 |
                 LanguageVersion::Nix218 |
                 LanguageVersion::Latest |
                 LanguageVersion::Experimental)
    }
}

/// Builder for `ParserConfig`
///
/// Provides a fluent interface for constructing parser configurations.
#[derive(Debug)]
pub struct ParserConfigBuilder {
    config: ParserConfig,
}

impl ParserConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }
    
    /// Set whether to allow errors
    pub fn allow_errors(mut self, allow: bool) -> Self {
        self.config.allow_errors = allow;
        self
    }
    
    /// Set whether to validate output
    pub fn validate_output(mut self, validate: bool) -> Self {
        self.config.validate_output = validate;
        self
    }
    
    /// Set language version
    pub fn language_version(mut self, version: LanguageVersion) -> Self {
        self.config.language_version = version;
        self
    }
    
    /// Set whether to include locations
    pub fn include_locations(mut self, include: bool) -> Self {
        self.config.include_locations = include;
        self
    }
    
    /// Set whether to include comments
    pub fn include_comments(mut self, include: bool) -> Self {
        self.config.include_comments = include;
        self
    }
    
    /// Set whether to preserve whitespace
    pub fn preserve_whitespace(mut self, preserve: bool) -> Self {
        self.config.preserve_whitespace = preserve;
        self
    }
    
    /// Set maximum nesting depth
    pub fn max_nesting_depth(mut self, depth: Option<usize>) -> Self {
        self.config.max_nesting_depth = depth;
        self
    }
    
    /// Set timeout
    pub fn timeout_ms(mut self, timeout: Option<u64>) -> Self {
        self.config.timeout_ms = timeout;
        self
    }
    
    /// Set whether to collect statistics
    pub fn collect_statistics(mut self, collect: bool) -> Self {
        self.config.collect_statistics = collect;
        self
    }
    
    /// Enable a feature flag
    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.config.enable_feature(name);
        self
    }
    
    /// Build the final configuration
    pub fn build(self) -> ParserConfig {
        self.config
    }
}

impl Default for ParserConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ParserConfig::default();
        assert!(config.allow_errors);
        assert!(config.include_locations);
        assert!(config.incremental_parsing);
    }

    #[test]
    fn test_strict_config() {
        let config = ParserConfig::strict();
        assert!(!config.allow_errors);
        assert!(config.validate_output);
    }

    #[test]
    fn test_config_builder() {
        let config = ParserConfig::builder()
            .allow_errors(false)
            .language_version(LanguageVersion::Nix218)
            .include_comments(true)
            .enable_feature("experimental_syntax")
            .build();
            
        assert!(!config.allow_errors);
        assert_eq!(config.language_version, LanguageVersion::Nix218);
        assert!(config.include_comments);
        assert!(config.is_feature_enabled("experimental_syntax"));
    }

    #[test]
    fn test_language_versions() {
        assert!(LanguageVersion::Latest.supports_flakes());
        assert!(!LanguageVersion::Nix23.supports_flakes());
        assert!(LanguageVersion::Nix218.supports_scientific_notation());
        assert!(!LanguageVersion::Nix24.supports_scientific_notation());
    }

    #[test]
    fn test_feature_flags() {
        let mut config = ParserConfig::default();
        assert!(!config.is_feature_enabled("test_feature"));
        
        config.enable_feature("test_feature");
        assert!(config.is_feature_enabled("test_feature"));
        
        config.disable_feature("test_feature");
        assert!(!config.is_feature_enabled("test_feature"));
    }

    #[test]
    fn test_statistics_collection() {
        let config = ParserConfig::performance();
        assert!(config.collect_statistics);
        
        let config = ParserConfig::builder()
            .collect_statistics(true)
            .build();
        assert!(config.collect_statistics);
    }
}