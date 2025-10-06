use crate::{
    analysis::{
        crates::{
            CrateNames,
            CratePaths,
        },
        imports::{
            analyze_imports,
            print_analysis_summary,
        },
        ExportAnalyzer,
    },
    common::path::format_relative_path,
    core::{
        ast_manager::AstManager,
        path::ImportPath,
    },
    io::files::{
        check_crates_compilation,
        read_and_parse_file,
        write_file,
        CompileResults,
    },
    syntax::{
        parser::ImportInfo,
        transformer::{
            replace_imports_with_strategy,
            CrossCrateReplacementStrategy,
            SelfCrateReplacementStrategy,
        },
        visitor::{
            merge_pub_uses,
            parse_existing_pub_uses,
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
    ast_manager: AstManager,
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
            ast_manager: AstManager::new(verbose),
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
        print_analysis_summary(
            &analysis_result,
            &imports,
            &self.crate_names,
            self.verbose,
        );

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
        match &self.crate_names {
            CrateNames::CrossRefactor { .. } => {
                let strategy = CrossCrateReplacementStrategy {
                    crate_names: self.crate_names.clone(),
                };
                replace_imports_with_strategy(
                    imports,
                    strategy,
                    workspace_root,
                    self.dry_run,
                    self.verbose,
                )?;
            },
            CrateNames::SelfRefactor { .. } => {
                let strategy = SelfCrateReplacementStrategy;
                replace_imports_with_strategy(
                    imports,
                    strategy,
                    workspace_root,
                    self.dry_run,
                    self.verbose,
                )?;
            },
        }

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
        &mut self,
        source_crate_path: &Path,
        imported_items: &BTreeSet<String>,
        _workspace_root: &Path,
    ) -> Result<()> {
        let lib_rs_path = source_crate_path.join("src").join("lib.rs");

        if !lib_rs_path.exists() {
            if self.verbose {
                println!("Warning: lib.rs not found at {}, skipping pub use additions", lib_rs_path.display());
            }
            return Ok(());
        }

        // Get exports analyzer first to avoid borrowing conflicts
        let exports_analyzer = self.exports();

        // Use AST manager for cached parsing
        let syntax_tree = self.ast_manager.get_or_parse(&lib_rs_path)?;

        // Parse existing pub use statements into a tree structure
        let (existing_tree, _replaceable_ranges) =
            parse_existing_pub_uses(syntax_tree);

        // Use improved existing pub use collection for final identifier checking
        let existing_exports = exports_analyzer
            .collect_existing_exports(syntax_tree, source_crate_path);

        if self.verbose {
            println!(
                "🔍 Found {} existing exported items:",
                existing_exports.len()
            );
            for item in &existing_exports {
                println!("  • {}", item);
            }
        }

        // Convert string items to structured ImportPath objects for better processing
        let import_paths: Vec<ImportPath> = imported_items
            .iter()
            .filter_map(|item| ImportPath::parse(item).ok())
            .collect();

        // Filter out items that are already exported using structured path analysis
        let items_to_add: BTreeSet<String> = import_paths
            .iter()
            .filter(|import_path| {
                // Strip the crate prefix to get relative path
                let relative_path = match &self.crate_names {
                    CrateNames::CrossRefactor { source_crate, .. } =>
                        import_path.strip_crate_prefix(source_crate),
                    CrateNames::SelfRefactor { crate_name } => import_path
                        .strip_crate_prefix(crate_name)
                        .or_else(|| import_path.strip_crate_prefix("crate")),
                };

                if let Some(rel_path) = relative_path {
                    // Only skip if the final identifier is already exported AND
                    // the import path is a direct import (no intermediate segments)
                    let final_ident = &import_path.final_item;

                    if existing_exports.contains(final_ident)
                        && import_path.is_direct_import()
                    {
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
                } else {
                    true // Keep items we can't parse properly
                }
            })
            .map(|path| path.full_path())
            .collect();

        if items_to_add.is_empty() {
            if self.verbose {
                println!(
                    "✅ No new pub use statements needed for {} (all items already exported)",
                    format_relative_path(&lib_rs_path)
                );
            }
            return Ok(());
        }

        // Use intelligent merger to combine existing and new pub use statements
        let crate_name = match &self.crate_names {
            CrateNames::CrossRefactor { source_crate, .. } => source_crate,
            CrateNames::SelfRefactor { crate_name } => crate_name,
        };

        let merged_statements =
            merge_pub_uses(existing_tree, &items_to_add, crate_name);

        if merged_statements.is_empty() {
            if self.verbose {
                println!(
                    "✅ No new pub use statements needed for {}",
                    format_relative_path(&lib_rs_path)
                );
            }
            return Ok(());
        }

        // Read original content for modification (since AST manager only gives parsed form)
        let original_content = std::fs::read_to_string(&lib_rs_path)?;
        let mut new_content = String::new();
        let lines: Vec<&str> = original_content.lines().collect();
        let mut skip_until_semicolon = false;

        for line in lines {
            let trimmed = line.trim();

            // Check if this line starts a replaceable pub use statement
            if trimmed.starts_with("pub use") && !trimmed.contains("#[cfg") {
                // Check if it's a local crate import (not external)
                if trimmed.contains("::") {
                    let _after_use =
                        trimmed.strip_prefix("pub use").unwrap().trim();
                    // Skip this pub use statement to avoid duplicates
                    skip_until_semicolon = true;
                    continue;
                } else {
                    // Simple pub use statement like "pub use math"
                    skip_until_semicolon = true;
                    continue;
                }
            }

            // If we're skipping a multi-line pub use statement
            if skip_until_semicolon {
                if trimmed.ends_with(';') {
                    skip_until_semicolon = false; // End of statement
                }
                continue; // Skip this line
            }

            new_content.push_str(line);
            new_content.push('\n');
        }

        // Add merged pub use statements at the end
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push('\n');
        new_content.push_str("// Merged pub use statements\n");
        for statement in &merged_statements {
            new_content.push_str(statement);
            new_content.push('\n');
        }

        if self.verbose {
            println!(
                "🔄 Replacing existing pub use statements in {} with merged statements:",
                format_relative_path(&lib_rs_path)
            );
            for statement in &merged_statements {
                println!("  {}", statement.trim());
            }
        }

        if !self.dry_run {
            write_file(&lib_rs_path, &new_content)?;
            // Invalidate AST cache since we modified the file
            self.ast_manager.invalidate(&lib_rs_path);
        }

        Ok(())
    }
}
