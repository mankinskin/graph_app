use crate::{
    crate_analyzer::{
        CrateNames,
        CratePaths,
    },
    import_parser::ImportInfo,
    utils::{
        common::format_relative_path,
        exports::analyzer::ExportAnalyzer,
        file_operations::{
            check_crates_compilation,
            read_and_parse_file,
            write_file,
            CompileResults,
        },
        import_analysis::{
            analyze_imports,
            print_analysis_summary,
        },
        import_replacement::{
            replace_imports_with_strategy,
            CrossCrateReplacementStrategy,
        },
        pub_use_generation::{
            collect_existing_pub_uses,
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
    crate_names: CrateNames,
    dry_run: bool,
    verbose: bool,
}

impl RefactorEngine {
    pub fn new(
        crate_names: &CrateNames,
        dry_run: bool,
        verbose: bool,
    ) -> Self {
        Self {
            crate_names: crate_names.unhyphen(), // Convert hyphens to underscores for import matching
            dry_run,
            verbose,
        }
    }

    pub fn refactor_imports(
        &mut self,
        crate_paths: &CratePaths,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        // Step 1: Analyze and categorize imports
        let analysis_result =
            analyze_imports(&imports, &self.crate_names, workspace_root);

        // Enhanced output showing analysis results
        print_analysis_summary(&analysis_result, &imports, &self.crate_names);

        let crate_path = match crate_paths {
            CratePaths::SelfRefactor { crate_path } => crate_path,
            CratePaths::CrossRefactor {
                source_crate_path,
                target_crate_path: _,
            } => source_crate_path,
        };
        // Step 2: Update source crate's lib.rs with pub use statements
        self.update_source_lib_rs(
            crate_path,
            &analysis_result.all_imported_items,
            workspace_root,
        )?;

        // Step 3: Replace imports in target crate files
        let strategy = CrossCrateReplacementStrategy {
            crate_names: self.crate_names.clone(),
        };
        //let strategy = SelfCrateReplacementStrategy;
        replace_imports_with_strategy(
            imports,
            strategy,
            workspace_root,
            self.dry_run,
            self.verbose,
        )?;

        // Always check compilation after refactoring to ensure we didn't break anything
        if !self.dry_run {
            println!("🔧 Checking compilation after modifications...");
            let compile_results =
                check_crates_compilation(crate_paths, self.verbose)?;
            let (source_compiles, target) = match compile_results {
                CompileResults::SelfCrate { self_compiles } =>
                    (self_compiles, None),
                CompileResults::CrossCrate {
                    source_compiles,
                    target_compiles,
                } => (source_compiles, Some(target_compiles)),
            };
            if !source_compiles {
                bail!("Source crate failed to compile after refactoring. This indicates a bug in the refactor tool.");
            }
            if let Some(false) = target {
                bail!("Target crate failed to compile after refactoring. This indicates a bug in the refactor tool.");
            }

            if self.verbose {
                let (crates, s) = if target.is_none() {
                    println!("✅ Source crate compiles successfully after refactoring");
                    ("Source crate", "s")
                } else {
                    ("Both source and target crates", "")
                };
                println!(
                    "✅ {}  compile{} successfully after refactoring",
                    crates, s
                );
            }
        }

        Ok(())
    }
    fn exports(&self) -> ExportAnalyzer {
        ExportAnalyzer {
            verbose: self.verbose,
        }
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
        let existing_exports = self
            .exports()
            .collect_existing_exports(&syntax_tree, source_crate_path);

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
                let item_name = self.crate_names.get_prefixes_to_strip().iter().find_map(|prefix|
                    item.strip_prefix(prefix)
                )
                .unwrap_or(item);

                // Check if the final identifier is already exported
                let final_ident =
                    item_name.split("::").last().unwrap_or(item_name);

                // Only skip if the final identifier is already exported AND
                // the import path has only one component (i.e., it's a direct import)
                let path_components: Vec<&str> = item_name.split("::").collect();
                let is_direct_import = path_components.len() == 1;

                if self.verbose {
                    println!(
                        "  🔍 Analyzing '{}': item_name='{}', final_ident='{}', components={}, is_direct={}, already_exported={}",
                        item, item_name, final_ident, path_components.len(), is_direct_import, existing_exports.contains(final_ident)
                    );
                }

                if existing_exports.contains(final_ident) && is_direct_import {
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
                println!(
                    "✅ No new pub use statements needed for {} (all items already exported)",
                    format_relative_path(&lib_rs_path, workspace_root)
                );
            }
            return Ok(());
        }

        // Collect conditional items for feature flag grouping
        let (_, conditional_items) = collect_existing_pub_uses(&syntax_tree);

        // Generate nested pub use statements for the filtered items
        let all_items_to_export = items_to_add.clone();

        // Also add root-level functions that are already exported and used
        // BUT don't add them if they're already defined in lib.rs to avoid conflicts
        // Check for hello() function usage in app.rs specifically
        let app_rs_path = source_crate_path.join("src").join("app.rs");
        if app_rs_path.exists() && existing_exports.contains("hello") {
            if let Ok(content) = std::fs::read_to_string(&app_rs_path) {
                if content.contains("hello()") {
                    println!("🔍 Debug: Found usage of root-level function 'hello' in app.rs, but NOT adding to pub use (already defined in lib.rs)");
                    // Don't add to all_items_to_export - it's already defined in lib.rs
                    // Instead, we need to ensure usage analysis includes it
                }
            }
        }

        println!("🔍 Debug: all_items_to_export before generate_nested_pub_use: {:?}", all_items_to_export);

        let nested_pub_use = generate_nested_pub_use(
            &all_items_to_export,
            &BTreeSet::new(), // Empty since we already filtered
            &conditional_items,
            &self.crate_names,
            self.verbose,
        );

        if nested_pub_use.is_empty() {
            if self.verbose {
                println!(
                    "✅ No new pub use statements needed for {}",
                    format_relative_path(&lib_rs_path, workspace_root)
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
            println!(
                "Adding nested pub use statement to {}",
                format_relative_path(&lib_rs_path, workspace_root)
            );
            println!("{}", nested_pub_use.trim());
        }

        if !self.dry_run {
            write_file(&lib_rs_path, &new_content)?;
        }

        Ok(())
    }
}
