use std::collections::BTreeSet;
use syn::{
    Attribute,
    File,
    Item,
    ItemUse,
    UseTree,
};

/// Parse existing pub use statements from a syntax tree
/// Returns (combined items, replaceable ranges) for existing pub use merging
pub fn parse_existing_pub_uses(syntax_tree: &File) -> (BTreeSet<String>, Vec<(usize, usize)>) {
    let mut combined_items = BTreeSet::new();
    let mut replaceable_ranges = Vec::new();

    for (i, item) in syntax_tree.items.iter().enumerate() {
        if let Item::Use(item_use) = item {
            if is_pub_use(item_use) && !has_cfg_attribute(&item_use.attrs) {
                // Extract items from this pub use statement
                extract_use_items(&item_use.tree, &mut combined_items);
                replaceable_ranges.push((i, i + 1));
            }
        }
    }

    (combined_items, replaceable_ranges)
}

/// Check if a use statement is public
fn is_pub_use(item_use: &ItemUse) -> bool {
    matches!(item_use.vis, syn::Visibility::Public(_))
}

/// Check if attributes contain cfg directives
fn has_cfg_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
    })
}

/// Extract all items from a use tree recursively
fn extract_use_items(use_tree: &UseTree, items: &mut BTreeSet<String>) {
    match use_tree {
        UseTree::Path(path) => {
            extract_use_items(&path.tree, items);
        }
        UseTree::Name(name) => {
            items.insert(name.ident.to_string());
        }
        UseTree::Group(group) => {
            for item in &group.items {
                extract_use_items(item, items);
            }
        }
        UseTree::Glob(_) => {
            // Glob imports don't add specific items
        }
        UseTree::Rename(rename) => {
            items.insert(rename.rename.to_string());
        }
    }
}

/// Merge existing pub use items with new items and generate consolidated statements
pub fn merge_pub_uses(
    _existing_items: BTreeSet<String>,
    new_items: &BTreeSet<String>,
    crate_name: &str, // Used to strip the prefix from new_items
) -> Vec<String> {
    if new_items.is_empty() {
        return Vec::new();
    }

    // Strip the crate prefix from all items (e.g., "source_crate::math::add" -> "math::add")
    let stripped_items: Vec<String> = new_items
        .iter()
        .filter_map(|item| {
            // Try to strip both "crate_name::" and "crate::" prefixes
            item.strip_prefix(&format!("{}::", crate_name))
                .or_else(|| item.strip_prefix("crate::"))
                .map(|s| s.to_string())
        })
        .collect();

    if stripped_items.is_empty() {
        return Vec::new();
    }

    // For simplicity, just create one consolidated pub use statement using "crate"
    let items_str = stripped_items.join(", ");
    
    vec![format!("pub use crate::{{{}}};", items_str)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_existing_pub_uses() {
        let file: File = parse_quote! {
            pub use crate::math;
            pub use crate::utils::string_ops;
            use crate::private_item; // Not pub, should be ignored
            
            pub struct Config {}
        };

        let (items, ranges) = parse_existing_pub_uses(&file);
        
        assert_eq!(items.len(), 2);
        assert!(items.contains("math"));
        assert!(items.contains("string_ops"));
        assert_eq!(ranges.len(), 2);
    }

    #[test] 
    fn test_extract_use_items() {
        let use_tree: UseTree = parse_quote! { 
            crate::{math::{add, subtract}, utils::format_string}
        };
        
        let mut items = BTreeSet::new();
        extract_use_items(&use_tree, &mut items);
        
        assert_eq!(items.len(), 3);
        assert!(items.contains("add"));
        assert!(items.contains("subtract"));
        assert!(items.contains("format_string"));
    }
}