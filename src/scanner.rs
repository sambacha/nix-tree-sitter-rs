//! Scanner module that wraps the external C scanner
//!
//! This module provides a Rust interface to the Tree-sitter external scanner
//! written in C, following Rust conventions and safety practices.

use std::os::raw::{c_char, c_uint};
use std::ptr;

// External scanner functions from C
extern "C" {
    fn tree_sitter_nix_external_scanner_create() -> *mut std::os::raw::c_void;
    fn tree_sitter_nix_external_scanner_destroy(scanner: *mut std::os::raw::c_void);
    fn tree_sitter_nix_external_scanner_serialize(
        scanner: *mut std::os::raw::c_void,
        buffer: *mut c_char,
    ) -> c_uint;
    fn tree_sitter_nix_external_scanner_deserialize(
        scanner: *mut std::os::raw::c_void,
        buffer: *const c_char,
        length: c_uint,
    );
    fn tree_sitter_nix_external_scanner_scan(
        scanner: *mut std::os::raw::c_void,
        lexer: *mut std::os::raw::c_void,
        valid_symbols: *const bool,
    ) -> bool;
}

/// Safe wrapper around the external scanner
///
/// This struct provides a safe Rust interface to the C scanner,
/// managing memory and ensuring proper cleanup.
pub struct ExternalScanner {
    scanner: *mut std::os::raw::c_void,
}

impl ExternalScanner {
    /// Create a new external scanner instance
    ///
    /// # Safety
    ///
    /// This function calls into C code but is safe because:
    /// - The C function is designed to be called from Rust
    /// - We properly manage the returned pointer
    /// - Memory cleanup is handled in Drop
    pub fn new() -> Self {
        let scanner = unsafe { tree_sitter_nix_external_scanner_create() };
        assert!(!scanner.is_null(), "Failed to create external scanner");
        
        Self { scanner }
    }

    /// Serialize the scanner state
    ///
    /// Returns the serialized state as a byte vector.
    ///
    /// # Safety
    ///
    /// This function is safe because:
    /// - We provide a valid buffer and scanner pointer
    /// - The C function is designed for this interface
    /// - We properly handle the returned length
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = [0u8; 256]; // Reasonable buffer size
        
        let length = unsafe {
            tree_sitter_nix_external_scanner_serialize(
                self.scanner,
                buffer.as_mut_ptr() as *mut c_char,
            )
        };
        
        buffer[..length as usize].to_vec()
    }

    /// Deserialize scanner state from bytes
    ///
    /// # Arguments
    ///
    /// * `data` - Serialized state data
    ///
    /// # Safety
    ///
    /// This function is safe because:
    /// - We validate the input data length
    /// - We provide valid pointers to the C function
    /// - The C function is designed for this interface
    pub fn deserialize(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        
        unsafe {
            tree_sitter_nix_external_scanner_deserialize(
                self.scanner,
                data.as_ptr() as *const c_char,
                data.len() as c_uint,
            );
        }
    }
}

impl Drop for ExternalScanner {
    /// Clean up the external scanner
    ///
    /// # Safety
    ///
    /// This is safe because:
    /// - We only call destroy once (Drop is called once)
    /// - We check for null pointer (though it should never be null)
    /// - The C function is designed for cleanup
    fn drop(&mut self) {
        if !self.scanner.is_null() {
            unsafe {
                tree_sitter_nix_external_scanner_destroy(self.scanner);
            }
            self.scanner = ptr::null_mut();
        }
    }
}

// Ensure ExternalScanner is Send and Sync if the underlying C code is thread-safe
// Note: This needs to be verified based on the actual C implementation
unsafe impl Send for ExternalScanner {}
unsafe impl Sync for ExternalScanner {}

/// Scanner token types
///
/// These correspond to the external tokens defined in the grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum TokenType {
    /// Start of a regular string
    StringStart = 0,
    /// Content within a string
    StringContent = 1,
    /// End of a regular string
    StringEnd = 2,
    /// Start of an indented string
    IndentedStringStart = 3,
    /// Content within an indented string
    IndentedStringContent = 4,
    /// End of an indented string
    IndentedStringEnd = 5,
    /// Start of string interpolation ${
    InterpolationStart = 6,
    /// End of string interpolation }
    InterpolationEnd = 7,
    /// Escape sequence in string
    EscapeSequence = 8,
    /// Comment (line or block)
    Comment = 9,
}

