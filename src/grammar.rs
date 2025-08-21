//! Grammar-related types and utilities for the Nix language
//!
//! This module provides types and functions for working with the Nix grammar,
//! including node type definitions, grammar rules, and Tree-sitter language binding.

use tree_sitter::Language;

extern "C" {
    fn tree_sitter_nix() -> Language;
}

/// Get the Tree-sitter language for Nix
///
/// # Safety
///
/// This function is safe because it calls a well-defined C function
/// that returns a valid Tree-sitter language structure.
pub fn language() -> Language {
    unsafe { tree_sitter_nix() }
}

/// Nix language node types
///
/// These correspond to the node types defined in the Tree-sitter grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    // Root
    SourceFile,
    
    // Literals
    Integer,
    Float,
    String,
    IndentedString,
    Boolean,
    Null,
    Identifier,
    Path,
    Uri,
    
    // Collections
    List,
    Attrset,
    RecAttrset,
    
    // Expressions
    BinaryExpression,
    UnaryExpression,
    Application,
    FunctionExpression,
    LetExpression,
    IfExpression,
    WithExpression,
    AssertExpression,
    ParenthesizedExpression,
    
    // Attribute operations
    Select,
    HasAttr,
    
    // Structural
    Binding,
    Inherit,
    Attrpath,
    Formals,
    Formal,
    
    // String parts
    StringInterpolation,
    
    // Comments and whitespace
    Comment,
    
    // Error nodes
    Error,
    Missing,
}

impl NodeType {
    /// Get all node types
    pub const fn all() -> &'static [NodeType] {
        &[
            NodeType::SourceFile,
            NodeType::Integer,
            NodeType::Float,
            NodeType::String,
            NodeType::IndentedString,
            NodeType::Boolean,
            NodeType::Null,
            NodeType::Identifier,
            NodeType::Path,
            NodeType::Uri,
            NodeType::List,
            NodeType::Attrset,
            NodeType::RecAttrset,
            NodeType::BinaryExpression,
            NodeType::UnaryExpression,
            NodeType::Application,
            NodeType::FunctionExpression,
            NodeType::LetExpression,
            NodeType::IfExpression,
            NodeType::WithExpression,
            NodeType::AssertExpression,
            NodeType::ParenthesizedExpression,
            NodeType::Select,
            NodeType::HasAttr,
            NodeType::Binding,
            NodeType::Inherit,
            NodeType::Attrpath,
            NodeType::Formals,
            NodeType::Formal,
            NodeType::StringInterpolation,
            NodeType::Comment,
            NodeType::Error,
            NodeType::Missing,
        ]
    }
    
    /// Get the string representation of the node type
    pub const fn as_str(self) -> &'static str {
        match self {
            NodeType::SourceFile => "source_file",
            NodeType::Integer => "integer",
            NodeType::Float => "float",
            NodeType::String => "string",
            NodeType::IndentedString => "indented_string",
            NodeType::Boolean => "boolean",
            NodeType::Null => "null",
            NodeType::Identifier => "identifier",
            NodeType::Path => "path",
            NodeType::Uri => "uri",
            NodeType::List => "list",
            NodeType::Attrset => "attrset",
            NodeType::RecAttrset => "rec_attrset",
            NodeType::BinaryExpression => "binary_expression",
            NodeType::UnaryExpression => "unary_expression",
            NodeType::Application => "application",
            NodeType::FunctionExpression => "function_expression",
            NodeType::LetExpression => "let_expression",
            NodeType::IfExpression => "if_expression",
            NodeType::WithExpression => "with_expression",
            NodeType::AssertExpression => "assert_expression",
            NodeType::ParenthesizedExpression => "parenthesized_expression",
            NodeType::Select => "select",
            NodeType::HasAttr => "has_attr",
            NodeType::Binding => "binding",
            NodeType::Inherit => "inherit",
            NodeType::Attrpath => "attrpath",
            NodeType::Formals => "formals",
            NodeType::Formal => "formal",
            NodeType::StringInterpolation => "string_interpolation",
            NodeType::Comment => "comment",
            NodeType::Error => "ERROR",
            NodeType::Missing => "MISSING",
        }
    }
    
    /// Parse a node type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "source_file" => Some(NodeType::SourceFile),
            "integer" => Some(NodeType::Integer),
            "float" => Some(NodeType::Float),
            "string" => Some(NodeType::String),
            "indented_string" => Some(NodeType::IndentedString),
            "boolean" => Some(NodeType::Boolean),
            "null" => Some(NodeType::Null),
            "identifier" => Some(NodeType::Identifier),
            "path" => Some(NodeType::Path),
            "uri" => Some(NodeType::Uri),
            "list" => Some(NodeType::List),
            "attrset" => Some(NodeType::Attrset),
            "rec_attrset" => Some(NodeType::RecAttrset),
            "binary_expression" => Some(NodeType::BinaryExpression),
            "unary_expression" => Some(NodeType::UnaryExpression),
            "application" => Some(NodeType::Application),
            "function_expression" => Some(NodeType::FunctionExpression),
            "let_expression" => Some(NodeType::LetExpression),
            "if_expression" => Some(NodeType::IfExpression),
            "with_expression" => Some(NodeType::WithExpression),
            "assert_expression" => Some(NodeType::AssertExpression),
            "parenthesized_expression" => Some(NodeType::ParenthesizedExpression),
            "select" => Some(NodeType::Select),
            "has_attr" => Some(NodeType::HasAttr),
            "binding" => Some(NodeType::Binding),
            "inherit" => Some(NodeType::Inherit),
            "attrpath" => Some(NodeType::Attrpath),
            "formals" => Some(NodeType::Formals),
            "formal" => Some(NodeType::Formal),
            "string_interpolation" => Some(NodeType::StringInterpolation),
            "comment" => Some(NodeType::Comment),
            "ERROR" => Some(NodeType::Error),
            "MISSING" => Some(NodeType::Missing),
            _ => None,
        }
    }
    
    /// Check if this node type represents a literal value
    pub const fn is_literal(self) -> bool {
        matches!(self, 
                 NodeType::Integer | 
                 NodeType::Float | 
                 NodeType::String | 
                 NodeType::IndentedString | 
                 NodeType::Boolean | 
                 NodeType::Null |
                 NodeType::Path |
                 NodeType::Uri)
    }
    
    /// Check if this node type represents an expression
    pub const fn is_expression(self) -> bool {
        matches!(self,
                 NodeType::BinaryExpression |
                 NodeType::UnaryExpression |
                 NodeType::Application |
                 NodeType::FunctionExpression |
                 NodeType::LetExpression |
                 NodeType::IfExpression |
                 NodeType::WithExpression |
                 NodeType::AssertExpression |
                 NodeType::ParenthesizedExpression |
                 NodeType::Select |
                 NodeType::HasAttr |
                 NodeType::List |
                 NodeType::Attrset |
                 NodeType::RecAttrset) || self.is_literal()
    }
    
    /// Check if this node type represents an error condition
    pub const fn is_error(self) -> bool {
        matches!(self, NodeType::Error | NodeType::Missing)
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for NodeType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NodeType::from_str(s).ok_or_else(|| format!("Unknown node type: {}", s))
    }
}

