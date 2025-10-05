use crate::import_parser::ImportInfo;
use std::{
    collections::BTreeSet,
    path::Path,
};

pub struct ImportAnalysisResult {
    pub all_imported_items: BTreeSet<String>,
    pub glob_imports: usize,
    pub specific_imports: usize,
    pub import_types: std::collections::HashMap<String, Vec<String>>,
}

/// Analyze and categorize imports, building a map of imported items and their locations
pub fn analyze_imports(
    imports: &[ImportInfo],
    source_crate_name: &str,
    workspace_root: &Path,
) -> ImportAnalysisResult {
    let mut all_imported_items = BTreeSet::new();
    let mut glob_imports = 0;
    let mut specific_imports = 0;
    let mut import_types = std::collections::HashMap::new();

    let workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    for import in imports {
        if import.imported_items.contains(&"*".to_string()) {
            glob_imports += 1;
        } else {
            specific_imports += 1;
            for item in &import.imported_items {
                if item != "*" {
                    all_imported_items.insert(item.clone());

                    // Track which files import this item with relative paths and simplified imports
                    let canonical_file_path = import
                        .file_path
                        .canonicalize()
                        .unwrap_or_else(|_| import.file_path.clone());

                    let relative_path = canonical_file_path
                        .strip_prefix(&workspace_root)
                        .unwrap_or(&import.file_path);

                    // Make import path relative to source crate
                    let simplified_import = import
                        .import_path
                        .strip_prefix(&format!("{}::", source_crate_name))
                        .unwrap_or(&import.import_path);

                    import_types
                        .entry(item.clone())
                        .or_insert_with(Vec::new)
                        .push(format!(
                            "{}:{}",
                            relative_path.display(),
                            simplified_import
                        ));
                }
            }
        }
    }

    ImportAnalysisResult {
        all_imported_items,
        glob_imports,
        specific_imports,
        import_types,
    }
}

/// Analyze crate:: imports within a single crate
pub fn analyze_crate_imports(
    imports: &[ImportInfo],
    workspace_root: &Path,
) -> ImportAnalysisResult {
    let mut all_imported_items = BTreeSet::new();
    let mut glob_imports = 0;
    let mut specific_imports = 0;
    let mut import_types = std::collections::HashMap::new();

    let workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    for import in imports {
        if import.imported_items.contains(&"*".to_string()) {
            glob_imports += 1;
        } else {
            specific_imports += 1;
            for item in &import.imported_items {
                if item != "*" {
                    all_imported_items.insert(item.clone());

                    // Track which files import this item with relative paths and simplified imports
                    let canonical_file_path = import
                        .file_path
                        .canonicalize()
                        .unwrap_or_else(|_| import.file_path.clone());

                    let relative_path = canonical_file_path
                        .strip_prefix(&workspace_root)
                        .unwrap_or(&import.file_path);

                    // Make import path relative to crate (remove "crate::" prefix)
                    let simplified_import = import
                        .import_path
                        .strip_prefix("crate::")
                        .unwrap_or(&import.import_path);

                    import_types
                        .entry(item.clone())
                        .or_insert_with(Vec::new)
                        .push(format!(
                            "{}:{}",
                            relative_path.display(),
                            simplified_import
                        ));
                }
            }
        }
    }

    ImportAnalysisResult {
        all_imported_items,
        glob_imports,
        specific_imports,
        import_types,
    }
}

/// Print analysis summary for regular imports
pub fn print_import_analysis_summary(
    result: &ImportAnalysisResult,
    imports: &[ImportInfo],
    source_crate_name: &str,
) {
    println!("📊 Import Analysis Summary:");
    println!("  • Total imports found: {}", imports.len());
    println!(
        "  • Glob imports (use {}::*): {}",
        source_crate_name, result.glob_imports
    );
    println!("  • Specific imports: {}", result.specific_imports);
    println!(
        "  • Unique items imported: {}",
        result.all_imported_items.len()
    );

    if !result.all_imported_items.is_empty() {
        println!("\n🔍 Detected imported items from '{}':", source_crate_name);
        print_imported_items(&result.all_imported_items, &result.import_types);
        println!();
    } else if result.glob_imports > 0 {
        println!("\n💡 Note: Only glob imports (use {}::*) found. No specific items to re-export.", source_crate_name);
        println!("   This means the target crate is already using the most general import pattern.");
        println!();
    }
}

/// Print analysis summary for crate:: imports
pub fn print_crate_analysis_summary(
    result: &ImportAnalysisResult,
    imports: &[ImportInfo],
) {
    println!("📊 Import Analysis Summary:");
    println!("  • Total imports found: {}", imports.len());
    println!("  • Glob imports (use crate::*): {}", result.glob_imports);
    println!("  • Specific imports: {}", result.specific_imports);
    println!(
        "  • Unique items imported: {}",
        result.all_imported_items.len()
    );

    if !result.all_imported_items.is_empty() {
        println!("\n🔍 Detected crate:: imports:");
        print_imported_items(&result.all_imported_items, &result.import_types);
        println!();
    } else if result.glob_imports > 0 {
        println!("\n💡 Note: Only glob imports (use crate::*) found. No specific items to re-export.");
        println!("   This means the crate is already using the most general import pattern.");
        println!();
    }
}

/// Helper function to print imported items with their locations
fn print_imported_items(
    items: &BTreeSet<String>,
    import_types: &std::collections::HashMap<String, Vec<String>>,
) {
    for item in items {
        if let Some(files) = import_types.get(item) {
            println!("  • {}", item);
            for file_info in files.iter().take(3) {
                println!("      └─ {}", file_info);
            }
            if files.len() > 3 {
                println!("      └─ ... and {} more locations", files.len() - 3);
            }
        } else {
            println!("  • {}", item);
        }
    }
}
