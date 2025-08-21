//! Caching infrastructure for parse results

use std::sync::{Arc, Mutex};
use lru::LruCache;
use std::num::NonZeroUsize;

use crate::parser::ParseResult;

/// Cache for storing parse results
pub struct ParseCache {
    cache: Arc<Mutex<LruCache<String, ParseResult>>>,
}

impl ParseCache {
    /// Create a new cache with the specified capacity
    pub fn new(capacity: usize) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap()));
        Self {
            cache: Arc::new(Mutex::new(cache)),
        }
    }
    
    /// Get a cached parse result
    pub fn get(&self, key: &str) -> Option<ParseResult> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(key).cloned()
    }
    
    /// Insert a parse result into the cache
    pub fn insert(&self, key: String, value: ParseResult) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, value);
    }
    
    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new(100)
    }
}