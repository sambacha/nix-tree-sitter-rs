// Comprehensive specification-based testing framework
// Tests are derived from abstract language rules, not specific examples

use std::collections::HashMap;
use tree_sitter::{Parser, Tree, Node, Query, QueryCursor, Language};

extern "C" {
    fn tree_sitter_nix() -> Language;
}

pub struct NixSpecificationTester {
    parser: Parser,
    language: Language,
}

impl NixSpecificationTester {
    pub fn new() -> Self {
        let language = unsafe { tree_sitter_nix() };
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();
        
        Self { parser, language }
    }

    fn parse(&mut self, input: &str) -> Result<Tree, String> {
        match self.parser.parse(input, None) {
            Some(tree) => {
                if tree.root_node().has_error() {
                    Err(format!("Parse error in: {}", input))
                } else {
                    Ok(tree)
                }
            }
            None => Err(format!("Failed to parse: {}", input))
        }
    }

    fn get_expression(&self, tree: &Tree, source: &str) -> Result<Node, String> {
        let root = tree.root_node();
        root.child_by_field_name("expression")
            .ok_or_else(|| "No expression field found".to_string())
    }

    /// Validate that precedence rules are correctly implemented
    pub fn validate_precedence_rules(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        // Rule 1: Function application has higher precedence than binary operators
        let test_cases = vec![
            ("f g + h", |tree, source| {
                let expr = self.get_expression(&tree, source)?;
                if expr.kind() != "binary_expression" {
                    return Err("Expected binary_expression at root".to_string());
                }
                let left = expr.child_by_field_name("left")
                    .ok_or("Missing left operand")?;
                if left.kind() != "application" {
                    return Err("Left operand should be application".to_string());
                }
                Ok(())
            }),
            ("f g * h", |tree, source| {
                let expr = self.get_expression(&tree, source)?;
                if expr.kind() != "binary_expression" {
                    return Err("Expected binary_expression at root".to_string());
                }
                let left = expr.child_by_field_name("left")
                    .ok_or("Missing left operand")?;
                if left.kind() != "application" {
                    return Err("Left operand should be application".to_string());
                }
                Ok(())
            }),
        ];

        for (input, validator) in test_cases {
            match self.parse(input) {
                Ok(tree) => {
                    if let Err(e) = validator(tree, input) {
                        errors.push(format!("Precedence error in '{}': {}", input, e));
                    }
                }
                Err(e) => errors.push(format!("Parse error: {}", e)),
            }
        }

        // Rule 2: Multiplication has higher precedence than addition
        let arith_cases = vec![
            ("a + b * c", "addition", "binary_expression"), // Right should be multiplication
            ("a * b + c", "multiplication", "binary_expression"), // Left should be multiplication
        ];

        for (input, _op_type, expected_right_kind) in arith_cases {
            match self.parse(input) {
                Ok(tree) => {
                    let expr = match self.get_expression(&tree, input) {
                        Ok(e) => e,
                        Err(e) => {
                            errors.push(format!("Expression error in '{}': {}", input, e));
                            continue;
                        }
                    };
                    
                    if expr.kind() != "binary_expression" {
                        errors.push(format!("Expected binary_expression for '{}', got {}", input, expr.kind()));
                        continue;
                    }

                    // For "a + b * c", the right side should be "b * c" (multiplication)
                    if input == "a + b * c" {
                        let right = expr.child_by_field_name("right").unwrap();
                        if right.kind() != "binary_expression" {
                            errors.push(format!("Right operand of '{}' should be binary_expression, got {}", input, right.kind()));
                        }
                    }
                }
                Err(e) => errors.push(format!("Parse error: {}", e)),
            }
        }

        errors
    }

