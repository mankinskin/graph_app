//! Structured representation of import paths to replace error-prone string manipulation

use anyhow::{
    anyhow,
    Result,
};
use std::fmt;

/// A structured representation of an import path like "crate_name::module::submodule::Item"
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImportPath {
    /// The root crate name (e.g., "my_crate", "crate")
    pub crate_name: String,
    /// Path segments between crate and final item (e.g., ["module", "submodule"])
    pub segments: Vec<String>,
    /// The final imported item (e.g., "Item")
    pub final_item: String,
}

impl ImportPath {
    /// Parse a string path like "crate_name::module::Item" into structured form
    pub fn parse(path_str: &str) -> Result<Self> {
        let parts: Vec<&str> = path_str.split("::").collect();

        if parts.len() < 2 {
            return Err(anyhow!(
                "Import path must have at least crate::item format: {}",
                path_str
            ));
        }

        let crate_name = parts[0].to_string();
        let final_item = parts.last().unwrap().to_string();
        let segments = if parts.len() > 2 {
            parts[1..parts.len() - 1]
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        };

        Ok(ImportPath {
            crate_name,
            segments,
            final_item,
        })
    }

    /// Create from components
    pub fn new(
        crate_name: String,
        segments: Vec<String>,
        final_item: String,
    ) -> Self {
        Self {
            crate_name,
            segments,
            final_item,
        }
    }

    /// Normalize crate name by converting hyphens to underscores
    pub fn normalize_crate_name(&mut self) {
        self.crate_name = self.crate_name.replace('-', "_");
    }

    /// Get the path relative to the crate root (without crate name)
    pub fn relative_to_crate(&self) -> String {
        if self.segments.is_empty() {
            self.final_item.clone()
        } else {
            format!("{}::{}", self.segments.join("::"), self.final_item)
        }
    }

    /// Check if this represents a glob import (final_item is "*")
    pub fn is_glob(&self) -> bool {
        self.final_item == "*"
    }

    /// Get the full path as a string
    pub fn full_path(&self) -> String {
        if self.segments.is_empty() {
            format!("{}::{}", self.crate_name, self.final_item)
        } else {
            format!(
                "{}::{}::{}",
                self.crate_name,
                self.segments.join("::"),
                self.final_item
            )
        }
    }

    /// Get just the module path (without final item)
    pub fn module_path(&self) -> String {
        if self.segments.is_empty() {
            self.crate_name.clone()
        } else {
            format!("{}::{}", self.crate_name, self.segments.join("::"))
        }
    }

    /// Check if this path starts with the given crate name
    pub fn starts_with_crate(
        &self,
        crate_name: &str,
    ) -> bool {
        self.crate_name == crate_name
            || self.crate_name == crate_name.replace('-', "_")
    }

    /// Strip the crate prefix and return relative path, or None if not from this crate
    pub fn strip_crate_prefix(
        &self,
        crate_name: &str,
    ) -> Option<String> {
        if self.starts_with_crate(crate_name) {
            Some(self.relative_to_crate())
        } else {
            None
        }
    }

    /// Check if this is a direct import (no intermediate segments)
    pub fn is_direct_import(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the depth of nesting (number of segments + 1 for final item)
    pub fn depth(&self) -> usize {
        self.segments.len() + 1
    }
}

impl fmt::Display for ImportPath {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.full_path())
    }
}

impl From<&str> for ImportPath {
    fn from(path_str: &str) -> Self {
        ImportPath::parse(path_str).unwrap_or_else(|_| {
            // Fallback for malformed paths
            ImportPath {
                crate_name: "unknown".to_string(),
                segments: Vec::new(),
                final_item: path_str.to_string(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_path() {
        let path = ImportPath::parse("crate::Item").unwrap();
        assert_eq!(path.crate_name, "crate");
        assert_eq!(path.segments, Vec::<String>::new());
        assert_eq!(path.final_item, "Item");
        assert!(path.is_direct_import());
    }

    #[test]
    fn test_parse_nested_path() {
        let path =
            ImportPath::parse("my_crate::module::submodule::Item").unwrap();
        assert_eq!(path.crate_name, "my_crate");
        assert_eq!(path.segments, vec!["module", "submodule"]);
        assert_eq!(path.final_item, "Item");
        assert!(!path.is_direct_import());
        assert_eq!(path.depth(), 3);
    }

    #[test]
    fn test_relative_path() {
        let path = ImportPath::parse("my_crate::module::Item").unwrap();
        assert_eq!(path.relative_to_crate(), "module::Item");
    }

    #[test]
    fn test_glob_import() {
        let path = ImportPath::parse("crate::module::*").unwrap();
        assert!(path.is_glob());
    }

    #[test]
    fn test_normalize_crate_name() {
        let mut path = ImportPath::parse("my-crate::Item").unwrap();
        path.normalize_crate_name();
        assert_eq!(path.crate_name, "my_crate");
    }

    #[test]
    fn test_starts_with_crate() {
        let path = ImportPath::parse("my_crate::Item").unwrap();
        assert!(path.starts_with_crate("my_crate"));
        assert!(path.starts_with_crate("my-crate")); // Should handle normalization
        assert!(!path.starts_with_crate("other_crate"));
    }

    #[test]
    fn test_strip_crate_prefix() {
        let path = ImportPath::parse("my_crate::module::Item").unwrap();
        assert_eq!(
            path.strip_crate_prefix("my_crate"),
            Some("module::Item".to_string())
        );
        assert_eq!(
            path.strip_crate_prefix("my-crate"),
            Some("module::Item".to_string())
        );
        assert_eq!(path.strip_crate_prefix("other_crate"), None);
    }
}
