use crate::ast::Expression;

/// Query engine for pattern matching on AST
pub struct QueryEngine {
    patterns: Vec<Pattern>,
}

impl QueryEngine {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }
    
    /// Add a pattern to the query engine
    pub fn add_pattern(&mut self, pattern: Pattern) {
        self.patterns.push(pattern);
    }
    
    /// Execute queries on an expression
    pub fn query(&self, expr: &Expression) -> Vec<Match> {
        let mut matches = Vec::new();
        
        for pattern in &self.patterns {
            if let Some(m) = pattern.match_expression(expr) {
                matches.push(m);
            }
        }
        
        matches
    }
}

/// Pattern for matching AST nodes
#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub matcher: Matcher,
}

impl Pattern {
    pub fn new(name: impl Into<String>, matcher: Matcher) -> Self {
        Self {
            name: name.into(),
            matcher,
        }
    }
    
    fn match_expression(&self, expr: &Expression) -> Option<Match> {
        if self.matcher.matches(expr) {
            Some(Match {
                pattern_name: self.name.clone(),
                matched_expression: expr.clone(),
            })
        } else {
            None
        }
    }
}

/// Matcher for pattern matching
#[derive(Debug, Clone)]
pub enum Matcher {
    /// Match any expression
    Any,
    
    /// Match a specific expression type
    Type(ExpressionType),
    
    /// Match an identifier with a specific name
    Identifier(String),
    
    /// Match a binary operation
    BinaryOp(crate::ast::BinaryOperator),
    
    /// Combine matchers
    And(Box<Matcher>, Box<Matcher>),
    Or(Box<Matcher>, Box<Matcher>),
    Not(Box<Matcher>),
}

impl Matcher {
    pub fn matches(&self, expr: &Expression) -> bool {
        match self {
            Matcher::Any => true,
            Matcher::Type(t) => t.matches(expr),
            Matcher::Identifier(name) => {
                matches!(expr, Expression::Identifier(id) if id == name)
            }
            Matcher::BinaryOp(op) => {
                matches!(expr, Expression::BinaryOp { op: expr_op, .. } if expr_op == op)
            }
            Matcher::And(a, b) => a.matches(expr) && b.matches(expr),
            Matcher::Or(a, b) => a.matches(expr) || b.matches(expr),
            Matcher::Not(m) => !m.matches(expr),
        }
    }
}

/// Expression type for pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionType {
    Integer,
    Float,
    String,
    Boolean,
    Null,
    Identifier,
    List,
    AttributeSet,
    Function,
    Application,
    LetIn,
    With,
    If,
    Assert,
    BinaryOp,
    UnaryOp,
    Select,
    HasAttr,
    Interpolation,
}

impl ExpressionType {
    pub fn matches(&self, expr: &Expression) -> bool {
        match (self, expr) {
            (ExpressionType::Integer, Expression::Integer(_)) => true,
            (ExpressionType::Float, Expression::Float(_)) => true,
            (ExpressionType::String, Expression::String(_)) => true,
            (ExpressionType::Boolean, Expression::Boolean(_)) => true,
            (ExpressionType::Null, Expression::Null) => true,
            (ExpressionType::Identifier, Expression::Identifier(_)) => true,
            (ExpressionType::List, Expression::List(_)) => true,
            (ExpressionType::AttributeSet, Expression::AttributeSet { .. }) => true,
            (ExpressionType::Function, Expression::Function { .. }) => true,
            (ExpressionType::Application, Expression::Application { .. }) => true,
            (ExpressionType::LetIn, Expression::LetIn { .. }) => true,
            (ExpressionType::With, Expression::With { .. }) => true,
            (ExpressionType::If, Expression::If { .. }) => true,
            (ExpressionType::Assert, Expression::Assert { .. }) => true,
            (ExpressionType::BinaryOp, Expression::BinaryOp { .. }) => true,
            (ExpressionType::UnaryOp, Expression::UnaryOp { .. }) => true,
            (ExpressionType::Select, Expression::Select { .. }) => true,
            (ExpressionType::HasAttr, Expression::HasAttr { .. }) => true,
            (ExpressionType::Interpolation, Expression::StringInterpolation { .. }) => true,
            _ => false,
        }
    }
}

/// A match result
#[derive(Debug, Clone)]
pub struct Match {
    pub pattern_name: String,
    pub matched_expression: Expression,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::BinaryOperator;
    
    #[test]
    fn test_pattern_matching() {
        let engine = QueryEngine::new();
        
        let expr = Expression::Integer(42);
        let pattern = Pattern::new("integer", Matcher::Type(ExpressionType::Integer));
        
        assert!(pattern.matcher.matches(&expr));
    }
    
    #[test]
    fn test_binary_op_matcher() {
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Integer(1)),
            right: Box::new(Expression::Integer(2)),
        };
        
        let matcher = Matcher::BinaryOp(BinaryOperator::Add);
        assert!(matcher.matches(&expr));
        
        let matcher = Matcher::BinaryOp(BinaryOperator::Subtract);
        assert!(!matcher.matches(&expr));
    }
    
    #[test]
    fn test_combined_matchers() {
        let expr = Expression::Integer(42);
        
        let matcher = Matcher::And(
            Box::new(Matcher::Type(ExpressionType::Integer)),
            Box::new(Matcher::Not(Box::new(Matcher::Type(ExpressionType::Float)))),
        );
        
        assert!(matcher.matches(&expr));
    }
}