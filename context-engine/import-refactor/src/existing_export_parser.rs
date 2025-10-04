use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use syn::{File, Item, UseTree};

/// Information about existing pub use statements in a crate
#[derive(Default)]
pub struct ExistingExports {
    /// All exported items (flattened from nested structures)
    pub exported_items: BTreeSet<String>,
    /// Conditional exports with their cfg attributes
    pub conditional_exports: BTreeMap<String, syn::Attribute>,
    /// Raw pub use statements (for debugging)
    pub raw_statements: Vec<String>,
}

impl std::fmt::Debug for ExistingExports {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExistingExports")
            .field("exported_items", &self.exported_items)
            .field("conditional_exports_count", &self.conditional_exports.len())
            .field("raw_statements", &self.raw_statements)
            .finish()
    }
}

/// Parser for existing pub use statements
pub struct ExistingExportParser;

impl ExistingExportParser {
    /// Parse existing pub use statements from a syntax tree
    pub fn parse_existing_exports(syntax_tree: &File) -> Result<ExistingExports> {
        let mut exports = ExistingExports::default();

        for item in &syntax_tree.items {
            if let Item::Use(use_item) = item {
                if matches!(use_item.vis, syn::Visibility::Public(_)) {
                    // Store raw statement for debugging
                    let raw_stmt = quote::quote!(#use_item).to_string();
                    exports.raw_statements.push(raw_stmt);

                    // Extract all exported items from this use statement
                    Self::extract_exported_items(&use_item.tree, &mut exports.exported_items, &use_item.attrs);
                }
            }
        }

        Ok(exports)
    }

    /// Recursively extract all exported items from a use tree
    fn extract_exported_items(
        tree: &UseTree,
        exported_items: &mut BTreeSet<String>,
        attrs: &[syn::Attribute],
    ) {
        match tree {
            UseTree::Path(path) => {
                // Handle paths like `utils::string_ops::capitalize`
                Self::extract_exported_items(&path.tree, exported_items, attrs);
            }
            UseTree::Name(name) => {
                // Handle simple names like `format_string`
                let item_name = name.ident.to_string();
                exported_items.insert(item_name);
            }
            UseTree::Rename(rename) => {
                // Handle renames like `format_string as fmt`
                let renamed = rename.rename.to_string();
                exported_items.insert(renamed);
            }
            UseTree::Glob(_) => {
                // Handle glob imports like `module::*`
                // These are harder to track, but we can note them
                exported_items.insert("*".to_string());
            }
            UseTree::Group(group) => {
                // Handle groups like `{capitalize, encoding::Encoder}`
                for item in &group.items {
                    Self::extract_exported_items(item, exported_items, attrs);
                }
            }
        }
    }

    /// Check if an item is already exported (considering nested paths)
    pub fn is_already_exported(
        item_path: &str,
        source_crate_name: &str,
        exports: &ExistingExports,
    ) -> bool {
        // Extract the final identifier from the path
        let final_ident = item_path
            .split("::")
            .last()
            .unwrap_or(item_path);

        // Check if this identifier is already exported
        if exports.exported_items.contains(final_ident) {
            return true;
        }

        // Check if the full path (relative to crate) is already exported
        if item_path.starts_with(&format!("{}::", source_crate_name)) {
            let relative_path = item_path
                .strip_prefix(&format!("{}::", source_crate_name))
                .unwrap_or(item_path);
            
            // Check various forms of the path
            if exports.exported_items.contains(relative_path) ||
               exports.exported_items.contains(&format!("crate::{}", relative_path)) {
                return true;
            }
        }

        false
    }

    /// Merge new exports with existing ones, avoiding duplicates
    pub fn merge_exports(
        new_items: &BTreeSet<String>,
        source_crate_name: &str,
        existing: &ExistingExports,
    ) -> BTreeSet<String> {
        let mut merged = BTreeSet::new();

        for item in new_items {
            if !Self::is_already_exported(item, source_crate_name, existing) {
                // Convert to relative path format for pub use
                if item.starts_with(&format!("{}::", source_crate_name)) {
                    let relative_path = item
                        .strip_prefix(&format!("{}::", source_crate_name))
                        .unwrap_or(item);
                    merged.insert(relative_path.to_string());
                } else {
                    merged.insert(item.clone());
                }
            }
        }

        merged
    }
}