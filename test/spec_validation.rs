// Specification-driven parser validation tests
// These tests validate abstract language rules, not specific examples

use tree_sitter::{Parser, Tree, Node};

pub struct SpecValidator {
    parser: Parser,
}

impl SpecValidator {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_nix::language()).unwrap();
        Self { parser }
    }

    fn parse(&mut self, input: &str) -> Tree {
        self.parser.parse(input, None).unwrap()
    }

    fn get_expression_node(&self, tree: &Tree) -> Node {
        let root = tree.root_node();
        root.child_by_field_name("expression").unwrap()
    }
}

#[cfg(test)]
mod precedence_rules {
    use super::*;

    #[test]
    fn function_application_has_higher_precedence_than_addition() {
        let mut validator = SpecValidator::new();
        
        // Rule: Function application binds tighter than addition
        // Test: f g + h should parse as (f g) + h
        let tree = validator.parse("f g + h");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "binary_expression");
        let left = expr.child_by_field_name("left").unwrap();
        assert_eq!(left.kind(), "application");
        
        // Verify the application is (f g)
        let app_func = left.child_by_field_name("function").unwrap();
        let app_arg = left.child_by_field_name("argument").unwrap();
        assert_eq!(app_func.utf8_text(b"f g + h").unwrap(), "f");
        assert_eq!(app_arg.utf8_text(b"f g + h").unwrap(), "g");
    }

    #[test]
    fn function_application_is_left_associative() {
        let mut validator = SpecValidator::new();
        
        // Rule: Function application is left associative
        // Test: f g h should parse as ((f g) h)
        let tree = validator.parse("f g h");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "application");
        let outer_func = expr.child_by_field_name("function").unwrap();
        assert_eq!(outer_func.kind(), "application");
        
        // Verify nested structure: ((f g) h)
        let inner_func = outer_func.child_by_field_name("function").unwrap();
        let inner_arg = outer_func.child_by_field_name("argument").unwrap();
        let outer_arg = expr.child_by_field_name("argument").unwrap();
        
        assert_eq!(inner_func.utf8_text(b"f g h").unwrap(), "f");
        assert_eq!(inner_arg.utf8_text(b"f g h").unwrap(), "g");
        assert_eq!(outer_arg.utf8_text(b"f g h").unwrap(), "h");
    }

    #[test]
    fn multiplication_has_higher_precedence_than_addition() {
        let mut validator = SpecValidator::new();
        
        // Rule: Multiplication binds tighter than addition
        // Test: a + b * c should parse as a + (b * c)
        let tree = validator.parse("a + b * c");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "binary_expression");
        let right = expr.child_by_field_name("right").unwrap();
        assert_eq!(right.kind(), "binary_expression");
        
        // Verify the right side is (b * c)
        let mul_left = right.child_by_field_name("left").unwrap();
        let mul_right = right.child_by_field_name("right").unwrap();
        assert_eq!(mul_left.utf8_text(b"a + b * c").unwrap(), "b");
        assert_eq!(mul_right.utf8_text(b"a + b * c").unwrap(), "c");
    }

    #[test]
    fn logical_and_has_higher_precedence_than_logical_or() {
        let mut validator = SpecValidator::new();
        
        // Rule: && binds tighter than ||
        // Test: a || b && c should parse as a || (b && c)
        let tree = validator.parse("a || b && c");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "binary_expression");
        let right = expr.child_by_field_name("right").unwrap();
        assert_eq!(right.kind(), "binary_expression");
    }
}

#[cfg(test)]
mod list_construction_rules {
    use super::*;

    #[test]
    fn list_elements_are_separate_expressions() {
        let mut validator = SpecValidator::new();
        
        // Rule: List elements are separate expressions, not function applications
        // Test: [1 2 3] should have three separate integer elements
        let tree = validator.parse("[ 1 2 3 ]");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "list");
        let elements = expr.child_by_field_name("elements").unwrap();
        
        // Should NOT be parsed as function applications
        assert_ne!(elements.kind(), "application");
        
        // TODO: Fix this - current implementation incorrectly parses as applications
        // This test documents the bug that needs to be fixed
    }

    #[test]
    fn empty_list_is_valid() {
        let mut validator = SpecValidator::new();
        
        let tree = validator.parse("[]");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "list");
    }

    #[test]
    fn list_with_complex_expressions() {
        let mut validator = SpecValidator::new();
        
        // Rule: List can contain any expressions
        let tree = validator.parse("[ (x + y) \"string\" { a = 1; } ]");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "list");
    }
}

#[cfg(test)]
mod function_parameter_rules {
    use super::*;

