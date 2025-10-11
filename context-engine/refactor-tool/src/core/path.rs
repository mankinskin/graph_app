//! Structured representation of import paths to replace error-prone string manipulation

use crate::common::format::format_relative_path;
use anyhow::{
    anyhow,
    Result,
};
use std::{
    fmt,
    path::{
        Path,
        PathBuf,
    },
};

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

    /// Convert a super:: import to a crate:: import by resolving the relative path
    pub fn normalize_super_import(
        &mut self,
        file_path: &Path,
        crate_root: &Path,
    ) -> Result<()> {
        if self.crate_name != "super" {
            return Ok(()); // Not a super import, nothing to do
        }

        let absolute_path = resolve_super_to_crate_path(
            file_path,
            crate_root,
            &self.segments,
            &self.final_item,
        )?;

        // Update this ImportPath to use crate:: instead of super::
        self.crate_name = "crate".to_string();
        // The segments and final_item are already correct from the resolution
        self.segments = absolute_path.segments;
        self.final_item = absolute_path.final_item;

        Ok(())
    }
}

/// Resolve a super:: import to its equivalent crate:: form
///
/// This function takes a file path and super:: import segments and converts them
/// to their absolute path within the crate root.
pub fn resolve_super_to_crate_path(
    file_path: &Path,
    crate_root: &Path,
    super_segments: &[String],
    final_item: &str,
) -> Result<ImportPath> {
    // Get the directory containing the current file
    let current_dir = file_path.parent().ok_or_else(|| {
        anyhow!("Cannot get parent directory of {}", file_path.display())
    })?;

    // Get the path relative to the crate root
    let relative_to_crate = current_dir
        .strip_prefix(crate_root.join("src"))
        .map_err(|_| {
            anyhow!(
                "File {} is not within crate src directory {}",
                file_path.display(),
                crate_root.join("src").display()
            )
        })?;

    // Convert path to module segments
    let mut current_segments: Vec<String> =
        if relative_to_crate.as_os_str().is_empty() {
            // File is in src/ root
            Vec::new()
        } else {
            relative_to_crate
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect()
        };

    // Move up one level for each super:: (the first super takes us to parent)
    // Note: super:: means "parent module", so we need to go up one level from current module
    if !current_segments.is_empty() {
        current_segments.pop(); // Go to parent module
    } else {
        return Err(anyhow!("Cannot use super:: from crate root"));
    }

    // Apply any additional segments from the super import
    current_segments.extend(super_segments.iter().cloned());

    Ok(ImportPath::new(
        "crate".to_string(),
        current_segments,
        final_item.to_string(),
    ))
}

/// Check if a path represents a super:: import
pub fn is_super_import(path: &str) -> bool {
    path.starts_with("super::")
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

    #[test]
    fn test_is_super_import() {
        assert!(is_super_import("super::module::Item"));
        assert!(is_super_import("super::Item"));
        assert!(!is_super_import("crate::Item"));
        assert!(!is_super_import("other_crate::Item"));
    }

    #[test]
    fn test_resolve_super_to_crate_path() {
        use std::path::PathBuf;

        // Test file in src/submodule/file.rs importing super::Item
        let crate_root = PathBuf::from("/workspace/my_crate");
        let file_path =
            PathBuf::from("/workspace/my_crate/src/submodule/file.rs");

        let result =
            resolve_super_to_crate_path(&file_path, &crate_root, &[], "Item")
                .unwrap();

        assert_eq!(result.crate_name, "crate");
        assert_eq!(result.segments, Vec::<String>::new()); // Goes to crate root
        assert_eq!(result.final_item, "Item");
        assert_eq!(result.full_path(), "crate::Item");
    }

    #[test]
    fn test_resolve_super_with_segments() {
        use std::path::PathBuf;

        // Test file in src/deep/nested/file.rs importing super::utils::helper
        let crate_root = PathBuf::from("/workspace/my_crate");
        let file_path =
            PathBuf::from("/workspace/my_crate/src/deep/nested/file.rs");

        let result = resolve_super_to_crate_path(
            &file_path,
            &crate_root,
            &["utils".to_string()],
            "helper",
        )
        .unwrap();

        assert_eq!(result.crate_name, "crate");
        assert_eq!(result.segments, vec!["deep", "utils"]);
        assert_eq!(result.final_item, "helper");
        assert_eq!(result.full_path(), "crate::deep::utils::helper");
    }

    #[test]
    fn test_normalize_super_import() {
        use std::path::PathBuf;

        let crate_root = PathBuf::from("/workspace/my_crate");
        let file_path =
            PathBuf::from("/workspace/my_crate/src/submodule/file.rs");

        let mut path = ImportPath::parse("super::Item").unwrap();
        path.normalize_super_import(&file_path, &crate_root)
            .unwrap();

        assert_eq!(path.crate_name, "crate");
        assert_eq!(path.segments, Vec::<String>::new());
        assert_eq!(path.final_item, "Item");
        assert_eq!(path.full_path(), "crate::Item");
    }
}
