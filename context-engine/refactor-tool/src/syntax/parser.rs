use anyhow::{Context, Result};
use std::{
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use syn::visit::Visit;
use walkdir::WalkDir;

use crate::{
    analysis::crates::CratePaths,
    syntax::navigator::{UseTreeItemCollector, UseTreeNavigator},
};

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub file_path: PathBuf,
    pub import_path: String,
    pub line_number: usize,
    pub imported_items: Vec<String>,
}

impl PartialEq for ImportInfo {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        // Only compare file_path and import_path for deduplication
        self.file_path == other.file_path
            && self.import_path == other.import_path
    }
}

impl Eq for ImportInfo {}

impl Hash for ImportInfo {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        // Only hash file_path and import_path for deduplication
        // This allows ImportInfo to be used directly in IndexSet
        self.file_path.hash(state);
        self.import_path.hash(state);
    }
}

impl ImportInfo {
    /// Normalize explicit crate name imports to crate:: format
    /// This converts imports like `my_crate::module::Item` to `crate::module::Item`
    pub fn normalize_to_crate_format(
        &mut self,
        crate_name: &str,
    ) {
        let crate_name_prefix = format!("{}::", crate_name);

        // Normalize the import path
        if self.import_path.starts_with(&crate_name_prefix) {
            self.import_path =
                self.import_path.replace(&crate_name_prefix, "crate::");
        }

        // Normalize the imported items
        for item in &mut self.imported_items {
            if item.starts_with(&crate_name_prefix) {
                *item = item.replace(&crate_name_prefix, "crate::");
            }
        }
    }

    /// Normalize super:: imports to crate:: format by resolving relative paths
    /// This converts imports like `super::module::Item` to `crate::parent_module::module::Item`
    pub fn normalize_super_imports(
        &mut self,
        crate_root: &Path,
    ) -> Result<()> {
        use crate::core::path::{
            is_super_import, resolve_super_to_crate_path, ImportPath,
        };

        // Check if this is a super import
        if !is_super_import(&self.import_path) {
            return Ok(()); // Not a super import, nothing to do
        }

        // Parse and resolve the super import path to its crate:: equivalent
        let resolved_path = ImportPath::parse_and_resolve_super(
            &self.import_path,
            &self.file_path,
            crate_root,
        )?;

        // Update the import path
        self.import_path = resolved_path.full_path();

        // Update imported items - convert any super:: references
        for item in &mut self.imported_items {
            if is_super_import(item) {
                let item_path = ImportPath::parse(item)?;
                let resolved_item = resolve_super_to_crate_path(
                    &self.file_path,
                    crate_root,
                    &item_path.segments,
                    &item_path.final_item,
                )?;
                *item = resolved_item.full_path();
            }
        }

        Ok(())
    }
}

pub struct ImportParser {
    crate_name: String,
}

impl ImportParser {
    pub fn new(source_crate_name: &str) -> Self {
        Self {
            crate_name: source_crate_name.replace('-', "_"), // Convert hyphens to underscores for import matching
        }
    }

