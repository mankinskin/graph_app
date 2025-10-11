//! AST caching manager to eliminate redundant file parsing

use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
    time::SystemTime,
};
use syn::File;

/// Manager for cached AST parsing to eliminate redundant file reads and parsing
pub struct AstManager {
    /// Cache mapping file paths to (modification_time, parsed_ast)
    cache: HashMap<PathBuf, (SystemTime, File)>,
    /// Whether to enable verbose logging
    verbose: bool,
}

impl AstManager {
    /// Create a new AST manager
    pub fn new(verbose: bool) -> Self {
        Self {
            cache: HashMap::new(),
            verbose,
        }
    }

    /// Get a parsed AST, either from cache or by parsing the file
    pub fn get_or_parse(&mut self, path: &Path) -> Result<&File> {
        let canonical_path = path.canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

        // Check if we need to refresh the cache entry
        let should_parse = self.should_parse_file(&canonical_path)?;

        if should_parse {
            if self.verbose {
                println!("🔄 Parsing AST for {}", path.display());
            }
            self.parse_and_cache(&canonical_path)?;
        } else if self.verbose {
            println!("💾 Using cached AST for {}", path.display());
        }

        // Return the cached entry (we know it exists after parse_and_cache)
        Ok(&self.cache.get(&canonical_path).unwrap().1)
    }

    /// Invalidate cache entry for a specific file
    pub fn invalidate(&mut self, path: &Path) {
        if let Ok(canonical_path) = path.canonicalize() {
            self.cache.remove(&canonical_path);
            if self.verbose {
                println!("🗑️  Invalidated cache for {}", path.display());
            }
        }
    }

    /// Clear all cached entries
    pub fn clear_cache(&mut self) {
        let count = self.cache.len();
        self.cache.clear();
        if self.verbose && count > 0 {
            println!("🗑️  Cleared {} cached AST entries", count);
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.cache.len(),
            total_memory_estimate: self.estimate_memory_usage(),
        }
    }

    /// Force refresh of a file (re-parse even if cached)
    pub fn force_refresh(&mut self, path: &Path) -> Result<&File> {
        self.invalidate(path);
        self.get_or_parse(path)
    }

    // Private helper methods
    
    fn should_parse_file(&self, path: &PathBuf) -> Result<bool> {
        // Always parse if not in cache
        if !self.cache.contains_key(path) {
            return Ok(true);
        }

        // Check if file has been modified since cache entry
        let file_modified = fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for {}", path.display()))?
            .modified()
            .with_context(|| format!("Failed to get modification time for {}", path.display()))?;

        let (cached_time, _) = &self.cache[path];
        Ok(file_modified > *cached_time)
    }

    fn parse_and_cache(&mut self, path: &PathBuf) -> Result<()> {
        // Read and parse the file
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let syntax_tree = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse Rust file: {}", path.display()))?;

        // Get modification time
        let modified_time = fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for {}", path.display()))?
            .modified()
            .with_context(|| format!("Failed to get modification time for {}", path.display()))?;

        // Cache the parsed AST
        self.cache.insert(path.clone(), (modified_time, syntax_tree));

        Ok(())
    }

    fn estimate_memory_usage(&self) -> usize {
        // Rough estimate: each cached file is approximately 10KB + path size
        self.cache.iter().map(|(path, _)| {
            10_000 + path.to_string_lossy().len()
        }).sum()
    }
}

impl Default for AstManager {
    fn default() -> Self {
        Self::new(false)
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_memory_estimate: usize,
}

impl CacheStats {
    pub fn memory_mb(&self) -> f64 {
        self.total_memory_estimate as f64 / (1024.0 * 1024.0)
    }
}

/// Convenience functions for common AST operations with caching
impl AstManager {
    /// Parse a file and extract pub use statements using the cache
    pub fn get_pub_use_statements(&mut self, path: &Path) -> Result<Vec<syn::ItemUse>> {
        let ast = self.get_or_parse(path)?;
        
        let pub_uses = ast.items.iter()
            .filter_map(|item| {
                if let syn::Item::Use(use_item) = item {
                    if matches!(use_item.vis, syn::Visibility::Public(_)) {
                        Some(use_item.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(pub_uses)
    }

    /// Get all use statements (pub and private) from a file
    pub fn get_all_use_statements(&mut self, path: &Path) -> Result<Vec<syn::ItemUse>> {
        let ast = self.get_or_parse(path)?;
        
        let uses = ast.items.iter()
            .filter_map(|item| {
                if let syn::Item::Use(use_item) = item {
                    Some(use_item.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(uses)
    }

    /// Get all public items from a file
    pub fn get_public_items(&mut self, path: &Path) -> Result<Vec<String>> {
        use crate::syntax::item_info::ItemInfo;
        
        let ast = self.get_or_parse(path)?;
        
        let public_items = ast.items.iter()
            .filter_map(|item| {
                if item.is_public() {
                    item.get_identifier()
                } else {
                    None
                }
            })
            .collect();

        Ok(public_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_ast_manager_caching() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        
        // Write a test file
        fs::write(&file_path, "pub fn test() {}").unwrap();
        
        let mut manager = AstManager::new(false);
        
        // First parse should cache
        let _ast1 = manager.get_or_parse(&file_path).unwrap();
        assert_eq!(manager.cache_stats().total_entries, 1);
        
        // Second parse should use cache (we can't compare pointers easily, so just check cache hit)
        let _ast2 = manager.get_or_parse(&file_path).unwrap();
        assert_eq!(manager.cache_stats().total_entries, 1); // Still only one entry
    }

    #[test]
    fn test_cache_invalidation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        
        fs::write(&file_path, "pub fn test() {}").unwrap();
        
        let mut manager = AstManager::new(false);
        
        // Parse and cache
        manager.get_or_parse(&file_path).unwrap();
        assert_eq!(manager.cache_stats().total_entries, 1);
        
        // Invalidate
        manager.invalidate(&file_path);
        assert_eq!(manager.cache_stats().total_entries, 0);
    }

    #[test]
    fn test_pub_use_extraction() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        
        fs::write(&file_path, r#"
            pub use std::collections::HashMap;
            use std::fs;
            pub use crate::module::Item;
        "#).unwrap();
        
        let mut manager = AstManager::new(false);
        let pub_uses = manager.get_pub_use_statements(&file_path).unwrap();
        
        assert_eq!(pub_uses.len(), 2);
    }

    #[test]
    fn test_public_items_extraction() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        
        fs::write(&file_path, r#"
            pub fn public_fn() {}
            fn private_fn() {}
            pub struct PublicStruct;
            struct PrivateStruct;
        "#).unwrap();
        
        let mut manager = AstManager::new(false);
        let public_items = manager.get_public_items(&file_path).unwrap();
        
        assert_eq!(public_items.len(), 2);
        assert!(public_items.contains(&"public_fn".to_string()));
        assert!(public_items.contains(&"PublicStruct".to_string()));
    }
}