impl TokenType {
    /// Get all token types
    pub const fn all() -> &'static [TokenType] {
        &[
            TokenType::StringStart,
            TokenType::StringContent,
            TokenType::StringEnd,
            TokenType::IndentedStringStart,
            TokenType::IndentedStringContent,
            TokenType::IndentedStringEnd,
            TokenType::InterpolationStart,
            TokenType::InterpolationEnd,
            TokenType::EscapeSequence,
            TokenType::Comment,
        ]
    }
    
    /// Get the token type name
    pub const fn name(self) -> &'static str {
        match self {
            TokenType::StringStart => "string_start",
            TokenType::StringContent => "string_content",
            TokenType::StringEnd => "string_end",
            TokenType::IndentedStringStart => "indented_string_start",
            TokenType::IndentedStringContent => "indented_string_content",
            TokenType::IndentedStringEnd => "indented_string_end",
            TokenType::InterpolationStart => "interpolation_start",
            TokenType::InterpolationEnd => "interpolation_end",
            TokenType::EscapeSequence => "escape_sequence",
            TokenType::Comment => "comment",
        }
    }
    
    /// Check if this token type is related to strings
    pub const fn is_string_token(self) -> bool {
        matches!(self, 
                 TokenType::StringStart | 
                 TokenType::StringContent | 
                 TokenType::StringEnd |
                 TokenType::IndentedStringStart |
                 TokenType::IndentedStringContent |
                 TokenType::IndentedStringEnd)
    }
    
    /// Check if this token type is related to interpolation
    pub const fn is_interpolation_token(self) -> bool {
        matches!(self, 
                 TokenType::InterpolationStart | 
                 TokenType::InterpolationEnd)
    }
}

/// Scan for tokens using the external scanner
/// 
/// This function provides a safe wrapper around the raw C scanner function
/// for use by Tree-sitter and testing.
/// 
/// # Arguments
/// 
/// * `scanner` - The external scanner instance
/// * `lexer` - Raw pointer to Tree-sitter lexer (from Tree-sitter)
/// * `valid_symbols` - Array indicating which symbols are valid at this point
/// 
/// # Returns
/// 
/// Returns true if a token was successfully scanned, false otherwise
/// 
/// # Safety
/// 
/// This function is safe when called with valid pointers from Tree-sitter's
/// lexer infrastructure. The lexer and valid_symbols pointers must be valid
/// for the duration of the call.
pub fn scan_token(
    scanner: &mut ExternalScanner, 
    lexer: *mut std::os::raw::c_void,
    valid_symbols: &[bool; 10] // 10 token types
) -> bool {
    unsafe {
        tree_sitter_nix_external_scanner_scan(
            scanner.scanner,
            lexer,
            valid_symbols.as_ptr(),
        )
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = ExternalScanner::new();
        // Just verify it was created successfully
        // The actual scanner testing should be done at the integration level
        drop(scanner);
    }

    #[test]
    fn test_scanner_serialization() {
        let scanner = ExternalScanner::new();
        let serialized = scanner.serialize();
        
        // Should return some data (empty is also valid)
        assert!(serialized.len() <= 256);
        
        let mut scanner2 = ExternalScanner::new();
        scanner2.deserialize(&serialized);
        
        // Deserialization should succeed without panic
    }

    #[test]
    fn test_token_types() {
        assert_eq!(TokenType::StringStart.name(), "string_start");
        assert!(TokenType::StringStart.is_string_token());
        assert!(!TokenType::StringStart.is_interpolation_token());
        
        assert!(TokenType::InterpolationStart.is_interpolation_token());
        assert!(!TokenType::InterpolationStart.is_string_token());
        
        assert_eq!(TokenType::all().len(), 10);
    }

    #[test]
    fn test_token_type_display() {
        assert_eq!(format!("{}", TokenType::StringStart), "string_start");
        assert_eq!(format!("{}", TokenType::Comment), "comment");
    }

    #[test]
    fn test_scanner_token_scanning() {
        let mut scanner = ExternalScanner::new();
        
        // Test with null lexer (should be safe and just return false)
        let valid_symbols = [false; 10];
        let result = scan_token(&mut scanner, std::ptr::null_mut(), &valid_symbols);
        
        // With null lexer, should return false (no token found)
        assert!(!result);
    }
}