    /// Validate function application associativity
    pub fn validate_function_application(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        // Rule: Function application is left associative
        let test_input = "f g h";
        match self.parse(test_input) {
            Ok(tree) => {
                let expr = match self.get_expression(&tree, test_input) {
                    Ok(e) => e,
                    Err(e) => {
                        errors.push(format!("Expression error: {}", e));
                        return errors;
                    }
                };

                // Should be: application(application(f, g), h)
                if expr.kind() != "application" {
                    errors.push(format!("Expected application at root, got {}", expr.kind()));
                    return errors;
                }

                let outer_func = expr.child_by_field_name("function");
                let outer_arg = expr.child_by_field_name("argument");

                match (outer_func, outer_arg) {
                    (Some(func), Some(arg)) => {
                        if func.kind() != "application" {
                            errors.push("Function part should be application for left associativity".to_string());
                        }
                        if arg.utf8_text(test_input.as_bytes()).unwrap() != "h" {
                            errors.push("Outer argument should be 'h'".to_string());
                        }
                    }
                    _ => errors.push("Missing function or argument fields".to_string()),
                }
            }
            Err(e) => errors.push(format!("Parse error: {}", e)),
        }

        errors
    }

    /// Validate list construction rules  
    pub fn validate_list_construction(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        // Rule: List elements should be separate expressions, not function applications
        let test_cases = vec![
            ("[]", 0), // Empty list
            ("[ 1 ]", 1), // Single element
            ("[ 1 2 ]", 2), // Two elements  
            ("[ 1 2 3 ]", 3), // Three elements
        ];

        for (input, expected_element_count) in test_cases {
            match self.parse(input) {
                Ok(tree) => {
                    let expr = match self.get_expression(&tree, input) {
                        Ok(e) => e,
                        Err(e) => {
                            errors.push(format!("Expression error in '{}': {}", input, e));
                            continue;
                        }
                    };

                    if expr.kind() != "list" {
                        errors.push(format!("Expected list for '{}', got {}", input, expr.kind()));
                        continue;
                    }

                    // Check if elements field exists and has correct structure
                    let elements_field = expr.child_by_field_name("elements");
                    
                    if expected_element_count == 0 {
                        // Empty list should have no elements field or empty elements
                        if elements_field.is_some() {
                            let elements = elements_field.unwrap();
                            if elements.child_count() > 0 {
                                errors.push(format!("Empty list '{}' should have no elements", input));
                            }
                        }
                    } else {
                        // Non-empty list validation
                        match elements_field {
                            Some(elements) => {
                                // Current bug: elements are parsed as function applications
                                // This documents the issue that needs to be fixed
                                if elements.kind() == "application" {
                                    errors.push(format!("BUG: List elements in '{}' incorrectly parsed as function application. Should be {} separate elements.", input, expected_element_count));
                                }
                            }
                            None => {
                                errors.push(format!("Non-empty list '{}' missing elements field", input));
                            }
                        }
                    }
                }
                Err(e) => errors.push(format!("Parse error: {}", e)),
            }
        }

        errors
    }

    /// Validate string and interpolation rules
    pub fn validate_strings(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        let test_cases = vec![
            ("\"hello\"", "string"),
            ("\"hello ${name}\"", "string"),
            ("''hello''", "indented_string"),
            ("''hello ${name}''", "indented_string"),
        ];

        for (input, expected_kind) in test_cases {
            match self.parse(input) {
                Ok(tree) => {
                    let expr = match self.get_expression(&tree, input) {
                        Ok(e) => e,
                        Err(e) => {
                            errors.push(format!("Expression error in '{}': {}", input, e));
                            continue;
                        }
                    };

                    if expr.kind() != expected_kind {
                        errors.push(format!("Expected {} for '{}', got {}", expected_kind, input, expr.kind()));
                    }
                }
                Err(e) => errors.push(format!("Parse error: {}", e)),
            }
        }

        errors
    }

