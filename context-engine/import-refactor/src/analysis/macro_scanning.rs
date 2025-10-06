use crate::syntax::item_info::has_macro_export_attribute;
use anyhow::{Context, Result};
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
};

/// Scan all .rs files in the source crate for exported macros
pub fn scan_crate_for_exported_macros(
    source_crate_path: &Path,
    verbose: bool,
) -> Result<BTreeSet<String>> {
    let mut exported_macros = BTreeSet::new();
    let src_dir = source_crate_path.join("src");

    if !src_dir.exists() {
        return Ok(exported_macros);
    }

    scan_directory_for_macros(&src_dir, &mut exported_macros, verbose)?;

    if verbose {
        println!(
            "🔍 Found {} exported macros across source crate: {:?}",
            exported_macros.len(),
            exported_macros
        );
    }

    Ok(exported_macros)
}

/// Recursively scan a directory for .rs files and extract exported macros
pub fn scan_directory_for_macros(
    dir_path: &Path,
    exported_macros: &mut BTreeSet<String>,
    verbose: bool,
) -> Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively scan subdirectories
            scan_directory_for_macros(&path, exported_macros, verbose)?;
        } else if let Some(extension) = path.extension() {
            if extension == "rs" {
                scan_file_for_exported_macros(&path, exported_macros, verbose)?;
            }
        }
    }

    Ok(())
}

/// Scan a single .rs file for exported macros
pub fn scan_file_for_exported_macros(
    file_path: &Path,
    exported_macros: &mut BTreeSet<String>,
    verbose: bool,
) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

    let syntax_tree: syn::File = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {:?}", file_path))?;

    for item in &syntax_tree.items {
        if let syn::Item::Macro(macro_item) = item {
            if has_macro_export_attribute(&macro_item.attrs) {
                if let Some(ident) = &macro_item.ident {
                    exported_macros.insert(ident.to_string());
                    if verbose {
                        println!(
                            "  📝 Found exported macro '{}' in {}",
                            ident,
                            file_path.display()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}