#[cfg(feature = "cache")]
use lru::LruCache;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::{Arc, Mutex};

/// Cache for parsed expressions
#[cfg(feature = "cache")]
pub struct ParserCache {
    cache: Arc<Mutex<LruCache<u64, CachedResult>>>,
}

#[cfg(feature = "cache")]
impl ParserCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap()
            ))),
        }
    }
    
    pub fn get(&self, source: &str) -> Option<CachedResult> {
        let key = hash_source(source);
        let mut cache = self.cache.lock().unwrap();
        cache.get(&key).cloned()
    }
    
    pub fn insert(&self, source: &str, result: CachedResult) {
        let key = hash_source(source);
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, result);
    }
    
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

#[cfg(not(feature = "cache"))]
pub struct ParserCache;

#[cfg(not(feature = "cache"))]
impl ParserCache {
    pub fn new(_capacity: usize) -> Self {
        Self
    }
    
    pub fn get(&self, _source: &str) -> Option<CachedResult> {
        None
    }
    
    pub fn insert(&self, _source: &str, _result: CachedResult) {}
    
    pub fn clear(&self) {}
}

/// Cached parsing result
#[derive(Debug, Clone)]
pub struct CachedResult {
    pub tree_bytes: Vec<u8>,
    pub diagnostics: Vec<String>,
}

fn hash_source(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_operations() {
        let cache = ParserCache::new(10);
        
        let result = CachedResult {
            tree_bytes: vec![1, 2, 3],
            diagnostics: vec!["test".to_string()],
        };
        
        cache.insert("test", result.clone());
        
        let retrieved = cache.get("test");
        assert!(retrieved.is_some());
        
        #[cfg(feature = "cache")]
        assert_eq!(retrieved.unwrap().tree_bytes, vec![1, 2, 3]);
    }
}