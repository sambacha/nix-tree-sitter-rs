use tree_sitter::Language;

/// Get the Tree-sitter language for Nix
pub fn language() -> Language {
    unsafe { tree_sitter_nix() }
}

extern "C" {
    fn tree_sitter_nix() -> Language;
}

/// Grammar layers for progressive complexity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarLayer {
    /// Basic literals and operators
    Basic,
    /// Standard Nix features
    Standard,
    /// Advanced features including imports
    Advanced,
    /// Experimental features
    Experimental,
}

impl GrammarLayer {
    /// Get the allowed node types for this layer
    pub fn allowed_nodes(&self) -> &'static [&'static str] {
        match self {
            GrammarLayer::Basic => &[
                "integer", "float", "string", "boolean", "null",
                "identifier", "binary_expression", "unary_expression",
                "list", "attribute_set", "parenthesized_expression",
            ],
            GrammarLayer::Standard => &[
                // All basic nodes plus:
                "function", "function_application",
                "let_expression", "if_expression",
                "selection", "has_attribute",
            ],
            GrammarLayer::Advanced => &[
                // All standard nodes plus:
                "with_expression", "assert_expression",
                "import", "derivation",
            ],
            GrammarLayer::Experimental => &[
                // All nodes allowed
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grammar_layers() {
        let basic = GrammarLayer::Basic;
        assert!(basic.allowed_nodes().contains(&"integer"));
        assert!(!basic.allowed_nodes().contains(&"import"));
        
        let advanced = GrammarLayer::Advanced;
        assert!(advanced.allowed_nodes().contains(&"import"));
    }
}