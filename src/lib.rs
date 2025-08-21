//! # Nix Parser
//!
//! A modern, fast, and extensible parser for the Nix language built on Tree-sitter.
//! 
//! This library provides comprehensive parsing capabilities for the Nix language,
//! following Rust best practices for modularity, error handling, and API design.
//!
//! ## Quick Start
//!
//! ```rust
//! use nix_parser::prelude::*;
//!
//! let mut parser = NixParser::new()?;
//! let result = parser.parse("{ x = 1; y = 2; }")?;
//! # Ok::<(), nix_parser::ParseError>(())
//! ```
//!
//! ## Features
//!
//! - **High Performance**: Built on Tree-sitter for fast, incremental parsing
//! - **Error Recovery**: Comprehensive error handling with recovery strategies
//! - **Extensible**: Plugin system for custom analysis and transformations
//! - **Specification Compliant**: Follows official Nix language grammar
//! - **Memory Safe**: Written in Rust with zero-cost abstractions

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// Core parsing functionality
pub mod parser;
pub mod grammar;
pub mod scanner;

// AST and node types
pub mod ast;
pub mod visitor;

// Error handling
pub mod error;

// Analysis and transformation
pub mod analysis;
pub mod transform;

// Utilities
pub mod utils;

// Feature-gated modules
#[cfg(feature = "plugins")]
#[cfg_attr(docsrs, doc(cfg(feature = "plugins")))]
pub mod plugins;

#[cfg(feature = "cache")]
#[cfg_attr(docsrs, doc(cfg(feature = "cache")))]
pub mod cache;

#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
pub mod wasm;

#[cfg(feature = "python")]
#[cfg_attr(docsrs, doc(cfg(feature = "python")))]
pub mod python;

#[cfg(feature = "cli")]
#[cfg_attr(docsrs, doc(cfg(feature = "cli")))]
pub mod cli;

// Re-exports for convenience
pub use crate::parser::NixParser;
pub use crate::ast::{Expression, Node, SourceLocation};
pub use crate::error::{ParseError, Result};

/// Prelude module for common imports
///
/// This module re-exports the most commonly used types and traits
/// for convenient importing.
pub mod prelude {
    //! Common imports for working with the Nix parser
    
    pub use crate::parser::NixParser;
    pub use crate::ast::{Expression, Node, SourceLocation};
    pub use crate::error::{ParseError, Result};
    pub use crate::visitor::Visitor;
    
    #[cfg(feature = "plugins")]
    pub use crate::plugins::Plugin;
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Language version this parser supports
pub const SUPPORTED_NIX_VERSION: &str = "2.18";

/// Minimum Tree-sitter ABI version required
pub const MIN_TREE_SITTER_ABI: u32 = 14;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!SUPPORTED_NIX_VERSION.is_empty());
        assert!(MIN_TREE_SITTER_ABI >= 13);
    }
}