//! Incremental parsing support for the Nix parser

use std::collections::HashMap;
use tree_sitter::{Tree, InputEdit, Point};

use crate::parser::{NixParser, ParseResult};
use crate::error::{ParseError, Result};

/// Incremental parser that tracks document changes
///
/// This parser maintains state across multiple parse operations,
/// allowing for efficient re-parsing when only small portions
/// of a document change.
pub struct IncrementalParser {
    parser: NixParser,
    document_trees: HashMap<String, DocumentState>,
}

impl IncrementalParser {
    /// Create a new incremental parser
    pub fn new() -> Result<Self> {
        Ok(Self {
            parser: NixParser::new()?,
            document_trees: HashMap::new(),
        })
    }

    /// Parse a document for the first time
    ///
    /// # Arguments
    ///
    /// * `document_id` - Unique identifier for the document
    /// * `source` - The source code to parse
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing the parsed tree and diagnostics.
    pub fn parse_document(&mut self, document_id: impl Into<String>, source: &str) -> Result<ParseResult> {
        let doc_id = document_id.into();
        let result = self.parser.parse(source)?;
        
        self.document_trees.insert(doc_id, DocumentState {
            tree: result.tree().clone(),
            source: source.to_string(),
            version: 1,
        });
        
        Ok(result)
    }

    /// Update a document with changes and re-parse incrementally
    ///
    /// # Arguments
    ///
    /// * `document_id` - Unique identifier for the document
    /// * `changes` - List of changes to apply
    /// * `new_source` - The updated source code
    ///
    /// # Returns
    ///
    /// A `ParseResult` with the incrementally updated tree.
    pub fn update_document(
        &mut self, 
        document_id: &str, 
        changes: &[TextChange], 
        new_source: &str
    ) -> Result<ParseResult> {
        let doc_state = self.document_trees.get_mut(document_id)
            .ok_or_else(|| ParseError::ValidationError(
                format!("Document '{}' not found. Call parse_document first.", document_id)
            ))?;

        // Apply edits to the existing tree
        let mut tree = doc_state.tree.clone();
        for change in changes {
            let edit = change.to_input_edit(&doc_state.source, new_source);
            tree.edit(&edit);
        }

        // Re-parse with the old tree for incremental parsing
        let result = self.parser.parse_with_context(new_source, Some(&tree))?;

        // Update the stored state
        doc_state.tree = result.tree().clone();
        doc_state.source = new_source.to_string();
        doc_state.version += 1;

        Ok(result)
    }

    /// Remove a document from tracking
    pub fn remove_document(&mut self, document_id: &str) -> bool {
        self.document_trees.remove(document_id).is_some()
    }

    /// Get the current version of a document
    pub fn document_version(&self, document_id: &str) -> Option<u32> {
        self.document_trees.get(document_id).map(|state| state.version)
    }

    /// Check if a document is being tracked
    pub fn has_document(&self, document_id: &str) -> bool {
        self.document_trees.contains_key(document_id)
    }

    /// Get statistics about tracked documents
    pub fn stats(&self) -> IncrementalStats {
        IncrementalStats {
            document_count: self.document_trees.len(),
            total_source_size: self.document_trees.values()
                .map(|state| state.source.len())
                .sum(),
        }
    }

    /// Clear all tracked documents
    pub fn clear(&mut self) {
        self.document_trees.clear();
    }
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default IncrementalParser")
    }
}

/// State maintained for each document
#[derive(Debug, Clone)]
struct DocumentState {
    tree: Tree,
    source: String,
    version: u32,
}

/// Represents a text change in a document
///
/// This is typically created from LSP text document change events
/// or editor change notifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChange {
    /// Start position of the change (0-based)
    pub start: Position,
    
    /// End position of the change (0-based, exclusive)
    pub end: Position,
    
    /// New text to insert (empty string for deletions)
    pub new_text: String,
}

impl TextChange {
    /// Create a new text change
    pub fn new(start: Position, end: Position, new_text: impl Into<String>) -> Self {
        Self {
            start,
            end,
            new_text: new_text.into(),
        }
    }
    
    /// Create an insertion at a specific position
    pub fn insert(position: Position, text: impl Into<String>) -> Self {
        Self::new(position, position, text)
    }
    
    /// Create a deletion of a range
    pub fn delete(start: Position, end: Position) -> Self {
        Self::new(start, end, "")
    }
    
    /// Create a replacement of a range
    pub fn replace(start: Position, end: Position, text: impl Into<String>) -> Self {
        Self::new(start, end, text)
    }