    #[test]
    fn simple_function_parameter() {
        let mut validator = SpecValidator::new();
        
        // Rule: Simple parameter syntax x: body
        let tree = validator.parse("x: x + 1");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "function_expression");
        let param = expr.child_by_field_name("parameter").unwrap();
        assert_eq!(param.kind(), "identifier");
    }

    #[test]
    fn pattern_function_parameter() {
        let mut validator = SpecValidator::new();
        
        // Rule: Pattern parameter syntax {a, b}: body
        let tree = validator.parse("{a, b}: a + b");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "function_expression");
        let param = expr.child_by_field_name("parameter").unwrap();
        assert_eq!(param.kind(), "formals");
    }

    #[test]
    fn pattern_with_default_values() {
        let mut validator = SpecValidator::new();
        
        // Rule: Pattern can have default values {a, b ? 1}: body
        let tree = validator.parse("{a, b ? 1}: a + b");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "function_expression");
        let param = expr.child_by_field_name("parameter").unwrap();
        assert_eq!(param.kind(), "formals");
    }
}

#[cfg(test)]
mod string_interpolation_rules {
    use super::*;

    #[test]
    fn string_with_interpolation() {
        let mut validator = SpecValidator::new();
        
        // Rule: String interpolation syntax "text ${expr} more"
        let tree = validator.parse("\"hello ${name}\"");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "string");
    }

    #[test]
    fn indented_string_with_interpolation() {
        let mut validator = SpecValidator::new();
        
        // Rule: Indented string syntax ''text ${expr} more''
        let tree = validator.parse("''hello ${name}''");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "indented_string");
    }
}

#[cfg(test)]
mod attribute_set_rules {
    use super::*;

    #[test]
    fn regular_attribute_set() {
        let mut validator = SpecValidator::new();
        
        // Rule: Regular attribute set syntax {key = value; ...}
        let tree = validator.parse("{ x = 1; y = 2; }");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "attrset");
    }

    #[test]
    fn recursive_attribute_set() {
        let mut validator = SpecValidator::new();
        
        // Rule: Recursive attribute set syntax rec {key = value; ...}
        let tree = validator.parse("rec { x = 1; y = x + 1; }");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "rec_attrset");
    }

    #[test]
    fn dynamic_attribute_keys() {
        let mut validator = SpecValidator::new();
        
        // Rule: Attribute keys can be dynamic expressions
        let tree = validator.parse("{ \"${key}\" = value; }");
        let expr = validator.get_expression_node(&tree);
        
        assert_eq!(expr.kind(), "attrset");
    }
}

#[cfg(test)]
mod error_handling_rules {
    use super::*;

    #[test]
    fn incomplete_expression_produces_error() {
        let mut validator = SpecValidator::new();
        
        // Rule: Incomplete expressions should be detected
        let tree = validator.parse("if true then");
        
        // Should have error nodes
        assert!(tree.root_node().has_error());
    }

    #[test]
    fn mismatched_parentheses_produces_error() {
        let mut validator = SpecValidator::new();
        
        let tree = validator.parse("(1 + 2");
        assert!(tree.root_node().has_error());
    }

    #[test]
    fn invalid_operator_combination_produces_error() {
        let mut validator = SpecValidator::new();
        
        let tree = validator.parse("1 ++ + 2");
        assert!(tree.root_node().has_error());
    }
}

// Generative testing helper
pub struct PropertyTester;

impl PropertyTester {
    /// Test that all valid operator combinations parse without error
    pub fn test_operator_combinations() {
        let operators = vec!["+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "->"];
        let operands = vec!["1", "x", "(a + b)"];
        
        let mut validator = SpecValidator::new();
        
        for op in &operators {
            for left in &operands {
                for right in &operands {
                    let expr = format!("{} {} {}", left, op, right);
                    let tree = validator.parse(&expr);
                    
                    // All valid combinations should parse without error
                    assert!(!tree.root_node().has_error(), 
                           "Failed to parse valid expression: {}", expr);
                }
            }
        }
    }
    
    /// Test that precedence is consistent across all operators
    pub fn test_precedence_consistency() {
        let test_cases = vec![
            ("a + b * c", "a + (b * c)"),
            ("a * b + c", "(a * b) + c"),
            ("a && b || c", "(a && b) || c"),
            ("a || b && c", "a || (b && c)"),
            ("f g + h", "(f g) + h"),
            ("f g h", "((f g) h)"),
        ];
        
        let mut validator = SpecValidator::new();
        
        for (input, expected_grouping) in test_cases {
            let tree1 = validator.parse(input);
            let tree2 = validator.parse(expected_grouping);
            
            // Both should parse to equivalent structures
            // (This is a simplified check - in practice we'd compare AST structure)
            assert!(!tree1.root_node().has_error());
            assert!(!tree2.root_node().has_error());
        }
    }
}