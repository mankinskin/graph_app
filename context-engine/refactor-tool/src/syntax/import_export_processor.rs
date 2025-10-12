//! Unified API for parsing, generating, transforming, and merging import and export statements
//!
//! This module provides a high-level, consistent interface for all import/export operations
//! used throughout the refactor-tool crate. It consolidates the scattered functionality
//! from ImportParser, ExportAnalyzer, various replacement strategies, and merge operations
//! into a single, coherent API.

use crate::{
    analysis::crates::{CrateNames, CratePaths},
    analysis::exports::ExportAnalyzer,
    syntax::{
        parser::{ImportInfo, ImportParser},
        super_strategy::SuperNormalizationStrategy,
        transformer::{ImportReplacementStrategy, ReplacementAction},
        visitor::{merge_pub_uses, parse_existing_pub_uses},
    },
};
use anyhow::{Context as _, Result};
use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};
use syn::File;

/// Configuration and context for import/export processing operations
#[derive(Debug, Clone)]
pub struct ImportExportContext {
    /// The crates involved in the operation
    pub crate_names: CrateNames,
    /// Paths to the source and target crates
    pub crate_paths: CratePaths,
    /// Workspace root directory
    pub workspace_root: PathBuf,
    /// Whether to run in dry-run mode
    pub dry_run: bool,
    /// Whether to enable verbose output
    pub verbose: bool,
    /// Whether to normalize super:: imports to crate:: format
    pub normalize_super: bool,
    /// Whether to generate exports automatically
    pub generate_exports: bool,
}

impl ImportExportContext {
    pub fn new(
        crate_names: CrateNames,
        crate_paths: CratePaths,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            crate_names,
            crate_paths,
            workspace_root,
            dry_run: false,
            verbose: false,
            normalize_super: true,
            generate_exports: true,
        }
    }

    /// Configure dry-run mode
    pub fn with_dry_run(
        mut self,
        dry_run: bool,
    ) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Configure verbose output
    pub fn with_verbose(
        mut self,
        verbose: bool,
    ) -> Self {
        self.verbose = verbose;
        self
    }

    /// Configure super:: import normalization
    pub fn with_normalize_super(
        mut self,
        normalize_super: bool,
    ) -> Self {
        self.normalize_super = normalize_super;
        self
    }

    /// Configure export generation
    pub fn with_generate_exports(
        mut self,
        generate_exports: bool,
    ) -> Self {
        self.generate_exports = generate_exports;
        self
    }
}

/// Represents a parsed import tree with hierarchical structure
#[derive(Debug, Clone)]
pub struct ImportTree {
    /// Root-level imports (e.g., "use std::collections::HashMap;")
    pub simple_imports: Vec<ImportInfo>,
    /// Multi-item imports grouped by path (e.g., "use std::collections::{HashMap, HashSet};")
    pub grouped_imports: HashMap<String, Vec<String>>,
    /// Super imports that need normalization
    pub super_imports: Vec<ImportInfo>,
}

impl ImportTree {
    pub fn new() -> Self {
        Self {
            simple_imports: Vec::new(),
            grouped_imports: HashMap::new(),
            super_imports: Vec::new(),
        }
    }

    /// Merge another ImportTree into this one
    pub fn merge(
        &mut self,
        other: ImportTree,
    ) {
        self.simple_imports.extend(other.simple_imports);

        for (path, items) in other.grouped_imports {
            self.grouped_imports.entry(path).or_default().extend(items);
        }

        self.super_imports.extend(other.super_imports);
    }

    /// Get total number of imports
    pub fn count(&self) -> usize {
        self.simple_imports.len()
            + self
                .grouped_imports
                .values()
                .map(|v| v.len())
                .sum::<usize>()
            + self.super_imports.len()
    }

    /// Convert to flat list of ImportInfo for compatibility
    pub fn to_flat_imports(&self) -> Vec<ImportInfo> {
        let mut imports = Vec::new();
        imports.extend(self.simple_imports.clone());
        imports.extend(self.super_imports.clone());

        // Convert grouped imports back to ImportInfo
        for (base_path, items) in &self.grouped_imports {
            for item in items {
                imports.push(ImportInfo {
                    file_path: PathBuf::new(), // Will be set by caller if needed
                    import_path: format!("{}::{}", base_path, item),
                    line_number: 0,
                    imported_items: vec![item.clone()],
                });
            }
        }

        imports
    }
}