/// Grammar field names used in the Nix language
///
/// These correspond to the named fields defined in the grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldName {
    // General
    Expression,
    Body,
    
    // Binary expressions
    Left,
    Right,
    Operator,
    
    // Unary expressions  
    Argument,
    
    // Functions
    Function,
    Parameter,
    
    // Control flow
    Condition,
    Consequence,
    Alternative,
    
    // Let expressions
    Bindings,
    
    // Attribute sets
    Attrpath,
    
    // Lists
    Elements,
    
    // Attributes
    Name,
    Default,
    
    // Inheritance
    From,
    Attributes,
}

impl FieldName {
    /// Get the string representation of the field name
    pub const fn as_str(self) -> &'static str {
        match self {
            FieldName::Expression => "expression",
            FieldName::Body => "body",
            FieldName::Left => "left",
            FieldName::Right => "right",
            FieldName::Operator => "operator",
            FieldName::Argument => "argument",
            FieldName::Function => "function",
            FieldName::Parameter => "parameter",
            FieldName::Condition => "condition",
            FieldName::Consequence => "consequence",
            FieldName::Alternative => "alternative",
            FieldName::Bindings => "bindings",
            FieldName::Attrpath => "attrpath",
            FieldName::Elements => "elements",
            FieldName::Name => "name",
            FieldName::Default => "default",
            FieldName::From => "from",
            FieldName::Attributes => "attributes",
        }
    }
}

impl std::fmt::Display for FieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Grammar validation utilities
pub mod validation {
    use super::*;
    use tree_sitter::Node;
    
    /// Check if a node has the expected type
    pub fn validate_node_type(node: Node, expected: NodeType) -> bool {
        node.kind() == expected.as_str()
    }
    
    /// Check if a node has a required field
    pub fn has_required_field(node: Node, field: FieldName) -> bool {
        node.child_by_field_name(field.as_str()).is_some()
    }
    
    /// Validate that a node structure matches expectations
    pub fn validate_node_structure(node: Node, node_type: NodeType, required_fields: &[FieldName]) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Check node type
        if !validate_node_type(node, node_type) {
            errors.push(format!("Expected {} but got {}", node_type, node.kind()));
        }
        
        // Check required fields
        for field in required_fields {
            if !has_required_field(node, *field) {
                errors.push(format!("Missing required field: {}", field));
            }
        }
        
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_creation() {
        let lang = language();
        assert!(lang.abi_version() >= crate::MIN_TREE_SITTER_ABI as usize);
    }

    #[test]
    fn test_node_type_conversion() {
        assert_eq!(NodeType::Integer.as_str(), "integer");
        assert_eq!(NodeType::from_str("integer"), Some(NodeType::Integer));
        assert_eq!(NodeType::from_str("invalid"), None);
        
        let parsed: Result<NodeType, _> = "float".parse();
        assert_eq!(parsed.unwrap(), NodeType::Float);
    }

    #[test]
    fn test_node_type_categories() {
        assert!(NodeType::Integer.is_literal());
        assert!(NodeType::Integer.is_expression());
        assert!(!NodeType::Integer.is_error());
        
        assert!(!NodeType::BinaryExpression.is_literal());
        assert!(NodeType::BinaryExpression.is_expression());
        assert!(!NodeType::BinaryExpression.is_error());
        
        assert!(!NodeType::Error.is_literal());
        assert!(!NodeType::Error.is_expression());
        assert!(NodeType::Error.is_error());
    }

    #[test]
    fn test_field_names() {
        assert_eq!(FieldName::Expression.as_str(), "expression");
        assert_eq!(FieldName::Left.as_str(), "left");
        assert_eq!(format!("{}", FieldName::Right), "right");
    }

    #[test]
    fn test_node_types_coverage() {
        // Ensure all node types have string representations
        for node_type in NodeType::all() {
            let s = node_type.as_str();
            assert!(!s.is_empty());
            
            // Test round-trip conversion (except for error types which have special strings)
            if !node_type.is_error() {
                assert_eq!(NodeType::from_str(s), Some(*node_type));
            }
        }
    }
}