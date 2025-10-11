//! Unified UseTree navigation to eliminate duplication across multiple parsing functions

use syn::UseTree;
use std::collections::BTreeSet;
use crate::core::path::ImportPath;

/// Unified navigator for traversing syn::UseTree structures
/// Replaces the 5+ different use tree traversal functions in the codebase
pub struct UseTreeNavigator;

impl UseTreeNavigator {
    /// Extract items from a use tree using a pluggable collector
    pub fn extract_items<T>(&self, tree: &UseTree, collector: &mut T) 
    where 
        T: UseTreeItemCollector 
    {
        self.extract_items_recursive(tree, &[], collector);
    }

    /// Find specific patterns in a use tree
    pub fn find_patterns<P>(&self, tree: &UseTree, pattern: P) -> Vec<UseTreeMatch>
    where 
        P: UseTreePattern 
    {
        let mut matches = Vec::new();
        self.find_patterns_recursive(tree, &[], &pattern, &mut matches);
        matches
    }

    /// Extract items with full path context
    pub fn extract_with_paths(&self, tree: &UseTree) -> Vec<ImportPath> {
        let mut collector = PathCollector::new();
        self.extract_items(tree, &mut collector);
        collector.paths
    }

    /// Check if tree contains imports from a specific crate
    pub fn contains_crate_imports(&self, tree: &UseTree, crate_name: &str) -> bool {
        let normalized_crate = crate_name.replace('-', "_");
        self.contains_crate_recursive(tree, &[], &normalized_crate)
    }

    // Private recursive implementation
    fn extract_items_recursive<T>(&self, tree: &UseTree, path: &[String], collector: &mut T)
    where
        T: UseTreeItemCollector
    {
        match tree {
            UseTree::Path(use_path) => {
                let ident = use_path.ident.to_string();
                let mut new_path = path.to_vec();
                new_path.push(ident);
                self.extract_items_recursive(&use_path.tree, &new_path, collector);
            }
            UseTree::Name(name) => {
                let name_str = name.ident.to_string();
                collector.collect_name(&name_str, path);
            }
            UseTree::Rename(rename) => {
                let original = rename.ident.to_string();
                let renamed = rename.rename.to_string();
                collector.collect_rename(&original, &renamed, path);
            }
            UseTree::Glob(_) => {
                collector.collect_glob(path);
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    self.extract_items_recursive(item, path, collector);
                }
            }
        }
    }

    fn find_patterns_recursive<P>(
        &self, 
        tree: &UseTree, 
        path: &[String], 
        pattern: &P, 
        matches: &mut Vec<UseTreeMatch>
    )
    where
        P: UseTreePattern
    {
        match tree {
            UseTree::Path(use_path) => {
                let ident = use_path.ident.to_string();
                let mut new_path = path.to_vec();
                new_path.push(ident);
                
                if pattern.matches_path(&new_path) {
                    matches.push(UseTreeMatch::Path(new_path.clone()));
                }
                
                self.find_patterns_recursive(&use_path.tree, &new_path, pattern, matches);
            }
            UseTree::Name(name) => {
                let name_str = name.ident.to_string();
                if pattern.matches_name(&name_str, path) {
                    matches.push(UseTreeMatch::Name { 
                        name: name_str, 
                        path: path.to_vec() 
                    });
                }
            }
            UseTree::Rename(rename) => {
                let original = rename.ident.to_string();
                let renamed = rename.rename.to_string();
                if pattern.matches_rename(&original, &renamed, path) {
                    matches.push(UseTreeMatch::Rename { 
                        original, 
                        renamed, 
                        path: path.to_vec() 
                    });
                }
            }
            UseTree::Glob(_) => {
                if pattern.matches_glob(path) {
                    matches.push(UseTreeMatch::Glob(path.to_vec()));
                }
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    self.find_patterns_recursive(item, path, pattern, matches);
                }
            }
        }
    }

    fn contains_crate_recursive(&self, tree: &UseTree, path: &[String], crate_name: &str) -> bool {
        match tree {
            UseTree::Path(use_path) => {
                let ident = use_path.ident.to_string();
                
                // Check if we found the crate at the root
                if path.is_empty() && ident == crate_name {
                    return true;
                }
                
                let mut new_path = path.to_vec();
                new_path.push(ident);
                self.contains_crate_recursive(&use_path.tree, &new_path, crate_name)
            }
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {
                !path.is_empty() && path[0] == crate_name
            }
            UseTree::Group(group) => {
                group.items.iter().any(|item| {
                    self.contains_crate_recursive(item, path, crate_name)
                })
            }
        }
    }
}

/// Trait for collecting items from use tree traversal
pub trait UseTreeItemCollector {
    /// Collect a simple name
    fn collect_name(&mut self, name: &str, path: &[String]);
    
    /// Collect a glob import
    fn collect_glob(&mut self, path: &[String]);
    
    /// Collect a renamed import
    fn collect_rename(&mut self, original: &str, renamed: &str, path: &[String]);
}

/// Trait for matching patterns in use trees
pub trait UseTreePattern {
    fn matches_path(&self, path: &[String]) -> bool;
    fn matches_name(&self, name: &str, path: &[String]) -> bool;
    fn matches_rename(&self, original: &str, renamed: &str, path: &[String]) -> bool;
    fn matches_glob(&self, path: &[String]) -> bool;
}

