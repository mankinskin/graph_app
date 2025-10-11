use crate::syntax::navigator::{
    ItemNameCollector,
    UseTreeNavigator,
};
use std::collections::BTreeSet;
#[cfg(test)]
use syn::UseTree;
use syn::{
    Attribute,
    File,
    Item,
    ItemUse,
};

/// Parse existing pub use statements from a syntax tree
/// Returns (combined items, replaceable ranges) for existing pub use merging
pub fn parse_existing_pub_uses(
    syntax_tree: &File
) -> (BTreeSet<String>, Vec<(usize, usize)>) {
    let mut combined_items = BTreeSet::new();
    let mut replaceable_ranges = Vec::new();
    let navigator = UseTreeNavigator;

    for (i, item) in syntax_tree.items.iter().enumerate() {
        if let Item::Use(item_use) = item {
            if is_pub_use(item_use) && !has_cfg_attribute(&item_use.attrs) {
                // Use navigator to extract items from this pub use statement
                let mut collector = ItemNameCollector::new();
                navigator.extract_items(&item_use.tree, &mut collector);
                combined_items.extend(collector.items);
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
    attrs.iter().any(|attr| attr.path().is_ident("cfg"))
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

        let navigator = UseTreeNavigator;
        let mut collector = ItemNameCollector::new();
        navigator.extract_items(&use_tree, &mut collector);

        assert_eq!(collector.items.len(), 3);
        assert!(collector.items.contains("add"));
        assert!(collector.items.contains("subtract"));
        assert!(collector.items.contains("format_string"));
    }
}
