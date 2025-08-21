//! Conversion utilities for Tree-sitter integration

use tree_sitter::Node;

/// Extension trait for Tree-sitter Tree types
/// 
/// Provides additional functionality for working with Tree-sitter
/// parse trees in the context of Nix parsing.
pub trait TreeSitterExt {
    // Extensions for Tree-sitter types
}

/// Extension trait for Tree-sitter Node types
/// 
/// Provides convenient methods for extracting information
/// from Tree-sitter nodes and converting to AST types.
pub trait NodeExt {
    // Extensions for Node types  
}

impl TreeSitterExt for tree_sitter::Tree {}

impl NodeExt for Node<'_> {}