    pub fn find_imports_in_crates(
        &self,
        crate_paths: &CratePaths,
    ) -> Result<Vec<ImportInfo>> {
        match crate_paths {
            CratePaths::SelfRefactor { crate_path } => {
                self.find_imports_in_crate(crate_path)
            },
            CratePaths::CrossRefactor {
                source_crate_path: _,
                target_crate_path,
            } => self.find_imports_in_crate(target_crate_path),
        }
    }
    pub fn find_imports_in_crate(
        &self,
        crate_path: &Path,
    ) -> Result<Vec<ImportInfo>> {
        let mut imports = Vec::new();
        let src_path = crate_path.join("src");

        if !src_path.exists() {
            return Ok(imports);
        }

        for entry in WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                let file_imports = self.parse_file_imports(path)?;
                imports.extend(file_imports);
            }
        }

        Ok(imports)
    }

    /// Find all super:: imports in a crate
    pub fn find_super_imports_in_crate(
        crate_path: &Path
    ) -> Result<Vec<ImportInfo>> {
        let mut imports = Vec::new();
        let src_path = crate_path.join("src");

        if !src_path.exists() {
            return Ok(imports);
        }

        for entry in WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                let file_imports = Self::parse_file_super_imports(path)?;
                imports.extend(file_imports);
            }
        }

        Ok(imports)
    }

    /// Parse super:: imports from a single file
    fn parse_file_super_imports(file_path: &Path) -> Result<Vec<ImportInfo>> {
        let content = fs::read_to_string(file_path).with_context(|| {
            format!("Failed to read file: {}", file_path.display())
        })?;

        let syntax_tree = syn::parse_file(&content).with_context(|| {
            format!("Failed to parse Rust file: {}", file_path.display())
        })?;

        let mut visitor = SuperImportVisitor::new(file_path.to_path_buf());
        visitor.visit_file(&syntax_tree);

        Ok(visitor.imports)
    }

    fn parse_file_imports(
        &self,
        file_path: &Path,
    ) -> Result<Vec<ImportInfo>> {
        let content = fs::read_to_string(file_path).with_context(|| {
            format!("Failed to read file: {}", file_path.display())
        })?;

        let syntax_tree = syn::parse_file(&content).with_context(|| {
            format!("Failed to parse Rust file: {}", file_path.display())
        })?;

        let mut visitor =
            ImportVisitor::new(&self.crate_name, file_path.to_path_buf());
        visitor.visit_file(&syntax_tree);

        Ok(visitor.imports)
    }
}

struct ImportVisitor {
    source_crate_name: String,
    file_path: PathBuf,
    imports: Vec<ImportInfo>,
    navigator: UseTreeNavigator,
}

impl ImportVisitor {
    fn new(
        source_crate_name: &str,
        file_path: PathBuf,
    ) -> Self {
        Self {
            source_crate_name: source_crate_name.to_string(),
            file_path,
            imports: Vec::new(),
            navigator: UseTreeNavigator,
        }
    }
}

/// Collector that filters for specific crate imports and creates ImportInfo
struct CrateFilteredCollector {
    target_crate: String,
    collected_imports: Vec<(String, Vec<String>)>,
}

impl CrateFilteredCollector {
    fn new(crate_name: &str) -> Self {
        Self {
            target_crate: crate_name.replace('-', "_"),
            collected_imports: Vec::new(),
        }
    }
}

impl UseTreeItemCollector for CrateFilteredCollector {
    fn collect_name(
        &mut self,
        name: &str,
        path: &[String],
    ) {
        if path.is_empty() || path[0] != self.target_crate {
            return;
        }

        let full_path = if path.len() == 1 {
            format!("{}::{}", path[0], name)
        } else {
            format!("{}::{}::{}", path[0], path[1..].join("::"), name)
        };

        self.collected_imports
            .push((full_path.clone(), vec![full_path]));
    }

    fn collect_glob(
        &mut self,
        path: &[String],
    ) {
        if path.is_empty() || path[0] != self.target_crate {
            return;
        }

        let glob_path = format!("{}::*", path.join("::"));
        self.collected_imports
            .push((glob_path, vec!["*".to_string()]));
    }

    fn collect_rename(
        &mut self,
        original: &str,
        renamed: &str,
        path: &[String],
    ) {
        if path.is_empty() || path[0] != self.target_crate {
            return;
        }

        let full_path = if path.len() == 1 {
            format!("{}::{}", path[0], original)
        } else {
            format!("{}::{}::{}", path[0], path[1..].join("::"), original)
        };

        let display_path = format!("{} as {}", full_path, renamed);
        self.collected_imports.push((display_path, vec![full_path]));
    }
}

impl<'ast> Visit<'ast> for ImportVisitor {
    fn visit_item_use(
        &mut self,
        node: &'ast syn::ItemUse,
    ) {
        // Use the navigator to collect imports filtered by crate
        let mut collector =
            CrateFilteredCollector::new(&self.source_crate_name);
        self.navigator.extract_items(&node.tree, &mut collector);

        // Convert collected imports to ImportInfo objects
        for (import_path, imported_items) in collector.collected_imports {
            self.imports.push(ImportInfo {
                file_path: self.file_path.clone(),
                import_path,
                line_number: 0, // We'll rely on string matching instead of line numbers
                imported_items,
            });
        }
    }
}

