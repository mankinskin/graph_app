use refactor_tool::{
    CrateNames, CratePaths, ImportExportContext, ImportExportContextExt,
    ImportExportProcessor, ImportExportUtils, ProcessingResultsExt,
};
/// Example demonstrating the unified import/export processing API
///
/// This example shows how to use the new unified API to process imports
/// and exports in a consistent way across different refactoring scenarios.
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let workspace_root = Path::new(".");

    // Example 1: Quick cross-crate refactoring
    println!("=== Quick Cross-Crate Refactoring ===");

    let crate_paths = CratePaths::CrossCrate {
        source_crate_path: Path::new("./old_crate").to_path_buf(),
        target_crate_path: Path::new("./new_crate").to_path_buf(),
    };

    let results = ImportExportUtils::process_cross_crate(
        "old_crate",
        "new_crate",
        crate_paths,
        workspace_root,
        true, // dry_run
        true, // verbose
    )?;

    results.print_summary(true);

    // Example 2: Self-crate refactoring with custom context
    println!("\n=== Self-Crate Refactoring with Custom Context ===");
    let crate_names = CrateNames::SelfCrate {
        crate_name: "my_crate".to_string(),
    };

    let crate_paths = CratePaths::SelfCrate {
        crate_path: Path::new("./my_crate").to_path_buf(),
    };

    let context = ImportExportContext::new(
        crate_names,
        crate_paths,
        workspace_root.to_path_buf(),
    )
    .with_dry_run(true)
    .with_verbose(true)
    .for_self_crate();

    let processor = ImportExportProcessor::new(context);
    let results = processor.process()?;

    println!(
        "Found {} simple imports",
        results.import_tree.simple_imports.len()
    );
    println!(
        "Found {} grouped imports",
        results.import_tree.grouped_imports.len()
    );
    println!(
        "Found {} super imports",
        results.import_tree.super_imports.len()
    );

    // Example 3: Just normalize super:: imports
    println!("\n=== Super Import Normalization ===");
    let results = ImportExportUtils::normalize_super_imports(
        "example_crate",
        Path::new("./example_crate"),
        workspace_root,
        true, // dry_run
        true, // verbose
    )?;

    if results.has_changes() {
        println!("Super imports found and would be normalized");
        results.print_summary(true);
    } else {
        println!("No super imports found to normalize");
    }

    Ok(())
}
