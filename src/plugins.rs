use crate::error::Result;
use tree_sitter::Tree;

/// Plugin trait for extending parser functionality
pub trait Plugin: Send + Sync {
    /// Name of the plugin
    fn name(&self) -> &str;
    
    /// Pre-process source code before parsing
    fn pre_process(&mut self, source: String) -> Result<String> {
        Ok(source)
    }
    
    /// Post-process the parsed tree
    fn post_process(&mut self, tree: Tree) -> Result<Tree> {
        Ok(tree)
    }
    
    /// Validate the parsed tree
    fn validate(&self, _tree: &Tree) -> Result<()> {
        Ok(())
    }
}

/// Example plugin that adds logging
pub struct LoggingPlugin {
    name: String,
}

impl LoggingPlugin {
    pub fn new() -> Self {
        Self {
            name: "logging".to_string(),
        }
    }
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn pre_process(&mut self, source: String) -> Result<String> {
        tracing::debug!("Pre-processing {} bytes of source", source.len());
        Ok(source)
    }
    
    fn post_process(&mut self, tree: Tree) -> Result<Tree> {
        tracing::debug!("Post-processing tree with {} nodes", tree.root_node().child_count());
        Ok(tree)
    }
}

/// Plugin that normalizes whitespace
pub struct WhitespaceNormalizer;

impl Plugin for WhitespaceNormalizer {
    fn name(&self) -> &str {
        "whitespace_normalizer"
    }
    
    fn pre_process(&mut self, source: String) -> Result<String> {
        // Normalize line endings
        let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logging_plugin() {
        let plugin = LoggingPlugin::new();
        assert_eq!(plugin.name(), "logging");
    }
    
    #[test]
    fn test_whitespace_normalizer() {
        let mut plugin = WhitespaceNormalizer;
        let source = "hello\r\nworld\rtest".to_string();
        let result = plugin.pre_process(source).unwrap();
        assert_eq!(result, "hello\nworld\ntest");
    }
}