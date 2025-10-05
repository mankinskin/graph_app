use crate::{
    import_parser::ImportInfo,
    item_info::ItemInfo,
    utils::{
        file_operations::{
            check_crate_compilation,
            get_relative_path_for_display,
            read_and_parse_file,
            write_file,
        },
        import_analysis::{
            analyze_crate_imports,
            analyze_imports,
            print_crate_analysis_summary,
            print_import_analysis_summary,
        },
        import_replacement::{
            replace_crate_imports,
            replace_target_imports,
        },
        macro_scanning::scan_crate_for_exported_macros,
        pub_use_generation::{
            collect_existing_pub_uses,
            extract_exported_items_from_use_tree,
            generate_nested_pub_use,
        },
    },
};
use anyhow::{
    bail,
    Result,
};
use std::{
    collections::BTreeSet,
    path::Path,
};

pub struct RefactorEngine {
    source_crate_name: String,
    dry_run: bool,
    verbose: bool,
}

impl RefactorEngine {
    pub fn new(
        source_crate_name: &str,
        dry_run: bool,
        verbose: bool,
    ) -> Self {
        Self {
            source_crate_name: source_crate_name.replace('-', "_"), // Convert hyphens to underscores for import matching
            dry_run,
            verbose,
        }
    }

    pub fn refactor_imports(
        &mut self,
        source_crate_path: &Path,
        target_crate_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        // Step 1: Analyze and categorize imports
        let analysis_result =
            analyze_imports(&imports, &self.source_crate_name, workspace_root);

        // Enhanced output showing analysis results
        print_import_analysis_summary(
            &analysis_result,
            &imports,
            &self.source_crate_name,
        );

        // Step 2: Update source crate's lib.rs with pub use statements
        self.update_source_lib_rs(
            source_crate_path,
            &analysis_result.all_imported_items,
            workspace_root,
        )?;

        // Step 3: Replace imports in target crate files
        replace_target_imports(
            imports,
            &self.source_crate_name,
            workspace_root,
            self.dry_run,
            self.verbose,
        )?;

        // Always check compilation after refactoring to ensure we didn't break anything
        if !self.dry_run {
            println!("🔧 Checking compilation after modifications...");
            let source_compiles =
                check_crate_compilation(source_crate_path, self.verbose)?;
            let target_compiles =
                check_crate_compilation(target_crate_path, self.verbose)?;

            if !source_compiles {
                bail!("Source crate failed to compile after refactoring. This indicates a bug in the refactor tool.");
            }

            if !target_compiles {
                bail!("Target crate failed to compile after refactoring. This indicates a bug in the refactor tool.");
            }

            if self.verbose {
                println!("✅ Both source and target crates compile successfully after refactoring");
            }
        }

        Ok(())
    }

    /// Refactor internal crate:: imports within a single crate
    /// This moves crate:: imports to be pub use exports at the crate root level
    pub fn refactor_self_imports(
        &mut self,
        crate_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        // Step 1: Analyze and categorize crate:: imports
        let analysis_result = analyze_crate_imports(&imports, workspace_root);

        // Enhanced output showing analysis results
        print_crate_analysis_summary(&analysis_result, &imports);

        // Step 2: Update the crate's lib.rs with pub use statements
        self.update_source_lib_rs(
            crate_path,
            &analysis_result.all_imported_items,
            workspace_root,
        )?;

        // Step 3: Replace crate:: imports in the same crate
        replace_crate_imports(
            imports,
            workspace_root,
            self.dry_run,
            self.verbose,
        )?;

        // Always check compilation after refactoring to ensure we didn't break anything
        if !self.dry_run {
            println!("🔧 Checking compilation after modifications...");
            let crate_compiles =
                check_crate_compilation(crate_path, self.verbose)?;

            if !crate_compiles {
                bail!("Crate failed to compile after self-refactoring. This indicates a bug in the refactor tool.");
            }

            if self.verbose {
                println!(
                    "✅ Crate compiles successfully after self-refactoring"
                );
            }
        }

        Ok(())
    }

    fn update_source_lib_rs(
        &self,
        source_crate_path: &Path,
        imported_items: &BTreeSet<String>,
        workspace_root: &Path,
    ) -> Result<()> {
        let lib_rs_path = source_crate_path.join("src").join("lib.rs");

        if !lib_rs_path.exists() {
            if self.verbose {
                println!("Warning: lib.rs not found at {}, skipping pub use additions", lib_rs_path.display());
            }
            return Ok(());
        }

        let (original_content, syntax_tree) =
            read_and_parse_file(&lib_rs_path)?;

        // Use improved existing pub use collection
        let existing_exports =
            self.collect_existing_exports(&syntax_tree, source_crate_path);

        if self.verbose {
            println!(
                "🔍 Found {} existing exported items:",
                existing_exports.len()
            );
            for item in &existing_exports {
                println!("  • {}", item);
            }
        }

        // Filter out items that are already exported
        let items_to_add: BTreeSet<String> = imported_items
            .iter()
            .filter(|item| {
                let item_name = if item
                    .starts_with(&format!("{}::", &self.source_crate_name))
                {
                    item.strip_prefix(&format!("{}::", &self.source_crate_name))
                        .unwrap_or(item)
                } else {
                    item
                };

                // Check if the final identifier is already exported
                let final_ident =
                    item_name.split("::").last().unwrap_or(item_name);

                if existing_exports.contains(final_ident) {
                    if self.verbose {
                        println!(
                            "  ⚠️  Skipping '{}' - already exported",
                            final_ident
                        );
                    }
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        if items_to_add.is_empty() {
            if self.verbose {
                let relative_path =
                    get_relative_path_for_display(&lib_rs_path, workspace_root);
                println!(
                    "✅ No new pub use statements needed for {} (all items already exported)",
                    relative_path.display()
                );
            }
            return Ok(());
        }

        // Collect conditional items for feature flag grouping
        let (_, conditional_items) = collect_existing_pub_uses(&syntax_tree);

        // Generate nested pub use statements for the filtered items
        let nested_pub_use = generate_nested_pub_use(
            &items_to_add,
            &BTreeSet::new(), // Empty since we already filtered
            &conditional_items,
            &self.source_crate_name,
            self.verbose,
        );

        if nested_pub_use.is_empty() {
            if self.verbose {
                let relative_path =
                    get_relative_path_for_display(&lib_rs_path, workspace_root);
                println!(
                    "✅ No new pub use statements needed for {}",
                    relative_path.display()
                );
            }
            return Ok(());
        }

        // Insert new pub use statements at the end of the file
        let mut new_content = original_content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push('\n');
        new_content.push_str("// Auto-generated pub use statements\n");
        new_content.push_str(&nested_pub_use);

        if self.verbose {
            let relative_path =
                get_relative_path_for_display(&lib_rs_path, workspace_root);
            println!(
                "Adding nested pub use statement to {}",
                relative_path.display()
            );
            println!("{}", nested_pub_use.trim());
        }

        if !self.dry_run {
            write_file(&lib_rs_path, &new_content)?;
        }

        Ok(())
    }

    /// Collect existing exported items from pub use statements and direct definitions
    fn collect_existing_exports(
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
                        extract_exported_items_from_use_tree(
                            &use_item.tree,
                            exported_items,
                        );
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
