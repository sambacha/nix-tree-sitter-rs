//! AST transformation utilities for Nix code
//!
//! This module provides tools for transforming and manipulating
//! parsed Nix ASTs, including refactoring, optimization, and code generation.

pub mod refactor;
pub mod optimize;
pub mod codegen;
pub mod normalize;

pub use self::refactor::{Refactorer, RefactorRule, RefactorResult};
pub use self::optimize::{Optimizer, OptimizationPass, OptimizationResult};
pub use self::codegen::{CodeGenerator, GenerationContext};
pub use self::normalize::{Normalizer, NormalizationRule};

use crate::ast::Expression;
use crate::error::Result;

/// Main interface for AST transformations
///
/// Provides a unified interface for applying various transformations
/// to parsed Nix code.
pub struct Transformer {
    refactorer: Refactorer,
    optimizer: Optimizer,
    normalizer: Normalizer,
}

impl Transformer {
    /// Create a new transformer with default configuration
    pub fn new() -> Self {
        Self {
            refactorer: Refactorer::new(),
            optimizer: Optimizer::new(),
            normalizer: Normalizer::new(),
        }
    }
    
    /// Apply all transformations to an expression
    pub fn transform(&mut self, expression: Expression) -> Result<TransformResult> {
        let mut current = expression.clone();
        let mut steps = Vec::new();
        
        // Normalization (first pass)
        if let Ok(normalized) = self.normalizer.normalize(current.clone()) {
            if normalized != current {
                steps.push(TransformStep {
                    name: "normalize".to_string(),
                    description: "Normalize expression structure".to_string(),
                    before: current.clone(),
                    after: normalized.clone(),
                });
                current = normalized;
            }
        }
        
        // Optimization
        if let Ok(optimized) = self.optimizer.optimize(current.clone()) {
            if optimized.expression != current {
                steps.push(TransformStep {
                    name: "optimize".to_string(),
                    description: optimized.description,
                    before: current.clone(),
                    after: optimized.expression.clone(),
                });
                current = optimized.expression;
            }
        }
        
        // Refactoring (if any rules are enabled)
        if let Ok(refactored) = self.refactorer.refactor(current.clone()) {
            for result in refactored {
                if result.after != result.before {
                    steps.push(TransformStep {
                        name: result.rule_name,
                        description: result.description,
                        before: result.before,
                        after: result.after.clone(),
                    });
                    current = result.after;
                }
            }
        }
        
        Ok(TransformResult {
            original: expression,
            transformed: current,
            steps,
        })
    }
    
    /// Configure the transformer
    pub fn with_config(mut self, config: TransformerConfig) -> Self {
        if let Some(refactor_config) = config.refactor {
            self.refactorer = self.refactorer.with_config(refactor_config);
        }
        if let Some(optimizer_config) = config.optimizer {
            self.optimizer = self.optimizer.with_config(optimizer_config);
        }
        if let Some(normalizer_config) = config.normalizer {
            self.normalizer = self.normalizer.with_config(normalizer_config);
        }
        self
    }
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the transformer
#[derive(Debug, Clone, Default)]
pub struct TransformerConfig {
    /// Refactoring configuration
    pub refactor: Option<refactor::Config>,
    
    /// Optimization configuration
    pub optimizer: Option<optimize::Config>,
    
    /// Normalization configuration
    pub normalizer: Option<normalize::Config>,
}

/// Result of applying transformations
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// Original expression before transformation
    pub original: Expression,
    
    /// Final transformed expression
    pub transformed: Expression,
    
    /// Steps taken during transformation
    pub steps: Vec<TransformStep>,
}

impl TransformResult {
    /// Check if any transformations were applied
    pub fn was_transformed(&self) -> bool {
        !self.steps.is_empty()
    }
    
    /// Get a summary of transformations applied
    pub fn summary(&self) -> String {
        if self.steps.is_empty() {
            "No transformations applied".to_string()
        } else {
            let names: Vec<String> = self.steps.iter()
                .map(|s| s.name.clone())
                .collect();
            format!("Applied {} transformation(s): {}", 
                   self.steps.len(),
                   names.join(", "))
        }
    }
}

/// A single transformation step
#[derive(Debug, Clone)]
pub struct TransformStep {
    /// Name of the transformation rule
    pub name: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Expression before transformation
    pub before: Expression,
    
    /// Expression after transformation
    pub after: Expression,
}

/// Trait for transformation passes
///
/// Implementors can define custom transformation logic
/// that can be applied to Nix ASTs.
pub trait TransformPass {
    /// Apply the transformation to an expression
    fn apply(&mut self, expression: Expression) -> Result<Expression>;
    
    /// Get the name of this transformation pass
    fn name(&self) -> &str;
    
    /// Get a description of what this pass does
    fn description(&self) -> &str;
    
    /// Check if this pass should be applied to the given expression
    fn should_apply(&self, expression: &Expression) -> bool {
        // By default, try to apply to all expressions
        let _ = expression;
        true
    }
}

/// Utility functions for working with transformations
pub mod utils {
    use super::*;
    
    /// Apply a transformation pass to an expression
    pub fn apply_pass<P: TransformPass>(
        mut pass: P, 
        expression: Expression
    ) -> Result<Option<TransformStep>> {
        if !pass.should_apply(&expression) {
            return Ok(None);
        }
        
        let before = expression.clone();
        match pass.apply(expression) {
            Ok(after) => {
                if after == before {
                    Ok(None)
                } else {
                    Ok(Some(TransformStep {
                        name: pass.name().to_string(),
                        description: pass.description().to_string(),
                        before,
                        after,
                    }))
                }
            }
            Err(e) => Err(e),
        }
    }
    
    /// Check if two expressions are structurally equivalent
    pub fn expressions_equal(a: &Expression, b: &Expression) -> bool {
        // This would need to implement deep structural comparison
        // For now, use a simple comparison
        std::ptr::eq(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_creation() {
        let transformer = Transformer::new();
        drop(transformer);
    }

    #[test]
    fn test_transform_result() {
        use crate::ast::Expression;
        
        let expr = Expression::Integer(42);
        let result = TransformResult {
            original: expr.clone(),
            transformed: expr.clone(),
            steps: Vec::new(),
        };
        
        assert!(!result.was_transformed());
        assert_eq!(result.summary(), "No transformations applied");
    }
}