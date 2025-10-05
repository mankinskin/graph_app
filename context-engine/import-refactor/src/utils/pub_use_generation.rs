use crate::item_info::ItemInfo;
use std::collections::{BTreeMap, BTreeSet};
use syn::File;

/// Generate nested pub use statements for imported items
pub fn generate_nested_pub_use(
    imported_items: &BTreeSet<String>,
    existing_pub_uses: &BTreeSet<String>,
    conditional_items: &BTreeMap<String, Option<syn::Attribute>>,
    source_crate_name: &str,
    verbose: bool,
) -> String {
    // Build a tree structure for the imports using a simplified approach
    let mut paths_to_export = Vec::new();
    let mut conditional_exports = Vec::new();

    for item in imported_items {
        let relative_path = if item.starts_with("crate::") {
            // For self-refactor mode, strip "crate::" prefix
            item.strip_prefix("crate::").unwrap_or(item)
        } else if item.starts_with(&format!("{}::", source_crate_name)) {
            // For cross-crate mode, strip source crate prefix
            item.strip_prefix(&format!("{}::", source_crate_name))
                .unwrap_or(item)
        } else if !item.contains("::") {
            // This is a root-level item (e.g., "hello")
            item
        } else {
            // Skip items that don't match expected patterns
            if verbose {
                println!("  ⚠️  Skipping '{}' - doesn't match expected patterns", item);
            }
            continue;
        };

        // Extract the final identifier to check for conflicts and conditions
        let final_ident = relative_path.split("::").last().unwrap_or(relative_path);

        // Skip if already exists as a use statement
        if existing_pub_uses.contains(&format!("pub use crate::{};", relative_path)) {
            if verbose {
                println!(
                    "  ⚠️  Skipping '{}' - already exists as pub use",
                    final_ident
                );
            }
            continue;
        }

        // Skip if this identifier already exists in the crate
        if existing_pub_uses.contains(final_ident) {
            if verbose {
                println!("  ⚠️  Skipping '{}' - already exists in source crate", final_ident);
            }
            continue;
        }

        // Check if this item has conditional compilation
        if let Some(Some(attr)) = conditional_items.get(final_ident) {
            // This is a conditionally compiled item
            conditional_exports.push((relative_path.to_string(), attr.clone()));
            if verbose {
                println!(
                    "  📝 Found conditional item '{}' with cfg: {}",
                    final_ident,
                    quote::quote!(#attr)
                );
            }
        } else {
            paths_to_export.push(relative_path.to_string());
        }
    }

    // Generate the combined result
    let mut result = String::new();

    // First add conditional exports
    for (path, cfg_attr) in conditional_exports {
        result.push_str(&format!(
            "{}\npub use crate::{};\n",
            quote::quote!(#cfg_attr),
            path
        ));
    }

    // Then add unconditional exports if any
    if !paths_to_export.is_empty() {
        // Sort and deduplicate
        paths_to_export.sort();
        paths_to_export.dedup();

        // Generate nested structure for unconditional items
        result.push_str(&build_nested_structure(paths_to_export));
    }

    result
}

/// Build nested pub use structure from a list of paths
pub fn build_nested_structure(paths: Vec<String>) -> String {
    if paths.is_empty() {
        return String::new();
    }

    // Group paths by their first component
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut direct_exports = Vec::new();

    for path in paths {
        let components: Vec<&str> = path.split("::").collect();
        if components.len() == 1 {
            direct_exports.push(path);
        } else {
            let first = components[0].to_string();
            let rest = components[1..].join("::");
            groups.entry(first).or_default().push(rest);
        }
    }

    let mut result = String::new();
    result.push_str("pub use crate::{\n");

    // Add direct exports first
    for (i, export) in direct_exports.iter().enumerate() {
        result.push_str("    ");
        result.push_str(export);
        if i < direct_exports.len() - 1 || !groups.is_empty() {
            result.push(',');
        }
        result.push('\n');
    }

    // Add grouped exports
    let group_entries: Vec<_> = groups.iter().collect();
    for (i, (module, subpaths)) in group_entries.iter().enumerate() {
        if subpaths.len() == 1 && !subpaths[0].contains("::") {
            // Simple case: module::item
            result.push_str("    ");
            result.push_str(module);
            result.push_str("::");
            result.push_str(&subpaths[0]);
        } else {
            // Complex case: nested structure
            result.push_str("    ");
            result.push_str(module);
            result.push_str("::{\n");

            // Recursively handle subpaths
            let sub_result = build_nested_substructure(subpaths.to_vec(), 2);
            result.push_str(&sub_result);

            result.push_str("    }");
        }

        if i < group_entries.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }

    result.push_str("};\n");
    result
}

/// Build nested substructure for grouped imports
pub fn build_nested_substructure(paths: Vec<String>, indent_level: usize) -> String {
    let indent = "    ".repeat(indent_level);
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut direct_exports = Vec::new();

    for path in paths {
        let components: Vec<&str> = path.split("::").collect();
        if components.len() == 1 {
            direct_exports.push(path);
        } else {
            let first = components[0].to_string();
            let rest = components[1..].join("::");
            groups.entry(first).or_default().push(rest);
        }
    }

    let mut result = String::new();

    // Add direct exports
    for (i, export) in direct_exports.iter().enumerate() {
        result.push_str(&indent);
        result.push_str(export);
        if i < direct_exports.len() - 1 || !groups.is_empty() {
            result.push(',');
        }
        result.push('\n');
    }

    // Add grouped exports
    let group_entries: Vec<_> = groups.iter().collect();
    for (i, (module, subpaths)) in group_entries.iter().enumerate() {
        if subpaths.len() == 1 && !subpaths[0].contains("::") {
            result.push_str(&indent);
            result.push_str(module);
            result.push_str("::");
            result.push_str(&subpaths[0]);
        } else {
            result.push_str(&indent);
            result.push_str(module);
            result.push_str("::{\n");

            let sub_result = build_nested_substructure(
                subpaths.to_vec(),
                indent_level + 1,
            );
            result.push_str(&sub_result);

            result.push_str(&indent);
            result.push('}');
        }

        if i < group_entries.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }

    result
}

/// Collect existing pub use statements and conditional items from a syntax tree
pub fn collect_existing_pub_uses(
    syntax_tree: &File,
) -> (BTreeSet<String>, BTreeMap<String, Option<syn::Attribute>>) {
    let mut existing = BTreeSet::new();
    let mut conditional_items = BTreeMap::new();

    for item in &syntax_tree.items {
        if item.is_public() {
            if let Some(name) = item.get_identifier() {
                existing.insert(name.clone());

                // Check for conditional compilation attributes
                let cfg_attr = extract_cfg_attribute(item.get_attributes());
                if cfg_attr.is_some() {
                    conditional_items.insert(name, cfg_attr);
                }
            }
        }
    }

    (existing, conditional_items)
}

/// Extract cfg attribute from a list of attributes
pub fn extract_cfg_attribute(attrs: &[syn::Attribute]) -> Option<syn::Attribute> {
    for attr in attrs {
        if attr.path().is_ident("cfg") {
            return Some(attr.clone());
        }
    }
    None
}

/// Recursively extract exported item names from a use tree
pub fn extract_exported_items_from_use_tree(
    tree: &syn::UseTree,
    exported_items: &mut BTreeSet<String>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            extract_exported_items_from_use_tree(&path.tree, exported_items);
        }
        syn::UseTree::Name(name) => {
            exported_items.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            exported_items.insert(rename.rename.to_string());
        }
        syn::UseTree::Glob(_) => {
            exported_items.insert("*".to_string());
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                extract_exported_items_from_use_tree(item, exported_items);
            }
        }
    }
}