    /// Convert to Tree-sitter's InputEdit format
    fn to_input_edit(&self, old_source: &str, new_source: &str) -> InputEdit {
        let old_start_byte = position_to_byte_offset(old_source, self.start);
        let old_end_byte = position_to_byte_offset(old_source, self.end);
        let new_end_byte = old_start_byte + self.new_text.len();

        InputEdit {
            start_byte: old_start_byte,
            old_end_byte,
            new_end_byte,
            start_position: Point::new(self.start.line, self.start.character),
            old_end_position: Point::new(self.end.line, self.end.character),
            new_end_position: byte_offset_to_position(new_source, new_end_byte),
        }
    }
}

/// Position in a text document (0-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// Line number (0-based)
    pub line: usize,
    
    /// Character offset within the line (0-based, UTF-16 code units)
    pub character: usize,
}

impl Position {
    /// Create a new position
    pub const fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
    
    /// Position at the start of the document
    pub const fn zero() -> Self {
        Self::new(0, 0)
    }
}

/// Statistics about incremental parsing state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalStats {
    /// Number of documents being tracked
    pub document_count: usize,
    
    /// Total size of all tracked source code
    pub total_source_size: usize,
}

// Helper functions for position and byte offset conversion

fn position_to_byte_offset(source: &str, position: Position) -> usize {
    let mut current_line = 0;
    let mut current_char = 0;
    
    for (byte_offset, ch) in source.char_indices() {
        if current_line == position.line && current_char == position.character {
            return byte_offset;
        }
        
        if ch == '\n' {
            current_line += 1;
            current_char = 0;
        } else {
            current_char += ch.len_utf16();
        }
    }
    
    source.len()
}

fn byte_offset_to_position(source: &str, byte_offset: usize) -> Point {
    let mut line = 0;
    let mut column = 0;
    
    for (offset, ch) in source.char_indices() {
        if offset >= byte_offset {
            break;
        }
        
        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += ch.len_utf8();
        }
    }
    
    Point::new(line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_parser_creation() {
        let parser = IncrementalParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_document_tracking() {
        let mut parser = IncrementalParser::new().unwrap();
        
        // Parse initial document
        let result = parser.parse_document("test.nix", "{ x = 1; }");
        assert!(result.is_ok());
        assert!(parser.has_document("test.nix"));
        assert_eq!(parser.document_version("test.nix"), Some(1));
        
        // Update document
        let changes = vec![
            TextChange::replace(
                Position::new(0, 6), 
                Position::new(0, 7), 
                "2"
            )
        ];
        let result = parser.update_document("test.nix", &changes, "{ x = 2; }");
        assert!(result.is_ok());
        assert_eq!(parser.document_version("test.nix"), Some(2));
        
        // Remove document
        assert!(parser.remove_document("test.nix"));
        assert!(!parser.has_document("test.nix"));
    }

    #[test]
    fn test_text_changes() {
        let insert = TextChange::insert(Position::new(0, 5), "hello");
        assert_eq!(insert.start, insert.end);
        assert_eq!(insert.new_text, "hello");
        
        let delete = TextChange::delete(Position::new(0, 0), Position::new(0, 5));
        assert_eq!(delete.new_text, "");
        
        let replace = TextChange::replace(Position::new(0, 0), Position::new(0, 5), "world");
        assert_eq!(replace.new_text, "world");
    }

    #[test]
    fn test_position_conversion() {
        let source = "line1\nline2\nline3";
        
        // Test position to byte offset
        let pos = Position::new(1, 0); // Start of second line
        let byte_offset = position_to_byte_offset(source, pos);
        assert_eq!(byte_offset, 6); // After "line1\n"
        
        // Test byte offset to position
        let point = byte_offset_to_position(source, 6);
        assert_eq!(point.row, 1);
        assert_eq!(point.column, 0);
    }

    #[test]
    fn test_incremental_stats() {
        let mut parser = IncrementalParser::new().unwrap();
        
        let initial_stats = parser.stats();
        assert_eq!(initial_stats.document_count, 0);
        assert_eq!(initial_stats.total_source_size, 0);
        
        parser.parse_document("test1.nix", "{ x = 1; }").unwrap();
        parser.parse_document("test2.nix", "{ y = 2; }").unwrap();
        
        let stats = parser.stats();
        assert_eq!(stats.document_count, 2);
        assert_eq!(stats.total_source_size, 18); // 9 + 9 characters
    }
}