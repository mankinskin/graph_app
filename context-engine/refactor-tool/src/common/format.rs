use std::path::Path;

/// Format a path relative to workspace root for display purposes
pub fn format_relative_path(path: &Path) -> String {
    // For now, just convert to string - could be enhanced to strip workspace root
    path.display().to_string()
}

/// Print file location in a formatted way
pub fn print_file_location(file_path: &Path, workspace_root: &Path, line_number: usize) {
    let relative_path = file_path
        .strip_prefix(workspace_root)
        .unwrap_or(file_path);
    println!("  📁 {}:{}", relative_path.display(), line_number);
}

/// Print file location with additional info
pub fn print_file_location_with_info(
    file_path: &Path,
    workspace_root: &Path,
    line_number: usize,
    info: String,
) {
    let relative_path = file_path
        .strip_prefix(workspace_root)
        .unwrap_or(file_path);
    println!("  📁 {}:{} - {}", relative_path.display(), line_number, info);
}