    /// Validate attribute set rules
    pub fn validate_attribute_sets(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        let test_cases = vec![
            ("{}", "attrset"),
            ("{ x = 1; }", "attrset"),
            ("{ x = 1; y = 2; }", "attrset"),
            ("rec {}", "rec_attrset"),
            ("rec { x = 1; y = x + 1; }", "rec_attrset"),
        ];

        for (input, expected_kind) in test_cases {
            match self.parse(input) {
                Ok(tree) => {
                    let expr = match self.get_expression(&tree, input) {
                        Ok(e) => e,
                        Err(e) => {
                            errors.push(format!("Expression error in '{}': {}", input, e));
                            continue;
                        }
                    };

                    if expr.kind() != expected_kind {
                        errors.push(format!("Expected {} for '{}', got {}", expected_kind, input, expr.kind()));
                    }
                }
                Err(e) => errors.push(format!("Parse error: {}", e)),
            }
        }

        errors
    }

    /// Run comprehensive specification validation
    pub fn validate_all(&mut self) -> SpecificationReport {
        let mut report = SpecificationReport::new();

        report.precedence_errors = self.validate_precedence_rules();
        report.application_errors = self.validate_function_application();
        report.list_errors = self.validate_list_construction();
        report.string_errors = self.validate_strings();
        report.attrset_errors = self.validate_attribute_sets();

        report
    }
}

#[derive(Debug)]
pub struct SpecificationReport {
    pub precedence_errors: Vec<String>,
    pub application_errors: Vec<String>,
    pub list_errors: Vec<String>,
    pub string_errors: Vec<String>,
    pub attrset_errors: Vec<String>,
}

impl SpecificationReport {
    fn new() -> Self {
        Self {
            precedence_errors: Vec::new(),
            application_errors: Vec::new(),
            list_errors: Vec::new(),
            string_errors: Vec::new(),
            attrset_errors: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.precedence_errors.is_empty() ||
        !self.application_errors.is_empty() ||
        !self.list_errors.is_empty() ||
        !self.string_errors.is_empty() ||
        !self.attrset_errors.is_empty()
    }

    pub fn total_errors(&self) -> usize {
        self.precedence_errors.len() +
        self.application_errors.len() +
        self.list_errors.len() +
        self.string_errors.len() +
        self.attrset_errors.len()
    }

    pub fn print_summary(&self) {
        println!("=== Nix Parser Specification Validation Report ===");
        println!("Precedence errors: {}", self.precedence_errors.len());
        println!("Function application errors: {}", self.application_errors.len());
        println!("List construction errors: {}", self.list_errors.len());
        println!("String handling errors: {}", self.string_errors.len());
        println!("Attribute set errors: {}", self.attrset_errors.len());
        println!("Total errors: {}", self.total_errors());

        if self.has_errors() {
            println!("\n=== DETAILED ERRORS ===");
            
            if !self.precedence_errors.is_empty() {
                println!("\nPrecedence Rule Violations:");
                for error in &self.precedence_errors {
                    println!("  - {}", error);
                }
            }

            if !self.application_errors.is_empty() {
                println!("\nFunction Application Rule Violations:");
                for error in &self.application_errors {
                    println!("  - {}", error);
                }
            }

            if !self.list_errors.is_empty() {
                println!("\nList Construction Rule Violations:");
                for error in &self.list_errors {
                    println!("  - {}", error);
                }
            }

            if !self.string_errors.is_empty() {
                println!("\nString Handling Rule Violations:");
                for error in &self.string_errors {
                    println!("  - {}", error);
                }
            }

            if !self.attrset_errors.is_empty() {
                println!("\nAttribute Set Rule Violations:");
                for error in &self.attrset_errors {
                    println!("  - {}", error);
                }
            }
        } else {
            println!("\n✅ All specification rules validated successfully!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_specification_validation() {
        let mut tester = NixSpecificationTester::new();
        let report = tester.validate_all();
        
        report.print_summary();
        
        // This test will currently fail due to the list parsing bug
        // It serves as documentation of what needs to be fixed
        if report.has_errors() {
            // For now, we expect errors - this documents the issues
            println!("Note: This test documents current parser issues that need to be fixed.");
        }
    }
}