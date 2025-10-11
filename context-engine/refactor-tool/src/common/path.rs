use std::path::Path;

/// Format a path relative to workspace root for display purposes
pub fn format_relative_path(path: &Path) -> String {
    // For now, just convert to string - could be enhanced to strip workspace root
    path.display().to_string()
}