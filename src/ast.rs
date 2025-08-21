use serde::{Deserialize, Serialize};
use std::fmt;

/// Base trait for all AST nodes
pub trait Node: fmt::Debug {
    fn location(&self) -> Option<SourceLocation>;
    fn children(&self) -> Vec<&dyn Node>;
    fn accept(&self, visitor: &mut dyn crate::visitor::Visitor);
}

/// Source location information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_position: (usize, usize),
    pub end_position: (usize, usize),
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize, start_byte: usize, end_byte: usize) -> Self {
        Self {
            start_byte,
            end_byte,
            start_position: (line, column),
            end_position: (line, column),
            line,
            column,
        }
    }
    
    /// Create from a Tree-sitter node
    pub fn from_tree_sitter_node(node: &tree_sitter::Node) -> Self {
        Self {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_position: (node.start_position().row, node.start_position().column),
            end_position: (node.end_position().row, node.end_position().column),
            line: node.start_position().row + 1, // Convert to 1-based
            column: node.start_position().column + 1, // Convert to 1-based
        }
    }
}

/// Main expression types in Nix
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    StringInterpolation {
        parts: Vec<StringPart>,
    },
    Path(PathType),
    Boolean(bool),
    Null,
    
    // Identifiers and references
    Identifier(String),
    
    // Collections
    List(Vec<Expression>),
    AttributeSet {
        recursive: bool,
        attributes: Vec<Attribute>,
    },
    
    // Functions
    Function {
        parameter: Parameter,
        body: Box<Expression>,
    },
    Application {
        function: Box<Expression>,
        argument: Box<Expression>,
    },
    
    // Control flow
    LetIn {
        bindings: Vec<Binding>,
        body: Box<Expression>,
    },
    With {
        scope: Box<Expression>,
        body: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
    },
    Assert {
        condition: Box<Expression>,
        body: Box<Expression>,
    },
    
    // Operators
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    
    // Selection and interpolation
    Select {
        expr: Box<Expression>,
        path: Vec<String>,
        default: Option<Box<Expression>>,
    },
    HasAttr {
        expr: Box<Expression>,
        path: Vec<String>,
    },
    
    // Import expression
    Import {
        path: Box<Expression>,
    },
    
    // Inherit statement
    Inherit {
        source: Option<Box<Expression>>,
        attributes: Vec<String>,
    },
}

/// String parts for interpolation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringPart {
    Literal(String),
    Interpolation(Box<Expression>),
}

/// Path types in Nix
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathType {
    Absolute(String),
    Relative(String),
    Home(String),
    Search(String),
}

/// Function parameter patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Parameter {
    Identifier(String),
    Pattern {
        fields: Vec<PatternField>,
        ellipsis: bool,
        bind: Option<String>,
    },
}

/// Pattern field in function parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternField {
    pub name: String,
    pub default: Option<Expression>,
}

/// Attribute in an attribute set
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub path: Vec<String>,
    pub value: Expression,
}

/// Binding in let expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub name: String,
    pub value: Expression,
    pub inherit: bool,
    pub from: Option<Expression>,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    
    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    
    // Logical
    And,
    Or,
    Implies,
    
    // Other
    Update,
    Concat,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Negate,
}

/// Parts of string interpolation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InterpolationPart {
    String(String),
    Expression(Box<Expression>),
}

impl Expression {
    /// Create an Expression from a Tree-sitter node
    pub fn from_tree_sitter_node(_node: tree_sitter::Node, _source: &str) -> crate::error::Result<Expression> {
        // Placeholder implementation - would parse the actual node
        Ok(Expression::Integer(0))
    }
}

impl Node for Expression {
    fn location(&self) -> Option<SourceLocation> {
        // In a full implementation, each variant would track its location
        None
    }
    
    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Expression::Function { body, .. } => vec![body.as_ref()],
            Expression::Application { function, argument } => {
                vec![function.as_ref(), argument.as_ref()]
            }
            Expression::LetIn { body, .. } => vec![body.as_ref()],
            Expression::With { scope, body } => vec![scope.as_ref(), body.as_ref()],
            Expression::If { condition, then_branch, else_branch } => {
                vec![condition.as_ref(), then_branch.as_ref(), else_branch.as_ref()]
            }
            Expression::Assert { condition, body } => {
                vec![condition.as_ref(), body.as_ref()]
            }
            Expression::BinaryOp { left, right, .. } => {
                vec![left.as_ref(), right.as_ref()]
            }
            Expression::UnaryOp { operand, .. } => vec![operand.as_ref()],
            Expression::Select { expr, .. } => vec![expr.as_ref()],
            Expression::HasAttr { expr, .. } => vec![expr.as_ref()],
            _ => vec![],
        }
    }
    
    fn accept(&self, visitor: &mut dyn crate::visitor::Visitor) {
        visitor.visit_expression(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_creation() {
        let expr = Expression::Integer(42);
        assert_eq!(expr, Expression::Integer(42));
        
        let expr = Expression::String("hello".to_string());
        assert_eq!(expr, Expression::String("hello".to_string()));
    }
    
    #[test]
    fn test_binary_op() {
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Integer(1)),
            right: Box::new(Expression::Integer(2)),
        };
        
        let children = expr.children();
        assert_eq!(children.len(), 2);
    }
}