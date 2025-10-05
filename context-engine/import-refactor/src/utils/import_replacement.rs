use crate::{
    import_parser::ImportInfo,
    utils::file_operations::{
        get_relative_path_for_display,
        write_file,
    },
};
use anyhow::{
    Context,
    Result,
};
use std::{
    collections::HashMap,
    fs,
    path::{
        Path,
        PathBuf,
    },
};

/// Replace imports in target crate files
pub fn replace_target_imports(
    imports: Vec<ImportInfo>,
    source_crate_name: &str,
    workspace_root: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Group imports by file
    let mut imports_by_file: HashMap<PathBuf, Vec<ImportInfo>> = HashMap::new();

    for import in imports {
        imports_by_file
            .entry(import.file_path.clone())
            .or_default()
            .push(import);
    }

    // Process each file
    for (file_path, file_imports) in &imports_by_file {
        replace_imports_in_file(
            file_path,
            file_imports.clone(),
            source_crate_name,
            workspace_root,
            dry_run,
            verbose,
        )?;
    }

    Ok(())
}

/// Replace crate:: imports within the same crate files (for self-refactor mode)
pub fn replace_crate_imports(
    imports: Vec<ImportInfo>,
    workspace_root: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Group imports by file
    let mut imports_by_file: HashMap<PathBuf, Vec<ImportInfo>> = HashMap::new();

    for import in imports {
        imports_by_file
            .entry(import.file_path.clone())
            .or_default()
            .push(import);
    }

    for (file_path, file_imports) in imports_by_file {
        replace_crate_imports_in_file(
            &file_path,
            file_imports,
            workspace_root,
            dry_run,
            verbose,
        )?;
    }

    Ok(())
}

/// Replace imports in a specific file with glob imports
fn replace_imports_in_file(
    file_path: &Path,
    imports: Vec<ImportInfo>,
    source_crate_name: &str,
    workspace_root: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let original_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let mut new_content = original_content.clone();
    let mut replacements_made = 0;

    // Sort imports by line number in reverse order to avoid offset issues
    let mut sorted_imports = imports;
    sorted_imports.sort_by(|a, b| b.line_number.cmp(&a.line_number));

    for import in sorted_imports {
        // Look for the import statement in the content
        if let Some(import_start) =
            new_content.find(&format!("use {};", import.import_path))
        {
            // Find the end of the line
            let import_end = new_content[import_start..]
                .find('\n')
                .map(|pos| import_start + pos + 1)
                .unwrap_or(new_content.len());

            // Replace with the new import
            let replacement = format!("use {}::*;", source_crate_name);
            new_content.replace_range(
                import_start..import_end,
                &format!("{}\n", replacement),
            );
            replacements_made += 1;

            if verbose {
                println!(
                    "  Replaced: use {}; -> {}",
                    import.import_path, replacement
                );
            }
        } else {
            // Try to find a more general pattern
            let patterns = [
                format!("use {}", import.import_path),
                format!("use {}::", source_crate_name),
            ];

            let mut found = false;
            for pattern in &patterns {
                if let Some(pattern_start) = new_content.find(pattern) {
                    // Find the semicolon that ends this use statement
                    if let Some(semicolon_pos) =
                        new_content[pattern_start..].find(';')
                    {
                        let use_end = pattern_start + semicolon_pos + 1;

                        // Find the start of the line
                        let line_start = new_content[..pattern_start]
                            .rfind('\n')
                            .map(|pos| pos + 1)
                            .unwrap_or(0);

                        // Replace the entire use statement
                        let replacement =
                            format!("use {}::*;", source_crate_name);
                        let full_replacement = format!(
                            "{}{}",
                            &new_content[line_start..pattern_start],
                            replacement
                        );

                        new_content.replace_range(
                            line_start..use_end,
                            &full_replacement,
                        );
                        replacements_made += 1;
                        found = true;

                        if verbose {
                            println!(
                                "  Replaced pattern: {} -> {}",
                                pattern, replacement
                            );
                        }
                        break;
                    }
                }
            }

            if !found && verbose {
                println!(
                    "  Warning: Could not find import to replace: {}",
                    import.import_path
                );
            }
        }
    }

    if replacements_made > 0 {
        if verbose {
            let relative_path =
                get_relative_path_for_display(file_path, workspace_root);
            println!(
                "Made {} replacements in {}",
                replacements_made,
                relative_path.display()
            );
        }

        if !dry_run {
            write_file(file_path, &new_content)?;
        }
    }

    Ok(())
}

/// Replace crate:: imports in a specific file (for self-refactor mode)
fn replace_crate_imports_in_file(
    file_path: &Path,
    imports: Vec<ImportInfo>,
    workspace_root: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let original_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let mut new_content = original_content.clone();
    let mut replacements_made = 0;

    // Sort imports by line number in reverse order to avoid offset issues
    let mut sorted_imports = imports;
    sorted_imports.sort_by(|a, b| b.line_number.cmp(&a.line_number));

    for import in sorted_imports {
        // For crate:: imports, we want to replace them with direct imports (remove "crate::" prefix)
        // For example: "use crate::module::Item;" becomes "use module::Item;" or just the item name

        // Look for the import statement in the content
        if let Some(import_start) =
            new_content.find(&format!("use {};", import.import_path))
        {
            // Find the end of the line
            let import_end = new_content[import_start..]
                .find('\n')
                .map(|pos| import_start + pos + 1)
                .unwrap_or(new_content.len());

            // Since we're adding pub use statements to the root, we can remove the crate:: imports
            // and they'll be available at the root level
            let relative_path =
                get_relative_path_for_display(file_path, workspace_root);

            if verbose {
                println!(
                    "  Removing crate:: import '{}' from {}",
                    import.import_path,
                    relative_path.display()
                );
            }

            // Remove the import line entirely since items will be available at root
            new_content.replace_range(import_start..import_end, "");
            replacements_made += 1;
        } else {
            // Try variations of the import statement format
            let patterns = [
                format!("use {}::{{", import.import_path),
                format!("use {{\n    {}", import.import_path),
            ];

            let mut found = false;
            for pattern in &patterns {
                if let Some(_pattern_start) = new_content.find(pattern) {
                    if verbose {
                        let relative_path = get_relative_path_for_display(
                            file_path,
                            workspace_root,
                        );
                        println!(
                            "  Found crate:: import pattern '{}' in {}",
                            pattern,
                            relative_path.display()
                        );
                    }
                    found = true;
                    break;
                }
            }

            if !found && verbose {
                println!(
                    "  Warning: Could not find crate:: import to remove: {}",
                    import.import_path
                );
            }
        }
    }

    if replacements_made > 0 {
        if verbose {
            let relative_path =
                get_relative_path_for_display(file_path, workspace_root);
            println!(
                "Removed {} crate:: imports from {}",
                replacements_made,
                relative_path.display()
            );
        }

        if !dry_run {
            write_file(file_path, &new_content)?;
        }
    }

    Ok(())
}