/// Results of export analysis operations  
#[derive(Debug, Clone)]
pub struct ExportAnalysis {
    /// Items already exported from the crate
    pub existing_exports: BTreeSet<String>,
    /// Items that need to be added as exports
    pub items_to_export: BTreeSet<String>,
    /// Existing pub use statements that can be merged
    pub existing_pub_uses: BTreeSet<String>,
    /// Generated pub use statements
    pub generated_statements: Vec<String>,
}

/// Path transformation operations for import processing
pub struct PathSegmentProcessor;

impl PathSegmentProcessor {
    /// Normalize super:: imports to crate:: format
    pub fn normalize_super_imports(
        import: &mut ImportInfo,
        crate_root: &Path,
    ) -> Result<bool> {
        use crate::core::path::is_super_import;

        if !is_super_import(&import.import_path) {
            return Ok(false); // No changes needed
        }

        import.normalize_super_imports(crate_root)?;
        Ok(true) // Changes made
    }

    /// Normalize crate name to crate:: format
    pub fn normalize_crate_name(
        import: &mut ImportInfo,
        crate_name: &str,
    ) -> bool {
        let normalized_name = crate_name.replace('-', "_");

        if import
            .import_path
            .starts_with(&format!("{}::", normalized_name))
        {
            import.normalize_to_crate_format(&normalized_name);
            true // Changes made
        } else {
            false // No changes needed
        }
    }

    /// Resolve relative paths to absolute crate paths
    pub fn resolve_path_segments(
        base_path: &str,
        segments: &[String],
    ) -> String {
        if segments.is_empty() {
            return base_path.to_string();
        }

        if base_path.is_empty() {
            segments.join("::")
        } else {
            format!("{}::{}", base_path, segments.join("::"))
        }
    }
}

/// Unified processor for import tree operations
pub struct ImportTreeProcessor {
    context: ImportExportContext,
}

impl ImportTreeProcessor {
    pub fn new(context: ImportExportContext) -> Self {
        Self { context }
    }

    /// Parse all imports from the target crate into a structured tree
    pub fn parse_imports(&self) -> Result<ImportTree> {
        let mut tree = ImportTree::new();

        match &self.context.crate_names {
            CrateNames::SelfCrate { crate_name } => {
                let source_path = self.context.crate_paths.source_path();

                // Parse crate:: imports
                let crate_parser = ImportParser::new("crate");
                let crate_imports = crate_parser
                    .find_imports_in_crate(source_path)
                    .context("Failed to parse crate:: imports")?;

                // Parse explicit crate name imports
                let explicit_parser = ImportParser::new(crate_name);
                let explicit_imports = explicit_parser
                    .find_imports_in_crate(source_path)
                    .context("Failed to parse explicit crate name imports")?;

                // Parse super:: imports if normalization is enabled
                let super_imports = if self.context.normalize_super {
                    ImportParser::find_super_imports_in_crate(source_path)
                        .context("Failed to parse super:: imports")?
                } else {
                    Vec::new()
                };

                // Categorize imports into the tree structure
                self.categorize_imports(
                    &mut tree,
                    crate_imports,
                    ImportCategory::Simple,
                )?;
                self.categorize_imports(
                    &mut tree,
                    explicit_imports,
                    ImportCategory::Simple,
                )?;
                self.categorize_imports(
                    &mut tree,
                    super_imports,
                    ImportCategory::Super,
                )?;
            },
            CrateNames::CrossCrate { source_crate, .. } => {
                let target_path = self.context.crate_paths.target_path();
                let import_parser = ImportParser::new(source_crate);
                let imports = import_parser
                    .find_imports_in_crate(target_path)
                    .context("Failed to parse cross-crate imports")?;

                self.categorize_imports(
                    &mut tree,
                    imports,
                    ImportCategory::Simple,
                )?;
            },
        }

        Ok(tree)
    }

    /// Normalize all imports in the tree according to context settings
    pub fn normalize_imports(
        &self,
        tree: &mut ImportTree,
    ) -> Result<usize> {
        let mut changes_made = 0;

        // Normalize super:: imports if enabled
        if self.context.normalize_super {
            let source_path = self.context.crate_paths.source_path();

            for import in &mut tree.super_imports {
                if PathSegmentProcessor::normalize_super_imports(
                    import,
                    source_path,
                )? {
                    changes_made += 1;
                }
            }

            // Move normalized super imports to simple imports
            let normalized_super: Vec<_> =
                tree.super_imports.drain(..).collect();
            tree.simple_imports.extend(normalized_super);
        }

        // Normalize explicit crate names to crate:: format
        if let CrateNames::SelfCrate { crate_name } = &self.context.crate_names
        {
            for import in &mut tree.simple_imports {
                if PathSegmentProcessor::normalize_crate_name(
                    import, crate_name,
                ) {
                    changes_made += 1;
                }
            }
        }

        Ok(changes_made)
    }