/// Result of pattern matching in use trees
#[derive(Debug, Clone)]
pub enum UseTreeMatch {
    Path(Vec<String>),
    Name { name: String, path: Vec<String> },
    Rename { original: String, renamed: String, path: Vec<String> },
    Glob(Vec<String>),
}

/// Simple collector that gathers item names
pub struct ItemNameCollector {
    pub items: BTreeSet<String>,
}

impl ItemNameCollector {
    pub fn new() -> Self {
        Self {
            items: BTreeSet::new(),
        }
    }
}

impl UseTreeItemCollector for ItemNameCollector {
    fn collect_name(&mut self, name: &str, _path: &[String]) {
        self.items.insert(name.to_string());
    }

    fn collect_glob(&mut self, _path: &[String]) {
        self.items.insert("*".to_string());
    }

    fn collect_rename(&mut self, _original: &str, renamed: &str, _path: &[String]) {
        self.items.insert(renamed.to_string());
    }
}

/// Collector that creates full ImportPath objects
pub struct PathCollector {
    pub paths: Vec<ImportPath>,
}

impl PathCollector {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
        }
    }
}

impl UseTreeItemCollector for PathCollector {
    fn collect_name(&mut self, name: &str, path: &[String]) {
        if path.is_empty() {
            return; // Skip incomplete paths
        }
        
        let crate_name = path[0].clone();
        let segments = if path.len() > 1 {
            path[1..].to_vec()
        } else {
            Vec::new()
        };
        
        self.paths.push(ImportPath::new(
            crate_name,
            segments,
            name.to_string(),
        ));
    }

    fn collect_glob(&mut self, path: &[String]) {
        if path.is_empty() {
            return;
        }
        
        let crate_name = path[0].clone();
        let segments = if path.len() > 1 {
            path[1..].to_vec()
        } else {
            Vec::new()
        };
        
        self.paths.push(ImportPath::new(
            crate_name,
            segments,
            "*".to_string(),
        ));
    }

    fn collect_rename(&mut self, original: &str, _renamed: &str, path: &[String]) {
        if path.is_empty() {
            return;
        }
        
        let crate_name = path[0].clone();
        let segments = if path.len() > 1 {
            path[1..].to_vec()
        } else {
            Vec::new()
        };
        
        // Store the original name since that's what we'll export
        self.paths.push(ImportPath::new(
            crate_name,
            segments,
            original.to_string(),
        ));
    }
}

/// Pattern matcher for specific crate imports
pub struct CrateImportPattern {
    pub target_crate: String,
}

impl CrateImportPattern {
    pub fn new(crate_name: &str) -> Self {
        Self {
            target_crate: crate_name.replace('-', "_"),
        }
    }
}

impl UseTreePattern for CrateImportPattern {
    fn matches_path(&self, path: &[String]) -> bool {
        !path.is_empty() && path[0] == self.target_crate
    }

    fn matches_name(&self, _name: &str, path: &[String]) -> bool {
        !path.is_empty() && path[0] == self.target_crate
    }

    fn matches_rename(&self, _original: &str, _renamed: &str, path: &[String]) -> bool {
        !path.is_empty() && path[0] == self.target_crate
    }

    fn matches_glob(&self, path: &[String]) -> bool {
        !path.is_empty() && path[0] == self.target_crate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_simple_names() {
        let tree: UseTree = parse_quote! { crate::{add, subtract} };
        let navigator = UseTreeNavigator;
        let mut collector = ItemNameCollector::new();
        
        navigator.extract_items(&tree, &mut collector);
        
        assert_eq!(collector.items.len(), 2);
        assert!(collector.items.contains("add"));
        assert!(collector.items.contains("subtract"));
    }

    #[test]
    fn test_extract_nested_paths() {
        let tree: UseTree = parse_quote! { my_crate::math::{add, multiply} };
        let navigator = UseTreeNavigator;
        let paths = navigator.extract_with_paths(&tree);
        
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().any(|p| p.final_item == "add" && p.segments == vec!["math"]));
        assert!(paths.iter().any(|p| p.final_item == "multiply" && p.segments == vec!["math"]));
    }

    #[test]
    fn test_contains_crate_imports() {
        let tree: UseTree = parse_quote! { my_crate::math::add };
        let navigator = UseTreeNavigator;
        
        assert!(navigator.contains_crate_imports(&tree, "my_crate"));
        assert!(navigator.contains_crate_imports(&tree, "my-crate")); // normalized
        assert!(!navigator.contains_crate_imports(&tree, "other_crate"));
    }

    #[test]
    fn test_crate_import_pattern() {
        let tree: UseTree = parse_quote! { my_crate::{math::add, utils::format} };
        let navigator = UseTreeNavigator;
        let pattern = CrateImportPattern::new("my_crate");
        
        let matches = navigator.find_patterns(&tree, pattern);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_glob_handling() {
        let tree: UseTree = parse_quote! { crate::* };
        let navigator = UseTreeNavigator;
        let mut collector = ItemNameCollector::new();
        
        navigator.extract_items(&tree, &mut collector);
        
        assert!(collector.items.contains("*"));
    }

    #[test]
    fn test_rename_handling() {
        let tree: UseTree = parse_quote! { crate::add as plus };
        let navigator = UseTreeNavigator;
        let mut collector = ItemNameCollector::new();
        
        navigator.extract_items(&tree, &mut collector);
        
        assert!(collector.items.contains("plus"));
    }
}