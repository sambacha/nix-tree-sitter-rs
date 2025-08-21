use crate::ast::*;

/// Visitor trait for traversing the AST
pub trait Visitor {
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Integer(n) => self.visit_integer(*n),
            Expression::Float(f) => self.visit_float(*f),
            Expression::String(s) => self.visit_string(s),
            Expression::Path(p) => self.visit_path(p),
            Expression::Boolean(b) => self.visit_boolean(*b),
            Expression::Null => self.visit_null(),
            Expression::Identifier(id) => self.visit_identifier(id),
            Expression::List(items) => self.visit_list(items),
            Expression::AttributeSet { recursive, attributes } => {
                self.visit_attribute_set(*recursive, attributes)
            }
            Expression::Function { parameter, body } => {
                self.visit_function(parameter, body)
            }
            Expression::Application { function, argument } => {
                self.visit_application(function, argument)
            }
            Expression::LetIn { bindings, body } => {
                self.visit_let_in(bindings, body)
            }
            Expression::With { scope, body } => {
                self.visit_with(scope, body)
            }
            Expression::If { condition, then_branch, else_branch } => {
                self.visit_if(condition, then_branch, else_branch)
            }
            Expression::Assert { condition, body } => {
                self.visit_assert(condition, body)
            }
            Expression::BinaryOp { op, left, right } => {
                self.visit_binary_op(*op, left, right)
            }
            Expression::UnaryOp { op, operand } => {
                self.visit_unary_op(*op, operand)
            }
            Expression::Select { expr, path, default } => {
                self.visit_select(expr, path, default.as_deref())
            }
            Expression::HasAttr { expr, path } => {
                self.visit_has_attr(expr, path)
            }
            Expression::StringInterpolation { parts } => {
                self.visit_string_interpolation(parts)
            }
            Expression::Import { path } => {
                self.visit_import(path)
            }
            Expression::Inherit { source, attributes } => {
                self.visit_inherit(source.as_deref(), attributes)
            }
        }
    }
    
    fn visit_integer(&mut self, _n: i64) {}
    fn visit_float(&mut self, _f: f64) {}
    fn visit_string(&mut self, _s: &str) {}
    fn visit_path(&mut self, _p: &PathType) {}
    fn visit_boolean(&mut self, _b: bool) {}
    fn visit_null(&mut self) {}
    fn visit_identifier(&mut self, _id: &str) {}
    
    fn visit_list(&mut self, items: &[Expression]) {
        for item in items {
            self.visit_expression(item);
        }
    }
    
    fn visit_attribute_set(&mut self, _recursive: bool, attributes: &[Attribute]) {
        for attr in attributes {
            self.visit_expression(&attr.value);
        }
    }
    
    fn visit_function(&mut self, _parameter: &Parameter, body: &Expression) {
        self.visit_expression(body);
    }
    
    fn visit_application(&mut self, function: &Expression, argument: &Expression) {
        self.visit_expression(function);
        self.visit_expression(argument);
    }
    
    fn visit_let_in(&mut self, bindings: &[Binding], body: &Expression) {
        for binding in bindings {
            self.visit_expression(&binding.value);
        }
        self.visit_expression(body);
    }
    
    fn visit_with(&mut self, scope: &Expression, body: &Expression) {
        self.visit_expression(scope);
        self.visit_expression(body);
    }
    
    fn visit_if(
        &mut self,
        condition: &Expression,
        then_branch: &Expression,
        else_branch: &Expression,
    ) {
        self.visit_expression(condition);
        self.visit_expression(then_branch);
        self.visit_expression(else_branch);
    }
    
    fn visit_assert(&mut self, condition: &Expression, body: &Expression) {
        self.visit_expression(condition);
        self.visit_expression(body);
    }
    
    fn visit_binary_op(
        &mut self,
        _op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) {
        self.visit_expression(left);
        self.visit_expression(right);
    }
    
    fn visit_unary_op(&mut self, _op: UnaryOperator, operand: &Expression) {
        self.visit_expression(operand);
    }
    
    fn visit_select(
        &mut self,
        expr: &Expression,
        _path: &[String],
        default: Option<&Expression>,
    ) {
        self.visit_expression(expr);
        if let Some(def) = default {
            self.visit_expression(def);
        }
    }
    
    fn visit_has_attr(&mut self, expr: &Expression, _path: &[String]) {
        self.visit_expression(expr);
    }
    
    fn visit_string_interpolation(&mut self, parts: &[StringPart]) {
        for part in parts {
            match part {
                StringPart::Literal(_) => {
                    // Nothing to visit for literals
                }
                StringPart::Interpolation(expr) => {
                    self.visit_expression(expr);
                }
            }
        }
    }
    
    fn visit_import(&mut self, path: &Expression) {
        self.visit_expression(path);
    }
    
    fn visit_inherit(&mut self, source: Option<&Expression>, _attributes: &[String]) {
        if let Some(source_expr) = source {
            self.visit_expression(source_expr);
        }
    }
}

/// Example visitor that collects all identifiers
pub struct IdentifierCollector {
    pub identifiers: Vec<String>,
}

impl IdentifierCollector {
    pub fn new() -> Self {
        Self {
            identifiers: Vec::new(),
        }
    }
}

impl Visitor for IdentifierCollector {
    fn visit_identifier(&mut self, id: &str) {
        self.identifiers.push(id.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identifier_collector() {
        let mut collector = IdentifierCollector::new();
        
        let expr = Expression::Identifier("test".to_string());
        collector.visit_expression(&expr);
        
        assert_eq!(collector.identifiers, vec!["test"]);
    }
    
    #[test]
    fn test_nested_visitor() {
        let mut collector = IdentifierCollector::new();
        
        let expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Identifier("x".to_string())),
            right: Box::new(Expression::Identifier("y".to_string())),
        };
        
        collector.visit_expression(&expr);
        assert_eq!(collector.identifiers, vec!["x", "y"]);
    }
}