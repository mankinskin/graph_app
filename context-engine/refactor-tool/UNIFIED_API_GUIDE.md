# Unified Import/Export Processing API

This document describes the new unified API for parsing, generating, transforming, and merging import and export statements in the refactor-tool crate.

## Overview

The unified API consolidates the previously scattered functionality across `ImportParser`, `ExportAnalyzer`, `ImportReplacementStrategy`, and various merge operations into a coherent, easy-to-use interface.

## Key Components

### 1. ImportExportProcessor

The main processor that orchestrates all import/export operations:

```rust
use refactor_tool::{
    ImportExportProcessor, ImportExportContext, CrateNames, CratePaths
};

// Create context
let crate_names = CrateNames::SelfCrate {
    crate_name: "my_crate".to_string(),
};
let crate_paths = CratePaths::SelfCrate {
    crate_path: "/path/to/crate".into(),
};
let context = ImportExportContext::new(crate_names, crate_paths, "/workspace".into())
    .with_verbose(true)
    .with_normalize_super(true);

// Process imports and exports
let processor = ImportExportProcessor::new(context);
let results = processor.process()?;

// Print summary
results.print_summary(true);
```

### 2. ImportExportUtils - Quick Start Functions

For common use cases, use the utility functions:

```rust
use refactor_tool::ImportExportUtils;

// Cross-crate refactoring
let results = ImportExportUtils::process_cross_crate(
    "source_crate",
    "target_crate", 
    crate_paths,
    workspace_root,
    false, // dry_run
    true,  // verbose
)?;

// Self-crate refactoring  
let results = ImportExportUtils::process_self_crate(
    "my_crate",
    crate_path,
    workspace_root,
    false, // dry_run
    true,  // verbose
)?;

// Just normalize super:: imports
let results = ImportExportUtils::normalize_super_imports(
    "my_crate",
    crate_path,
    workspace_root,
    false, // dry_run
    true,  // verbose
)?;
```

### 3. Extension Traits for Ergonomics

The API includes extension traits for better usability:

```rust
use refactor_tool::{CrateNamesExt, ImportExportContextExt, ProcessingResultsExt};

// Create contexts easily
let context = crate_names.to_verbose_context(crate_paths, workspace_root);

// Configure for specific scenarios
let context = context.for_cross_crate();  // or .for_self_crate()

// Analyze results
if results.has_changes() {
    results.print_summary(true);
    let changes = results.describe_changes();
}
```

## Migration from Legacy API

### Using the Unified Adapter

For gradual migration, use the `UnifiedApiAdapter`:

```rust
use refactor_tool::{RefactorConfig, UnifiedApiAdapter};

// Existing RefactorConfig
let config = RefactorConfig { /* ... */ };

// Use unified processor with same interface
let result = UnifiedApiAdapter::execute_with_unified_processor(config);

// Compare both methods during migration
let comparison = UnifiedApiAdapter::compare_processing_methods(config);
comparison.print_comparison();
```

### Converting RefactorConfig to ImportExportContext

```rust
use refactor_tool::{RefactorConfigExt, ImportExportContextExt};

let config = RefactorConfig { /* ... */ };
let crate_paths = /* ... */;

let context = config.to_import_export_context(crate_paths)
    .for_cross_crate();  // Apply scenario-specific settings
```

## Architecture Benefits

### 1. Unified Interface

All import/export operations now go through a single, consistent API:

- **Parsing**: `ImportTreeProcessor::parse_imports()`
- **Normalization**: `PathSegmentProcessor::normalize_*()` 
- **Analysis**: `ImportTreeProcessor::merge_with_exports()`
- **Replacement**: `ImportReplacementProcessor::apply_strategy()`

### 2. Structured Data Flow

The new `ImportTree` structure provides better organization:

```rust
pub struct ImportTree {
    pub simple_imports: Vec<ImportInfo>,           // Single imports
    pub grouped_imports: HashMap<String, Vec<String>>,  // Multi-item imports
    pub super_imports: Vec<ImportInfo>,            // Super:: imports
}
```

### 3. Context-Driven Configuration

All operations are configured through `ImportExportContext`:

```rust
pub struct ImportExportContext {
    pub crate_names: CrateNames,
    pub crate_paths: CratePaths, 
    pub workspace_root: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
    pub normalize_super: bool,
    pub generate_exports: bool,
}
```

### 4. Comprehensive Results

Processing results provide detailed information and analysis:

```rust
pub struct ProcessingResults {
    pub import_tree: ImportTree,
    pub export_analysis: Option<ExportAnalysis>,
    pub replacement_results: HashMap<PathBuf, Vec<ReplacementAction>>,
    pub super_results: Option<HashMap<PathBuf, Vec<ReplacementAction>>>,
    pub normalization_changes: usize,
}
```

## Strategy Pattern Integration

The unified API maintains compatibility with existing strategies:

```rust
// Get appropriate strategy for context
let strategy = replacement_processor.get_strategy();

// Apply to import tree
let results = replacement_processor.apply_strategy(&import_tree, strategy.as_ref())?;

// Handle super:: normalization separately if needed
let super_strategy = replacement_processor.get_super_strategy();
let super_results = replacement_processor.apply_strategy(&import_tree, &super_strategy)?;
```

## Analysis and Optimization

The API provides built-in analysis capabilities:

```rust
use refactor_tool::ImportTreeExt;

// Get statistics about the import tree
let stats = import_tree.stats();
println!("Simple imports: {}", stats.simple_imports);
println!("Grouped imports: {}", stats.grouped_imports);
println!("Unique modules: {}", stats.unique_modules);

// Find optimization opportunities
let optimizations = import_tree.find_optimizations();
for opt in optimizations {
    match opt {
        ImportOptimization::GroupableImports { module_path, import_count, potential_grouping } => {
            println!("Can group {} imports from {}: {}", import_count, module_path, potential_grouping);
        }
        ImportOptimization::UnnormalizedSuperImports { count } => {
            println!("Found {} super:: imports that could be normalized", count);
        }
        _ => {}
    }
}
```

## Testing and Validation

The unified API includes comprehensive testing utilities:

```rust
// Compare old vs new processing
let comparison = UnifiedApiAdapter::compare_processing_methods(config);
assert!(comparison.are_equivalent());

// Validate processing results
assert!(results.has_changes());
assert_eq!(results.total_imports(), expected_count);
assert_eq!(results.total_exports_generated(), expected_exports);
```

## Error Handling

All operations use `anyhow::Result` for consistent error handling:

```rust
use anyhow::{Result, Context};

let results = processor.process()
    .context("Failed to process imports and exports")?;
    
// Individual operations also provide context
let import_tree = tree_processor.parse_imports()
    .context("Failed to parse imports from crate")?;
```

## Performance Considerations

The unified API is designed for efficiency:

1. **Structured parsing** reduces redundant AST traversals
2. **Batched operations** group related changes  
3. **Lazy evaluation** only computes what's needed
4. **Caching** of parsed syntax trees and analysis results

## Future Extensions

The unified API is designed for extensibility:

- **Plugin system** for custom import transformations
- **Rule-based optimization** engine
- **Integration** with external tools (rustfmt, clippy)
- **Incremental processing** for large codebases
- **Parallel processing** of independent files

## Examples

See the `examples/` directory for complete working examples:

- `examples/cross_crate_refactor.rs` - Cross-crate import refactoring
- `examples/self_crate_cleanup.rs` - Self-crate import organization  
- `examples/super_normalization.rs` - Super:: import normalization
- `examples/export_generation.rs` - Automatic export generation
- `examples/migration_guide.rs` - Migrating from legacy API