    /// Merge import tree with existing exports to avoid conflicts
    pub fn merge_with_exports(
        &self,
        tree: &ImportTree,
    ) -> Result<ExportAnalysis> {
        let source_path = self.context.crate_paths.source_path();
        let lib_rs_path = source_path.join("src").join("lib.rs");

        // Parse existing exports
        let content = std::fs::read_to_string(&lib_rs_path)
            .context("Failed to read lib.rs for export analysis")?;
        let syntax_tree: File = syn::parse_file(&content)
            .context("Failed to parse lib.rs syntax tree")?;

        // Analyze existing exports
        let export_analyzer = ExportAnalyzer {
            verbose: self.context.verbose,
        };
        let existing_exports =
            export_analyzer.collect_existing_exports(&syntax_tree, source_path);

        // Parse existing pub use statements
        let (existing_pub_uses, _) = parse_existing_pub_uses(&syntax_tree);

        // Determine what needs to be exported
        let flat_imports = tree.to_flat_imports();
        let mut items_to_export = BTreeSet::new();

        for import in &flat_imports {
            // Extract the final item from the import path
            let final_item =
                import.import_path.split("::").last().unwrap_or("");
            if !existing_exports.contains(final_item) && !final_item.is_empty()
            {
                items_to_export.insert(import.import_path.clone());
            }
        }

        // Generate merged pub use statements
        let crate_name = match &self.context.crate_names {
            CrateNames::SelfCrate { crate_name } => crate_name,
            CrateNames::CrossCrate { source_crate, .. } => source_crate,
        };

        let generated_statements =
            if self.context.generate_exports && !items_to_export.is_empty() {
                merge_pub_uses(
                    existing_pub_uses.clone(),
                    &items_to_export,
                    crate_name,
                )
            } else {
                Vec::new()
            };

        Ok(ExportAnalysis {
            existing_exports,
            items_to_export,
            existing_pub_uses,
            generated_statements,
        })
    }

