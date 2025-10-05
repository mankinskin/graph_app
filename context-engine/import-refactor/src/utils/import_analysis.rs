use crate::{
    crate_analyzer::CrateNames,
    import_parser::ImportInfo,
    utils::common::format_relative_path,
};
use std::{
    collections::BTreeSet,
    path::Path,
};

impl CrateNames {
    /// Get the prefix to strip from import paths
    pub fn get_prefixes_to_strip(&self) -> Vec<String> {
        match self {
            CrateNames::CrossRefactor { source_crate, .. } =>
                vec![format!("{}::", source_crate)],
            CrateNames::SelfRefactor { crate_name } =>
                vec![format!("{}::", crate_name), "crate::".to_string()],
        }
    }

    /// Get label for summary display
    pub fn get_summary_label(&self) -> String {
        match self {
            CrateNames::CrossRefactor { source_crate, .. } =>
                source_crate.clone(),
            CrateNames::SelfRefactor { .. } => "crate".to_string(),
        }
    }

    /// Get glob import pattern description
    pub fn get_glob_pattern_description(&self) -> String {
        match self {
            CrateNames::CrossRefactor { source_crate, .. } =>
                format!("use {}::*", source_crate),
            CrateNames::SelfRefactor { .. } => "use crate::*".to_string(),
        }
    }
}

pub struct ImportAnalysisResult {
    pub all_imported_items: BTreeSet<String>,
    pub glob_imports: usize,
    pub specific_imports: usize,
    pub import_types: std::collections::HashMap<String, Vec<String>>,
}

/// Unified import analysis supporting multiple import contexts
pub fn analyze_imports(
    imports: &[ImportInfo],
    crate_names: &CrateNames,
    workspace_root: &Path,
) -> ImportAnalysisResult {
    let mut all_imported_items = BTreeSet::new();
    let mut glob_imports = 0;
    let mut specific_imports = 0;
    let mut import_types = std::collections::HashMap::new();

    let workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    let prefixes_to_strip = crate_names.get_prefixes_to_strip();

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

                    let relative_path = format_relative_path(
                        &canonical_file_path,
                        &workspace_root,
                    );

                    // Context-specific prefix stripping (THE ONLY DIFFERENCE!)

                    let simplified_import = prefixes_to_strip
                        .iter()
                        .find_map(|prefix| {
                            import.import_path.strip_prefix(prefix)
                        })
                        .unwrap_or(&import.import_path);

                    import_types
                        .entry(item.clone())
                        .or_insert_with(Vec::new)
                        .push(format!(
                            "{}:{}",
                            relative_path, simplified_import
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

/// Unified summary printing for any import context
pub fn print_analysis_summary(
    result: &ImportAnalysisResult,
    imports: &[ImportInfo],
    crate_names: &CrateNames,
) {
    let _summary_label = crate_names.get_summary_label();
    let glob_pattern = crate_names.get_glob_pattern_description();

    println!("📊 Import Analysis Summary:");
    println!("  • Total imports found: {}", imports.len());
    println!(
        "  • Glob imports ({}): {}",
        glob_pattern, result.glob_imports
    );
    println!("  • Specific imports: {}", result.specific_imports);
    println!(
        "  • Unique items imported: {}",
        result.all_imported_items.len()
    );

    if !result.all_imported_items.is_empty() {
        match crate_names {
            CrateNames::CrossRefactor { source_crate, .. } => {
                println!(
                    "\n🔍 Detected imported items from '{}':",
                    source_crate
                );
            },
            CrateNames::SelfRefactor { .. } => {
                println!("\n🔍 Detected crate:: imports:");
            },
        }
        print_imported_items(&result.all_imported_items, &result.import_types);
        println!();
    } else if result.glob_imports > 0 {
        match crate_names {
            CrateNames::CrossRefactor { source_crate, .. } => {
                println!("\n💡 Note: Only glob imports (use {}::*) found. No specific items to re-export.", source_crate);
            },
            CrateNames::SelfRefactor { .. } => {
                println!("\n💡 Note: Only glob imports (use crate::*) found. No specific items to re-export.");
            },
        }
        println!("   This means the target crate is already using the most general import pattern.");
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
