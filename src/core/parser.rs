use std::sync::Arc;
use tree_sitter::{Parser, Tree, Node as TSNode, Language};
use crate::ast::{Expression, Node, BinaryOperator, Attribute, Binding, Parameter, PatternField, PathType, StringPart};
use crate::error::{ParseError, Result};
use crate::plugins::Plugin;

#[cfg(feature = "cache")]
use lru::LruCache;
#[cfg(feature = "cache")]
use std::sync::Mutex;

/// The main Nix parser implementation
pub struct NixParser {
    parser: Parser,
    language: Language,
    plugins: Vec<Box<dyn Plugin>>,
    #[cfg(feature = "cache")]
    cache: Arc<Mutex<LruCache<String, Tree>>>,
}

impl NixParser {
    /// Create a new parser instance
    pub fn new() -> Result<Self> {
        let language = unsafe { tree_sitter_nix() };
        let mut parser = Parser::new();
        parser.set_language(&language)
            .map_err(|e| ParseError::LanguageError(e.to_string()))?;
        
        Ok(Self {
            parser,
            language,
            plugins: Vec::new(),
            #[cfg(feature = "cache")]
            cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(100).unwrap()
            ))),
        })
    }

    /// Parse Nix source code
    pub fn parse(&mut self, source: &str) -> Result<ParseResult> {
        // Check cache first
        #[cfg(feature = "cache")]
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(tree) = cache.get(source) {
                return Ok(ParseResult {
                    tree: tree.clone(),
                    source: source.to_string(),
                    diagnostics: Vec::new(),
                });
            }
        }

        // Run pre-processing plugins
        let processed = self.run_pre_plugins(source)?;
        
        // Parse with Tree-sitter
        let tree = self.parser.parse(&processed, None)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse input".into()))?;
        
        // Cache the result
        #[cfg(feature = "cache")]
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(source.to_string(), tree.clone());
        }
        
        // Run post-processing plugins
        let tree = self.run_post_plugins(tree)?;
        
        // Collect diagnostics
        let diagnostics = self.collect_diagnostics(&tree);
        
        Ok(ParseResult {
            tree,
            source: processed,
            diagnostics,
        })
    }

    /// Parse incrementally with edits
    pub fn parse_incremental(
        &mut self,
        old_tree: &Tree,
        source: &str,
        edits: &[Edit],
    ) -> Result<ParseResult> {
        let mut tree = old_tree.clone();
        
        for edit in edits {
            tree.edit(&edit.to_tree_sitter_edit());
        }
        
        let tree = self.parser.parse(source, Some(&tree))
            .ok_or_else(|| ParseError::ParseFailed("Incremental parse failed".into()))?;
        
        let diagnostics = self.collect_diagnostics(&tree);
        
        Ok(ParseResult {
            tree,
            source: source.to_string(),
            diagnostics,
        })
    }

    /// Add a plugin to the parser
    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Convert Tree-sitter tree to AST
    pub fn to_ast(&self, tree: &Tree, source: &str) -> Result<Expression> {
        let root = tree.root_node();
        self.node_to_ast(root, source)
    }

    fn node_to_ast(&self, node: TSNode, source: &str) -> Result<Expression> {
        match node.kind() {
            // Entry points and wrappers
            "source_file" | "expr" | "paren_expr" | "literal" => {
                let child = node.child(0)
                    .ok_or_else(|| ParseError::InvalidNode("Empty expression".into()))?;
                self.node_to_ast(child, source)
            }
            
            // Literals
            "integer" => {
                let text = node.utf8_text(source.as_bytes())
                    .map_err(|e| ParseError::Utf8Error(e))?;
                let value = text.parse::<i64>()
                    .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
                Ok(Expression::Integer(value))
            }
            "float" => {
                let text = node.utf8_text(source.as_bytes())
                    .map_err(|e| ParseError::Utf8Error(e))?;
                let value = text.parse::<f64>()
                    .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
                Ok(Expression::Float(value))
            }
            "string" => {
                // Parse string with external scanner tokens
                let mut parts = Vec::new();
                let mut has_interpolation = false;
                
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        match child.kind() {
                            "string_start" | "string_end" | "indented_string_start" | "indented_string_end" => {
                                // Skip start/end tokens
                                continue;
                            }
                            "string_content" | "indented_string_content" => {
                                let text = child.utf8_text(source.as_bytes())
                                    .map_err(|e| ParseError::Utf8Error(e))?;
                                if !text.is_empty() {
                                    parts.push(StringPart::Literal(text.to_string()));
                                }
                            }
                            "interpolation" => {
                                has_interpolation = true;
                                // Get the expression inside interpolation
                                if let Some(expr_node) = child.child(1) { // Skip interpolation_start token
                                    let expr = self.node_to_ast(expr_node, source)?;
                                    parts.push(StringPart::Interpolation(Box::new(expr)));
                                }
                            }
                            _ => {
                                // Handle any other content
                                let text = child.utf8_text(source.as_bytes())
                                    .map_err(|e| ParseError::Utf8Error(e))?;
                                if !text.is_empty() {
                                    parts.push(StringPart::Literal(text.to_string()));
                                }
                            }
                        }
                    }
                }
                
                if has_interpolation {
                    Ok(Expression::StringInterpolation { parts })
                } else if parts.is_empty() {
                    // Empty string
                    Ok(Expression::String(String::new()))
                } else {
                    // Simple string - concatenate all literal parts
                    let mut result = String::new();
                    for part in parts {
                        if let StringPart::Literal(s) = part {
                            result.push_str(&s);
                        }
                    }
                    Ok(Expression::String(result))
                }
            }
            "boolean" => {
                let text = node.utf8_text(source.as_bytes())
                    .map_err(|e| ParseError::Utf8Error(e))?;
                Ok(Expression::Boolean(text == "true"))
            }
            "null" => Ok(Expression::Null),
            "identifier" => {
                let text = node.utf8_text(source.as_bytes())
                    .map_err(|e| ParseError::Utf8Error(e))?;
                Ok(Expression::Identifier(text.to_string()))
            }
            "path" => {
                // Check children for search_path_token or regular path
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        match child.kind() {
                            "search_path_token" => {
                                let text = child.utf8_text(source.as_bytes())
                                    .map_err(|e| ParseError::Utf8Error(e))?;
                                let inner = &text[1..text.len()-1]; // Remove < >
                                return Ok(Expression::Path(PathType::Search(inner.to_string())));
                            }
                            _ => {
                                let text = child.utf8_text(source.as_bytes())
                                    .map_err(|e| ParseError::Utf8Error(e))?;
                                let path_type = if text.starts_with("/") {
                                    PathType::Absolute(text.to_string())
                                } else if text.starts_with("~/") {
                                    PathType::Home(text.to_string())
                                } else if text.starts_with("<") && text.ends_with(">") {
                                    PathType::Search(text[1..text.len()-1].to_string())
                                } else {
                                    PathType::Relative(text.to_string())
                                };
                                return Ok(Expression::Path(path_type));
                            }
                        }
                    }
                }
                // Fallback for direct path node
                let text = node.utf8_text(source.as_bytes())
                    .map_err(|e| ParseError::Utf8Error(e))?;
                let path_type = if text.starts_with("/") {
                    PathType::Absolute(text.to_string())
                } else if text.starts_with("~/") {
                    PathType::Home(text.to_string())
                } else {
                    PathType::Relative(text.to_string())
                };
                Ok(Expression::Path(path_type))
            }
            
            // Collections
            "list" => {
                let mut elements = Vec::new();
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        // Accept both list_element and expr for backward compatibility
                        if child.kind() == "list_element" || child.kind() == "expr" {
                            elements.push(self.node_to_ast(child, source)?);
                        }
                    }
                }
                Ok(Expression::List(elements))
            }
            "list_element" => {
                // list_element is a wrapper, parse its child
                let child = node.child(0)
                    .ok_or_else(|| ParseError::InvalidNode("Empty list element".into()))?;
                self.node_to_ast(child, source)
            }
            "attrset" => {
                let mut recursive = false;
                let mut attributes = Vec::new();
                
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        match child.kind() {
                            "rec" => recursive = true,
                            "attr" => {
                                // Parse attribute
                                if let Some(name_node) = child.child_by_field_name("name") {
                                    if let Some(value_node) = child.child_by_field_name("value") {
                                        let name = name_node.utf8_text(source.as_bytes())
                                            .map_err(|e| ParseError::Utf8Error(e))?;
                                        let value = self.node_to_ast(value_node, source)?;
                                        attributes.push(Attribute {
                                            path: vec![name.to_string()],
                                            value,
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                Ok(Expression::AttributeSet { recursive, attributes })
            }
            
            // Binary operations
            "binary" => {
                if let Some(left_node) = node.child_by_field_name("left") {
                    if let Some(right_node) = node.child_by_field_name("right") {
                        // Find the operator between left and right
                        let mut op_text = None;
                        for i in 0..node.child_count() {
                            if let Some(child) = node.child(i) {
                                let text = child.utf8_text(source.as_bytes())
                                    .map_err(|e| ParseError::Utf8Error(e))?;
                                if matches!(text, "+" | "-" | "*" | "/" | "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" | "++" | "//") {
                                    op_text = Some(text);
                                    break;
                                }
                            }
                        }
                        
                        if let Some(op_str) = op_text {
                            let left = Box::new(self.node_to_ast(left_node, source)?);
                            let right = Box::new(self.node_to_ast(right_node, source)?);
                            
                            let op = match op_str {
                                "+" => BinaryOperator::Add,
                                "-" => BinaryOperator::Subtract,
                                "*" => BinaryOperator::Multiply,
                                "/" => BinaryOperator::Divide,
                                "==" => BinaryOperator::Equal,
                                "!=" => BinaryOperator::NotEqual,
                                "<" => BinaryOperator::Less,
                                "<=" => BinaryOperator::LessEqual,
                                ">" => BinaryOperator::Greater,
                                ">=" => BinaryOperator::GreaterEqual,
                                "&&" => BinaryOperator::And,
                                "||" => BinaryOperator::Or,
                                "++" => BinaryOperator::Concat,
                                "//" => BinaryOperator::Update,
                                _ => return Err(ParseError::UnknownNodeType(
                                    format!("Unknown operator: {}", op_str)
                                )),
                            };
                            
                            return Ok(Expression::BinaryOp { op, left, right });
                        }
                    }
                }
                Err(ParseError::InvalidNode("Invalid binary operation".into()))
            }
            
            // Selection
            "select" => {
                if let Some(obj_node) = node.child_by_field_name("obj") {
                    if let Some(path_node) = node.child_by_field_name("path") {
                        let expr = Box::new(self.node_to_ast(obj_node, source)?);
                        let path_text = path_node.utf8_text(source.as_bytes())
                            .map_err(|e| ParseError::Utf8Error(e))?;
                        let path: Vec<String> = path_text.split('.').map(|s| s.to_string()).collect();
                        
                        return Ok(Expression::Select {
                            expr,
                            path,
                            default: None,
                        });
                    }
                }
                Err(ParseError::InvalidNode("Invalid selection".into()))
            }
            
            // Application
            "apply" => {
                if let Some(fn_node) = node.child_by_field_name("fn") {
                    if let Some(arg_node) = node.child_by_field_name("arg") {
                        let function = Box::new(self.node_to_ast(fn_node, source)?);
                        let argument = Box::new(self.node_to_ast(arg_node, source)?);
                        
                        return Ok(Expression::Application { function, argument });
                    }
                }
                Err(ParseError::InvalidNode("Invalid application".into()))
            }
            
            // Let expression
            "let_expr" => {
                let mut bindings = Vec::new();
                let mut body = None;
                
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        match child.kind() {
                            "binding" => {
                                if let Some(name_node) = child.child_by_field_name("name") {
                                    if let Some(value_node) = child.child_by_field_name("value") {
                                        let name = name_node.utf8_text(source.as_bytes())
                                            .map_err(|e| ParseError::Utf8Error(e))?;
                                        let value = self.node_to_ast(value_node, source)?;
                                        bindings.push(Binding {
                                            name: name.to_string(),
                                            value,
                                            inherit: false,
                                            from: None,
                                        });
                                    }
                                }
                            }
                            _ => {
                                if child.kind() != "let" && child.kind() != "in" && child.kind() != ";" {
                                    body = Some(Box::new(self.node_to_ast(child, source)?));
                                }
                            }
                        }
                    }
                }
                
                if let Some(body_node) = node.child_by_field_name("body") {
                    body = Some(Box::new(self.node_to_ast(body_node, source)?));
                }
                
                Ok(Expression::LetIn {
                    bindings,
                    body: body.ok_or_else(|| ParseError::InvalidNode("Let expression missing body".into()))?,
                })
            }
            
            // If expression
            "if_expr" => {
                let condition = node.child_by_field_name("cond")
                    .ok_or_else(|| ParseError::InvalidNode("If missing condition".into()))?;
                let then_branch = node.child_by_field_name("then_expr")
                    .ok_or_else(|| ParseError::InvalidNode("If missing then branch".into()))?;
                let else_branch = node.child_by_field_name("else_expr")
                    .ok_or_else(|| ParseError::InvalidNode("If missing else branch".into()))?;
                
                Ok(Expression::If {
                    condition: Box::new(self.node_to_ast(condition, source)?),
                    then_branch: Box::new(self.node_to_ast(then_branch, source)?),
                    else_branch: Box::new(self.node_to_ast(else_branch, source)?),
                })
            }
            
            // Function
            "function" => {
                let param_node = node.child_by_field_name("param")
                    .ok_or_else(|| ParseError::InvalidNode("Function missing parameter".into()))?;
                let body_node = node.child_by_field_name("body")
                    .ok_or_else(|| ParseError::InvalidNode("Function missing body".into()))?;
                
                let parameter = match param_node.kind() {
                    "param" => {
                        // param is a wrapper, get the actual content
                        if let Some(child) = param_node.child(0) {
                            match child.kind() {
                                "identifier" => {
                                    let text = child.utf8_text(source.as_bytes())
                                        .map_err(|e| ParseError::Utf8Error(e))?;
                                    Parameter::Identifier(text.to_string())
                                }
                                "pattern" => {
                                    // Handle pattern parameters through the child
                                    self.parse_pattern_parameter(child, source)?
                                }
                                _ => return Err(ParseError::InvalidNode(format!("Unexpected param child: {}", child.kind())))
                            }
                        } else {
                            return Err(ParseError::InvalidNode("Empty param node".into()));
                        }
                    }
                    "identifier" => {
                        let text = param_node.utf8_text(source.as_bytes())
                            .map_err(|e| ParseError::Utf8Error(e))?;
                        Parameter::Identifier(text.to_string())
                    }
                    "pattern" => {
                        self.parse_pattern_parameter(param_node, source)?
                    }
                    _ => Parameter::Identifier("_".to_string()), // Fallback
                };
                
                Ok(Expression::Function {
                    parameter,
                    body: Box::new(self.node_to_ast(body_node, source)?),
                })
            }
            
            // External scanner keywords
            "with_keyword" => Ok(Expression::Identifier("with".to_string())),
            "import_keyword" => Ok(Expression::Identifier("import".to_string())),
            "assert_keyword" => Ok(Expression::Identifier("assert".to_string())),
            "inherit_keyword" => Ok(Expression::Identifier("inherit".to_string())),
            
            // With expression
            "with_expr" => {
                let namespace_node = node.child_by_field_name("namespace")
                    .ok_or_else(|| ParseError::InvalidNode("With missing namespace".into()))?;
                let body_node = node.child_by_field_name("body")
                    .ok_or_else(|| ParseError::InvalidNode("With missing body".into()))?;
                
                Ok(Expression::With {
                    scope: Box::new(self.node_to_ast(namespace_node, source)?),
                    body: Box::new(self.node_to_ast(body_node, source)?),
                })
            }
            
            // Import expression
            "import_expr" => {
                let path_node = node.child_by_field_name("path")
                    .ok_or_else(|| ParseError::InvalidNode("Import missing path".into()))?;
                
                Ok(Expression::Import {
                    path: Box::new(self.node_to_ast(path_node, source)?),
                })
            }
            
            // Assert expression
            "assert_expr" => {
                let condition_node = node.child_by_field_name("condition")
                    .ok_or_else(|| ParseError::InvalidNode("Assert missing condition".into()))?;
                let body_node = node.child_by_field_name("body")
                    .ok_or_else(|| ParseError::InvalidNode("Assert missing body".into()))?;
                
                Ok(Expression::Assert {
                    condition: Box::new(self.node_to_ast(condition_node, source)?),
                    body: Box::new(self.node_to_ast(body_node, source)?),
                })
            }
            
            // Inherit statement
            "inherit" => {
                let source_expr = if let Some(source_node) = node.child_by_field_name("source") {
                    // Extract expression from (expr) parentheses
                    if let Some(expr_node) = source_node.child(1) { // Skip '(' 
                        Some(Box::new(self.node_to_ast(expr_node, source)?))
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                let attrs_node = node.child_by_field_name("attrs")
                    .ok_or_else(|| ParseError::InvalidNode("Inherit missing attributes".into()))?;
                
                let mut attributes = Vec::new();
                for i in 0..attrs_node.child_count() {
                    if let Some(attr_node) = attrs_node.child(i) {
                        if attr_node.kind() == "identifier" {
                            let attr_text = attr_node.utf8_text(source.as_bytes())
                                .map_err(|e| ParseError::Utf8Error(e))?;
                            attributes.push(attr_text.to_string());
                        }
                    }
                }
                
                Ok(Expression::Inherit {
                    source: source_expr,
                    attributes,
                })
            }
            
            _ => Err(ParseError::UnknownNodeType(node.kind().to_string())),
        }
    }

    fn run_pre_plugins(&mut self, source: &str) -> Result<String> {
        let mut result = source.to_string();
        for plugin in &mut self.plugins {
            result = plugin.pre_process(result)?;
        }
        Ok(result)
    }

    fn run_post_plugins(&mut self, tree: Tree) -> Result<Tree> {
        let mut result = tree;
        for plugin in &mut self.plugins {
            result = plugin.post_process(result)?;
        }
        Ok(result)
    }

    fn collect_diagnostics(&self, tree: &Tree) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let root = tree.root_node();
        self.collect_errors(root, &mut diagnostics);
        diagnostics
    }

    fn collect_errors(&self, node: TSNode, diagnostics: &mut Vec<Diagnostic>) {
        if node.is_error() {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: format!("Syntax error at {}-{}", 
                    node.start_position().row,
                    node.start_position().column),
                location: SourceLocation {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    start_position: (node.start_position().row, node.start_position().column),
                    end_position: (node.end_position().row, node.end_position().column),
                },
            });
        }
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_errors(child, diagnostics);
            }
        }
    }
    
    fn parse_pattern_parameter(&self, node: TSNode, source: &str) -> Result<Parameter> {
        let mut fields = Vec::new();
        let mut ellipsis = false;
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "pattern_elem" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = name_node.utf8_text(source.as_bytes())
                                .map_err(|e| ParseError::Utf8Error(e))?;
                            let default = if let Some(default_node) = child.child_by_field_name("default") {
                                Some(self.node_to_ast(default_node, source)?)
                            } else {
                                None
                            };
                            fields.push(PatternField {
                                name: name.to_string(),
                                default,
                            });
                        }
                    }
                    "..." => ellipsis = true,
                    _ => {}
                }
            }
        }
        
        Ok(Parameter::Pattern { fields, ellipsis, bind: None })
    }
}