/// Visitor for collecting super:: imports
struct SuperImportVisitor {
    file_path: PathBuf,
    imports: Vec<ImportInfo>,
    navigator: UseTreeNavigator,
}

impl SuperImportVisitor {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            imports: Vec::new(),
            navigator: UseTreeNavigator,
        }
    }
}

/// Collector that filters for super:: imports specifically
struct SuperImportCollector {
    collected_imports: Vec<(String, Vec<String>)>,
}

impl SuperImportCollector {
    fn new() -> Self {
        Self {
            collected_imports: Vec::new(),
        }
    }
}

impl UseTreeItemCollector for SuperImportCollector {
    fn collect_name(
        &mut self,
        name: &str,
        path: &[String],
    ) {
        if path.is_empty() || path[0] != "super" {
            return;
        }

        let full_path = if path.len() == 1 {
            format!("{}::{}", path[0], name)
        } else {
            format!("{}::{}::{}", path[0], path[1..].join("::"), name)
        };

        self.collected_imports
            .push((full_path.clone(), vec![full_path]));
    }

    fn collect_glob(
        &mut self,
        path: &[String],
    ) {
        if path.is_empty() || path[0] != "super" {
            return;
        }

        let glob_path = format!("{}::*", path.join("::"));
        self.collected_imports
            .push((glob_path, vec!["*".to_string()]));
    }

    fn collect_rename(
        &mut self,
        original: &str,
        renamed: &str,
        path: &[String],
    ) {
        if path.is_empty() || path[0] != "super" {
            return;
        }

        let full_path = if path.len() == 1 {
            format!("{}::{}", path[0], original)
        } else {
            format!("{}::{}::{}", path[0], path[1..].join("::"), original)
        };

        let display_path = format!("{} as {}", full_path, renamed);
        self.collected_imports.push((display_path, vec![full_path]));
    }
}

