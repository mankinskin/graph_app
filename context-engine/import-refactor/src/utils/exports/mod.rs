use std::collections::BTreeSet;

pub mod analyzer;
pub mod macro_scanning;

/// Recursively extract exported item names from a use tree
pub fn extract_exported_items_from_use_tree(
    tree: &syn::UseTree,
    exported_items: &mut BTreeSet<String>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            extract_exported_items_from_use_tree(&path.tree, exported_items);
        },
        syn::UseTree::Name(name) => {
            exported_items.insert(name.ident.to_string());
        },
        syn::UseTree::Rename(rename) => {
            exported_items.insert(rename.rename.to_string());
        },
        syn::UseTree::Glob(_) => {
            exported_items.insert("*".to_string());
        },
        syn::UseTree::Group(group) =>
            for item in &group.items {
                extract_exported_items_from_use_tree(item, exported_items);
            },
    }
}
