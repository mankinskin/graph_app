use std::path::Path;

/// Format a path relative to workspace root for display purposes
pub fn format_relative_path(path: &Path) -> String {
    // For now, just convert to string - could be enhanced to strip workspace root
    path.display().to_string()
}

/// Format a path relative to a specific workspace root
pub fn relative_path(path: &Path, workspace_root: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Normalize a path for consistent handling
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}