    // Helper method to categorize imports into tree structure
    fn categorize_imports(
        &self,
        tree: &mut ImportTree,
        imports: Vec<ImportInfo>,
        category: ImportCategory,
    ) -> Result<()> {
        match category {
            ImportCategory::Simple => {
                tree.simple_imports.extend(imports);
            },
            ImportCategory::Super => {
                tree.super_imports.extend(imports);
            },
            ImportCategory::Grouped => {
                // Group imports by base path for multi-item imports
                for import in imports {
                    if import.imported_items.len() > 1 {
                        tree.grouped_imports
                            .entry(import.import_path.clone())
                            .or_default()
                            .extend(import.imported_items);
                    } else {
                        tree.simple_imports.push(import);
                    }
                }
            },
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum ImportCategory {
    Simple,  // Regular imports
    Super,   // Super:: imports needing normalization
    Grouped, // Multi-item imports that should be grouped
}

/// Unified strategy-based processor for import replacements
pub struct ImportReplacementProcessor {
    context: ImportExportContext,
}

impl ImportReplacementProcessor {
    pub fn new(context: ImportExportContext) -> Self {
        Self { context }
    }

    /// Apply replacement strategy to all imports in the tree
    pub fn apply_strategy<S: ImportReplacementStrategy + ?Sized>(
        &self,
        tree: &ImportTree,
        strategy: &S,
    ) -> Result<HashMap<PathBuf, Vec<ReplacementAction>>> {
        let flat_imports = tree.to_flat_imports();
        let mut results = HashMap::new();

        // Group imports by file path
        let mut imports_by_file: HashMap<PathBuf, Vec<ImportInfo>> =
            HashMap::new();
        for import in flat_imports {
            imports_by_file
                .entry(import.file_path.clone())
                .or_default()
                .push(import);
        }

        // Apply strategy to each file's imports
        for (file_path, file_imports) in imports_by_file {
            let mut file_actions = Vec::new();

            for import in &file_imports {
                let action = if let Some(replacement) =
                    strategy.create_replacement(import)
                {
                    ReplacementAction::Replaced {
                        from: format!("use {};", import.import_path),
                        to: replacement,
                    }
                } else if strategy.should_remove_import(import) {
                    ReplacementAction::Removed {
                        original: format!("use {};", import.import_path),
                    }
                } else {
                    ReplacementAction::NotFound {
                        searched_for: import.import_path.clone(),
                    }
                };

                file_actions.push(action);
            }

            results.insert(file_path, file_actions);
        }

        Ok(results)
    }

    /// Get appropriate strategy based on context
    pub fn get_strategy(&self) -> Box<dyn ImportReplacementStrategy> {
        match &self.context.crate_names {
            CrateNames::SelfCrate { .. } => Box::new(
                crate::syntax::transformer::SelfCrateReplacementStrategy,
            ),
            CrateNames::CrossCrate { source_crate, .. } => Box::new(
                crate::syntax::transformer::CrossCrateReplacementStrategy {
                    crate_names: self.context.crate_names.clone(),
                },
            ),
        }
    }

    /// Get super normalization strategy
    pub fn get_super_strategy(&self) -> SuperNormalizationStrategy {
        SuperNormalizationStrategy {
            crate_root: self.context.crate_paths.source_path().to_path_buf(),
        }
    }
}

/// Main unified processor that orchestrates all import/export operations
pub struct ImportExportProcessor {
    context: ImportExportContext,
    tree_processor: ImportTreeProcessor,
    replacement_processor: ImportReplacementProcessor,
}

impl ImportExportProcessor {
    pub fn new(context: ImportExportContext) -> Self {
        let tree_processor = ImportTreeProcessor::new(context.clone());
        let replacement_processor =
            ImportReplacementProcessor::new(context.clone());

        Self {
            context,
            tree_processor,
            replacement_processor,
        }
    }

    /// Execute the complete import/export processing pipeline
    pub fn process(&self) -> Result<ProcessingResults> {
        // Step 1: Parse all imports into structured tree
        let mut import_tree = self
            .tree_processor
            .parse_imports()
            .context("Failed to parse imports")?;

        if self.context.verbose {
            println!("📊 Parsed {} imports total", import_tree.count());
        }

        // Step 2: Normalize imports according to context settings
        let normalization_changes = self
            .tree_processor
            .normalize_imports(&mut import_tree)
            .context("Failed to normalize imports")?;

        if self.context.verbose && normalization_changes > 0 {
            println!("🔄 Normalized {} import paths", normalization_changes);
        }

        // Step 3: Analyze exports and generate statements if needed
        let export_analysis = if self.context.generate_exports {
            Some(
                self.tree_processor
                    .merge_with_exports(&import_tree)
                    .context("Failed to analyze exports")?,
            )
        } else {
            None
        };

        // Step 4: Apply replacement strategy to transform imports
        let strategy = self.replacement_processor.get_strategy();
        let replacement_results = self
            .replacement_processor
            .apply_strategy(&import_tree, strategy.as_ref())
            .context("Failed to apply replacement strategy")?;

        // Step 5: Handle super:: import normalization if needed
        let super_results = if self.context.normalize_super {
            let super_strategy =
                self.replacement_processor.get_super_strategy();
            Some(
                self.replacement_processor
                    .apply_strategy(&import_tree, &super_strategy)
                    .context("Failed to apply super normalization")?,
            )
        } else {
            None
        };

        Ok(ProcessingResults {
            import_tree,
            export_analysis,
            replacement_results,
            super_results,
            normalization_changes,
        })
    }
}

/// Results of the complete import/export processing pipeline
#[derive(Debug)]
pub struct ProcessingResults {
    /// The processed import tree
    pub import_tree: ImportTree,
    /// Export analysis results (if export generation was enabled)
    pub export_analysis: Option<ExportAnalysis>,
    /// Results of applying replacement strategy
    pub replacement_results: HashMap<PathBuf, Vec<ReplacementAction>>,
    /// Results of super:: import normalization (if enabled)
    pub super_results: Option<HashMap<PathBuf, Vec<ReplacementAction>>>,
    /// Number of imports that were normalized
    pub normalization_changes: usize,
}

impl ProcessingResults {
    /// Get total number of imports processed
    pub fn total_imports(&self) -> usize {
        self.import_tree.count()
    }

    /// Get total number of replacement actions
    pub fn total_replacements(&self) -> usize {
        self.replacement_results
            .values()
            .map(|actions| actions.len())
            .sum::<usize>()
            + self
                .super_results
                .as_ref()
                .map(|results| {
                    results.values().map(|actions| actions.len()).sum::<usize>()
                })
                .unwrap_or(0)
    }

    /// Get total number of export statements generated
    pub fn total_exports_generated(&self) -> usize {
        self.export_analysis
            .as_ref()
            .map(|analysis| analysis.generated_statements.len())
            .unwrap_or(0)
    }
}