impl<'ast> Visit<'ast> for SuperImportVisitor {
    fn visit_item_use(
        &mut self,
        node: &'ast syn::ItemUse,
    ) {
        // Use the navigator to collect super:: imports
        let mut collector = SuperImportCollector::new();
        self.navigator.extract_items(&node.tree, &mut collector);

        // Convert collected imports to ImportInfo objects
        for (import_path, imported_items) in collector.collected_imports {
            self.imports.push(ImportInfo {
                file_path: self.file_path.clone(),
                import_path,
                line_number: 0, // We'll rely on string matching instead of line numbers
                imported_items,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_normalize_to_crate_format() {
        let mut import_info = ImportInfo {
            file_path: PathBuf::from("test.rs"),
            import_path: "my_crate::module::Item".to_string(),
            line_number: 1,
            imported_items: vec![
                "my_crate::module::Item".to_string(),
                "other_crate::Item".to_string(),
                "my_crate::other::Thing".to_string(),
            ],
        };

        import_info.normalize_to_crate_format("my_crate");

        assert_eq!(import_info.import_path, "crate::module::Item");
        assert_eq!(
            import_info.imported_items,
            vec![
                "crate::module::Item",
                "other_crate::Item", // Should remain unchanged
                "crate::other::Thing",
            ]
        );
    }

    #[test]
    fn test_normalize_to_crate_format_no_match() {
        let mut import_info = ImportInfo {
            file_path: PathBuf::from("test.rs"),
            import_path: "other_crate::module::Item".to_string(),
            line_number: 1,
            imported_items: vec!["other_crate::module::Item".to_string()],
        };

        import_info.normalize_to_crate_format("my_crate");

        // Should remain unchanged
        assert_eq!(import_info.import_path, "other_crate::module::Item");
        assert_eq!(
            import_info.imported_items,
            vec!["other_crate::module::Item"]
        );
    }

    #[test]
    fn test_import_info_equality_and_hashing() {
        use indexmap::IndexSet;

        let import1 = ImportInfo {
            file_path: PathBuf::from("test.rs"),
            import_path: "crate::module::Item".to_string(),
            line_number: 1,
            imported_items: vec!["Item".to_string()],
        };

        let import2 = ImportInfo {
            file_path: PathBuf::from("test.rs"),
            import_path: "crate::module::Item".to_string(),
            line_number: 2, // Different line number
            imported_items: vec!["Item".to_string(), "Other".to_string()], // Different imported items
        };

        let import3 = ImportInfo {
            file_path: PathBuf::from("other.rs"), // Different file
            import_path: "crate::module::Item".to_string(),
            line_number: 1,
            imported_items: vec!["Item".to_string()],
        };

        // Same file and import path should be equal (ignoring line number and imported items)
        assert_eq!(import1, import2);

        // Different file should not be equal
        assert_ne!(import1, import3);

        // Test that they work correctly in IndexSet
        let mut set = IndexSet::new();
        set.insert(import1);
        set.insert(import2); // Should not be inserted (duplicate)
        set.insert(import3); // Should be inserted (different file)

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_efficient_deduplication() {
        use indexmap::IndexSet;

        // Create test imports with duplicates
        let imports = vec![
            ImportInfo {
                file_path: PathBuf::from("lib.rs"),
                import_path: "crate::module::Item".to_string(),
                line_number: 1,
                imported_items: vec!["Item".to_string()],
            },
            ImportInfo {
                file_path: PathBuf::from("lib.rs"),
                import_path: "crate::module::Item".to_string(), // Duplicate
                line_number: 5,
                imported_items: vec!["Item".to_string(), "Other".to_string()],
            },
            ImportInfo {
                file_path: PathBuf::from("lib.rs"),
                import_path: "crate::other::Thing".to_string(),
                line_number: 10,
                imported_items: vec!["Thing".to_string()],
            },
            ImportInfo {
                file_path: PathBuf::from("main.rs"),
                import_path: "crate::module::Item".to_string(), // Not a duplicate (different file)
                line_number: 2,
                imported_items: vec!["Item".to_string()],
            },
        ];

        // Use IndexSet for automatic deduplication
        let deduplicated: IndexSet<ImportInfo> = imports.into_iter().collect();

        // Should have 3 unique imports (original had 1 duplicate in same file)
        assert_eq!(deduplicated.len(), 3);

        // Convert to Vec for assertions (IndexSet maintains insertion order)
        let deduplicated_vec: Vec<_> = deduplicated.into_iter().collect();

        // Verify the right ones were kept (first occurrence of each unique import)
        assert_eq!(deduplicated_vec[0].file_path, PathBuf::from("lib.rs"));
        assert_eq!(deduplicated_vec[0].import_path, "crate::module::Item");
        assert_eq!(deduplicated_vec[0].line_number, 1); // First occurrence kept

        assert_eq!(deduplicated_vec[1].file_path, PathBuf::from("lib.rs"));
        assert_eq!(deduplicated_vec[1].import_path, "crate::other::Thing");

        assert_eq!(deduplicated_vec[2].file_path, PathBuf::from("main.rs"));
        assert_eq!(deduplicated_vec[2].import_path, "crate::module::Item");
    }

    #[test]
    fn test_normalize_super_imports() {
        use std::path::PathBuf;

        let crate_root = PathBuf::from("/workspace/my_crate");
        let file_path =
            PathBuf::from("/workspace/my_crate/src/submodule/file.rs");

        let mut import_info = ImportInfo {
            file_path: file_path.clone(),
            import_path: "super::utils::Item".to_string(),
            line_number: 1,
            imported_items: vec!["super::utils::Item".to_string()],
        };

        import_info.normalize_super_imports(&crate_root).unwrap();

        // Should be converted to crate::utils::Item (since we're in submodule, super takes us to crate root)
        assert_eq!(import_info.import_path, "crate::utils::Item");
        assert_eq!(import_info.imported_items, vec!["crate::utils::Item"]);
    }

    #[test]
    fn test_normalize_super_imports_no_change() {
        use std::path::PathBuf;

        let crate_root = PathBuf::from("/workspace/my_crate");
        let file_path = PathBuf::from("/workspace/my_crate/src/file.rs");

        let mut import_info = ImportInfo {
            file_path: file_path.clone(),
            import_path: "crate::module::Item".to_string(),
            line_number: 1,
            imported_items: vec!["crate::module::Item".to_string()],
        };

        import_info.normalize_super_imports(&crate_root).unwrap();

        // Should remain unchanged since it's not a super import
        assert_eq!(import_info.import_path, "crate::module::Item");
        assert_eq!(import_info.imported_items, vec!["crate::module::Item"]);
    }
}
