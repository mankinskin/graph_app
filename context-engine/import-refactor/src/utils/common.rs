//! Unified utilities for common operations across the import refactor tool

use std::path::Path;

/// Format a path relative to workspace root for display purposes
pub fn format_relative_path(
    path: &Path,
    workspace_root: &Path,
) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Print file location with line number (common pattern in analysis output)
pub fn print_file_location(
    path: &Path,
    workspace_root: &Path,
    line_number: usize,
) {
    let relative = format_relative_path(path, workspace_root);
    println!("   📁 {}:{}", relative, line_number);
}

/// Print file location with additional info (used in pattern matching)
pub fn print_file_location_with_info<T: std::fmt::Display>(
    path: &Path,
    workspace_root: &Path,
    line_number: usize,
    info: T,
) {
    let relative = format_relative_path(path, workspace_root);
    println!("      • {}:{} ({})", relative, line_number, info);
}

/// Create consistent path display context for error messages
pub fn path_context(
    path: &Path,
    workspace_root: &Path,
) -> String {
    format!("in {}", format_relative_path(path, workspace_root))
}
