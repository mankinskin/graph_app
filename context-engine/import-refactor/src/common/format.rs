use std::path::Path;

/// Format Rust code for consistent presentation
pub fn format_rust_code(code: &str) -> String {
    // Simple formatting - could be enhanced with rustfmt integration
    code.trim().to_string()
}

/// Print a summary with consistent formatting
pub fn print_summary(title: &str, items: &[String]) {
    println!("📊 {}", title);
    for item in items {
        println!("   • {}", item);
    }
    if items.is_empty() {
        println!("   (none)");
    }
    println!();
}

/// Print file location with line number (common pattern in analysis output)
pub fn print_file_location(
    path: &Path,
    workspace_root: &Path,
    line_number: usize,
) {
    use crate::common::path::relative_path;
    let relative = relative_path(path, workspace_root);
    println!("   📁 {}:{}", relative, line_number);
}

/// Print file location with additional info (used in pattern matching)
pub fn print_file_location_with_info<T: std::fmt::Display>(
    path: &Path,
    workspace_root: &Path,
    line_number: usize,
    info: T,
) {
    use crate::common::path::relative_path;
    let relative = relative_path(path, workspace_root);
    println!("      • {}:{} ({})", relative, line_number, info);
}