impl Default for NixParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default parser")
    }
}

/// Result of parsing
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub tree: Tree,
    pub source: String,
    pub diagnostics: Vec<Diagnostic>,
}

/// Diagnostic information
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: SourceLocation,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Source location information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_position: (usize, usize),
    pub end_position: (usize, usize),
}

/// Edit operation for incremental parsing
#[derive(Debug, Clone)]
pub struct Edit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_position: (usize, usize),
    pub old_end_position: (usize, usize),
    pub new_end_position: (usize, usize),
}

impl Edit {
    fn to_tree_sitter_edit(&self) -> tree_sitter::InputEdit {
        tree_sitter::InputEdit {
            start_byte: self.start_byte,
            old_end_byte: self.old_end_byte,
            new_end_byte: self.new_end_byte,
            start_position: tree_sitter::Point {
                row: self.start_position.0,
                column: self.start_position.1,
            },
            old_end_position: tree_sitter::Point {
                row: self.old_end_position.0,
                column: self.old_end_position.1,
            },
            new_end_position: tree_sitter::Point {
                row: self.new_end_position.0,
                column: self.new_end_position.1,
            },
        }
    }
}

// External C function declaration for Tree-sitter language
extern "C" {
    fn tree_sitter_nix() -> Language;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = NixParser::new();
        assert!(parser.is_ok());
    }
}