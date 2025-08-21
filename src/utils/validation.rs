//! Validation utilities

/// Generic validator for applying validation rules
/// 
/// Provides a framework for validating Nix expressions against
/// configurable rules and constraints.
pub struct Validator {}

/// A single validation rule that can check expressions
#[derive(Debug, Clone)]
pub struct ValidationRule {}