//! Utility for analyzing which exported items are used in files
//! This is crucial for self-refactor mode to determine where to add import statements

use std::{
    collections::BTreeSet,
    path::Path,
};

use crate::{
    syntax::{
        item_info::ItemInfo,
        navigator::{UseTreeNavigator, ItemNameCollector},
    },
    analysis::macro_scanning::scan_crate_for_exported_macros,
};
pub struct ExportAnalyzer {
    pub verbose: bool,
}

impl ExportAnalyzer {
    /// Collect existing exported items from pub use statements and direct definitions
    pub fn collect_existing_exports(
        &self,
        syntax_tree: &syn::File,
        source_crate_path: &Path,
    ) -> BTreeSet<String> {
        let mut exported_items = BTreeSet::new();

        // Collect from lib.rs (direct definitions and pub use statements)
        self.collect_exports_from_file(syntax_tree, &mut exported_items);

        // Scan all source files for exported macros
        if let Ok(crate_exported_macros) =
            scan_crate_for_exported_macros(source_crate_path, self.verbose)
        {
            for macro_name in crate_exported_macros {
                exported_items.insert(macro_name);
            }
        }

        exported_items
    }

    /// Collect exported items from a single file's syntax tree
    fn collect_exports_from_file(
        &self,
        syntax_tree: &syn::File,
        exported_items: &mut BTreeSet<String>,
    ) {
        for item in &syntax_tree.items {
            match item {
                // Collect from pub use statements
                syn::Item::Use(use_item) =>
                    if use_item.is_public() {
                        let navigator = UseTreeNavigator;
                        let mut collector = ItemNameCollector::new();
                        navigator.extract_items(&use_item.tree, &mut collector);
                        exported_items.extend(collector.items);
                    },
                item => {
                    if let Some(ident) = item
                        .is_public()
                        .then(|| item.get_identifier())
                        .flatten()
                    {
                        exported_items.insert(ident);
                    }
                },
            }
        }
    }
}
