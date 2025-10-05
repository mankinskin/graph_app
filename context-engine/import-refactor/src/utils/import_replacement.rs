use crate::{
    crate_analyzer::CrateNames,
    import_parser::ImportInfo,
    utils::{
        common::format_relative_path,
        file_operations::write_file,
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

#[derive(Debug)]
pub enum ReplacementAction {
    Replaced { from: String, to: String },
    Removed { original: String },
    NotFound { searched_for: String },
}

/// Strategy for determining how to replace import statements
pub trait ImportReplacementStrategy {
    /// Create replacement text for an import, or None to remove it
    fn create_replacement(
        &self,
        import: &ImportInfo,
    ) -> Option<String>;

    /// Whether this import should be removed entirely
    fn should_remove_import(
        &self,
        import: &ImportInfo,
    ) -> bool {
        self.create_replacement(import).is_none()
    }

    /// Description of this replacement strategy
    fn get_description(&self) -> &str;

    /// Format verbose log message for this replacement
    fn format_verbose_message(
        &self,
        import: &ImportInfo,
        action: ReplacementAction,
        file_path: &Path,
        workspace_root: &Path,
    ) -> String;
}

/// Cross-crate replacement: A::module::Item -> A::*
pub struct CrossCrateReplacementStrategy {
    pub crate_names: CrateNames,
}

impl ImportReplacementStrategy for CrossCrateReplacementStrategy {
    fn create_replacement(
        &self,
        _import: &ImportInfo,
    ) -> Option<String> {
        Some(format!("use {}::*;", self.crate_names.source_crate()))
    }

    fn get_description(&self) -> &str {
        "Replace with glob import from source crate"
    }

    fn format_verbose_message(
        &self,
        _import: &ImportInfo,
        action: ReplacementAction,
        file_path: &Path,
        workspace_root: &Path,
    ) -> String {
        match action {
            ReplacementAction::Replaced { from, to } => {
                format!(
                    "  Replaced: {} -> {} in {}",
                    from,
                    to,
                    format_relative_path(file_path, workspace_root)
                )
            },
            ReplacementAction::NotFound { searched_for } => {
                format!(
                    "  Warning: Could not find import to replace: {} in {}",
                    searched_for,
                    format_relative_path(file_path, workspace_root)
                )
            },
            _ => unreachable!(),
        }
    }
}

/// Self-crate replacement: crate::module::Item -> (remove, use root exports)
pub struct SelfCrateReplacementStrategy;

impl ImportReplacementStrategy for SelfCrateReplacementStrategy {
    fn create_replacement(
        &self,
        _import: &ImportInfo,
    ) -> Option<String> {
        // For self-refactor mode, replace all crate:: imports with a single glob import
        Some("use crate::*;".to_string())
    }

    fn get_description(&self) -> &str {
        "Replace crate:: imports with glob import (use crate::*)"
    }
    fn format_verbose_message(
        &self,
        _import: &ImportInfo,
        action: ReplacementAction,
        file_path: &Path,
        workspace_root: &Path,
    ) -> String {
        match action {
            ReplacementAction::Replaced { from, to } => {
                format!(
                    "  Replaced crate:: import '{}' -> '{}' in {}",
                    from,
                    to,
                    format_relative_path(file_path, workspace_root)
                )
            },
            ReplacementAction::Removed { original } => {
                format!(
                    "  Removed crate:: import '{}' from {}",
                    original,
                    format_relative_path(file_path, workspace_root)
                )
            },
            ReplacementAction::NotFound { searched_for } => {
                format!(
                    "  Warning: Could not find crate:: import to replace: {} in {}",
                    searched_for, format_relative_path(file_path, workspace_root)
                )
            },
        }
    }
}

/// Unified import replacement using strategy pattern
pub fn replace_imports_with_strategy<S: ImportReplacementStrategy>(
    imports: Vec<ImportInfo>,
    strategy: S,
    workspace_root: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Group imports by file (EXACT SAME LOGIC AS BEFORE)
    let mut imports_by_file: HashMap<PathBuf, Vec<ImportInfo>> = HashMap::new();
    for import in imports {
        imports_by_file
            .entry(import.file_path.clone())
            .or_default()
            .push(import);
    }

    // Process each file
    for (file_path, file_imports) in imports_by_file {
        replace_imports_in_file_with_strategy(
            &file_path,
            file_imports,
            &strategy,
            workspace_root,
            dry_run,
            verbose,
        )?;
    }

    Ok(())
}

/// Unified file-level replacement with strategy
fn replace_imports_in_file_with_strategy<S: ImportReplacementStrategy>(
    file_path: &Path,
    imports: Vec<ImportInfo>,
    strategy: &S,
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
        let replacement_result = process_import_replacement(
            &mut new_content,
            &import,
            strategy,
            file_path,
            workspace_root,
            verbose,
        );

        if replacement_result {
            replacements_made += 1;
        }
    }

    if replacements_made > 0 {
        if verbose {
            println!(
                "Made {} replacements in {}",
                replacements_made,
                format_relative_path(file_path, workspace_root)
            );
        }

        if !dry_run {
            write_file(file_path, &new_content)?;
        }
    }

    Ok(())
}

/// Process a single import replacement using strategy
fn process_import_replacement<S: ImportReplacementStrategy>(
    content: &mut String,
    import: &ImportInfo,
    strategy: &S,
    file_path: &Path,
    workspace_root: &Path,
    verbose: bool,
) -> bool {
    // Try exact match first
    let exact_pattern = format!("use {};", import.import_path);
    if let Some(import_start) = content.find(&exact_pattern) {
        return apply_replacement_at_position(
            content,
            import_start,
            &exact_pattern,
            import,
            strategy,
            file_path,
            workspace_root,
            verbose,
        );
    }

    // Try pattern variations (UNIFIED LOGIC FROM BOTH PREVIOUS FUNCTIONS)
    let patterns = vec![
        format!("use {}", import.import_path),
        format!(
            "use {}::",
            import.import_path.split("::").next().unwrap_or("")
        ),
    ];

    for pattern in patterns {
        if let Some(pattern_start) = content.find(&pattern) {
            if let Some(semicolon_pos) = content[pattern_start..].find(';') {
                let use_end = pattern_start + semicolon_pos + 1;
                return apply_replacement_in_range(
                    content,
                    pattern_start,
                    use_end,
                    import,
                    strategy,
                    file_path,
                    workspace_root,
                    verbose,
                );
            }
        }
    }

    // Not found
    if verbose {
        let action = ReplacementAction::NotFound {
            searched_for: import.import_path.clone(),
        };
        println!(
            "{}",
            strategy.format_verbose_message(
                import,
                action,
                file_path,
                workspace_root
            )
        );
    }
    false
}

fn apply_replacement_at_position<S: ImportReplacementStrategy>(
    content: &mut String,
    start: usize,
    original_text: &str,
    import: &ImportInfo,
    strategy: &S,
    file_path: &Path,
    workspace_root: &Path,
    verbose: bool,
) -> bool {
    let end = start + original_text.len();

    if let Some(replacement) = strategy.create_replacement(import) {
        // Replace with new import
        content.replace_range(start..end, &replacement);

        if verbose {
            let action = ReplacementAction::Replaced {
                from: original_text.to_string(),
                to: replacement,
            };
            println!(
                "{}",
                strategy.format_verbose_message(
                    import,
                    action,
                    file_path,
                    workspace_root
                )
            );
        }
    } else {
        // Remove import entirely
        // Find line boundaries to remove the entire line
        let line_start =
            content[..start].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
        let line_end = content[start..]
            .find('\n')
            .map(|pos| start + pos + 1)
            .unwrap_or(content.len());

        content.replace_range(line_start..line_end, "");

        if verbose {
            let action = ReplacementAction::Removed {
                original: original_text.to_string(),
            };
            println!(
                "{}",
                strategy.format_verbose_message(
                    import,
                    action,
                    file_path,
                    workspace_root
                )
            );
        }
    }

    true
}

fn apply_replacement_in_range<S: ImportReplacementStrategy>(
    content: &mut String,
    start: usize,
    end: usize,
    import: &ImportInfo,
    strategy: &S,
    file_path: &Path,
    workspace_root: &Path,
    verbose: bool,
) -> bool {
    let original_text = content[start..end].to_string(); // Clone to avoid borrowing issues

    if let Some(replacement) = strategy.create_replacement(import) {
        // Find the start of the line for proper replacement
        let line_start =
            content[..start].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
        let _whitespace = &content[line_start..start];

        content.replace_range(start..end, &replacement);

        if verbose {
            let action = ReplacementAction::Replaced {
                from: original_text,
                to: replacement,
            };
            println!(
                "{}",
                strategy.format_verbose_message(
                    import,
                    action,
                    file_path,
                    workspace_root
                )
            );
        }
    } else {
        // Remove import entirely
        // Find line boundaries to remove the entire line
        let line_start =
            content[..start].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
        let line_end = content[end..]
            .find('\n')
            .map(|pos| end + pos + 1)
            .unwrap_or(content.len());

        content.replace_range(line_start..line_end, "");

        if verbose {
            let action = ReplacementAction::Removed {
                original: original_text,
            };
            println!(
                "{}",
                strategy.format_verbose_message(
                    import,
                    action,
                    file_path,
                    workspace_root
                )
            );
        }
    }

    true
}
