use crate::import_parser::ImportInfo;
use crate::utils::common::format_relative_path;
use std::{
    collections::BTreeSet,
    path::Path,
};

/// Context for import analysis - defines how imports should be processed
#[derive(Debug, Clone)]
pub enum ImportContext {
    /// Cross-crate imports: source_crate::module::Item
    CrossCrate { 
        source_crate_name: String 
    },
    /// Self-crate imports: crate::module::Item  
    SelfCrate,
}

impl ImportContext {
    /// Get the prefix to strip from import paths
    fn get_prefix_to_strip(&self) -> String {
        match self {
            ImportContext::CrossCrate { source_crate_name } => 
                format!("{}::", source_crate_name),
            ImportContext::SelfCrate => 
                "crate::".to_string(),
        }
    }
    
    /// Get label for summary display
    pub fn get_summary_label(&self) -> String {
        match self {
            ImportContext::CrossCrate { source_crate_name } => source_crate_name.clone(),
            ImportContext::SelfCrate => "crate".to_string(),
        }
    }
    
    /// Get glob import pattern description
    pub fn get_glob_pattern_description(&self) -> String {
        match self {
            ImportContext::CrossCrate { source_crate_name } => 
                format!("use {}::*", source_crate_name),
            ImportContext::SelfCrate => 
                "use crate::*".to_string(),
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
    context: ImportContext,
    workspace_root: &Path,
) -> ImportAnalysisResult {
    let mut all_imported_items = BTreeSet::new();
    let mut glob_imports = 0;
    let mut specific_imports = 0;
    let mut import_types = std::collections::HashMap::new();

    let workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    let prefix_to_strip = context.get_prefix_to_strip();

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

                    let relative_path = format_relative_path(&canonical_file_path, &workspace_root);

                    // Context-specific prefix stripping (THE ONLY DIFFERENCE!)
                    let simplified_import = import
                        .import_path
                        .strip_prefix(&prefix_to_strip)
                        .unwrap_or(&import.import_path);

                    import_types
                        .entry(item.clone())
                        .or_insert_with(Vec::new)
                        .push(format!(
                            "{}:{}",
                            relative_path,
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

/// Unified summary printing for any import context
pub fn print_analysis_summary(
    result: &ImportAnalysisResult,
    imports: &[ImportInfo],
    context: &ImportContext,
) {
    let _summary_label = context.get_summary_label();
    let glob_pattern = context.get_glob_pattern_description();
    
    println!("📊 Import Analysis Summary:");
    println!("  • Total imports found: {}", imports.len());
    println!("  • Glob imports ({}): {}", glob_pattern, result.glob_imports);
    println!("  • Specific imports: {}", result.specific_imports);
    println!("  • Unique items imported: {}", result.all_imported_items.len());

    if !result.all_imported_items.is_empty() {
        match context {
            ImportContext::CrossCrate { source_crate_name } => {
                println!("\n🔍 Detected imported items from '{}':", source_crate_name);
            }
            ImportContext::SelfCrate => {
                println!("\n🔍 Detected crate:: imports:");
            }
        }
        print_imported_items(&result.all_imported_items, &result.import_types);
        println!();
    } else if result.glob_imports > 0 {
        match context {
            ImportContext::CrossCrate { source_crate_name } => {
                println!("\n💡 Note: Only glob imports (use {}::*) found. No specific items to re-export.", source_crate_name);
            }
            ImportContext::SelfCrate => {
                println!("\n💡 Note: Only glob imports (use crate::*) found. No specific items to re-export.");
            }
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
