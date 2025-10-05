//! Utility for analyzing which exported items are used in files
//! This is crucial for self-refactor mode to determine where to add import statements

use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Analyze which exported items are used in a file
/// Returns a set of identifiers that appear to be used in the file
pub fn analyze_item_usage_in_file(
    file_path: &Path,
    exported_items: &[String],
) -> Result<HashSet<String>> {
    let content = fs::read_to_string(file_path)?;
    let mut used_items = HashSet::new();

    // For each exported item, check if it's used in the file
    for item in exported_items {
        if is_item_used_in_content(&content, item) {
            used_items.insert(item.clone());
        }
    }

    Ok(used_items)
}

/// Add import statements to a file for the given items
pub fn add_import_statements_to_file(
    file_path: &Path,
    items_to_import: &HashSet<String>,
    dry_run: bool,
) -> Result<()> {
    if items_to_import.is_empty() {
        return Ok(());
    }

    let content = fs::read_to_string(file_path)?;
    
    // Create the import statement
    let mut items: Vec<String> = items_to_import.iter().cloned().collect();
    items.sort(); // For consistent output
    
    let import_statement = if items.len() == 1 {
        format!("use crate::{};", items[0])
    } else {
        let items_str = items.join(", ");
        format!("use crate::{{{}}};", items_str)
    };

    // Find the best place to insert the import
    let new_content = insert_import_statement(&content, &import_statement)?;

    if !dry_run {
        fs::write(file_path, new_content)?;
    }

    println!("  Added import: {} to {}", import_statement, file_path.display());
    Ok(())
}

/// Insert an import statement at the appropriate location in the file
fn insert_import_statement(content: &str, import_statement: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut import_added = false;

    for (i, line) in lines.iter().enumerate() {
        // Add the import after any existing use statements, or at the top after comments
        if !import_added && should_insert_import_before_line(line, i, &lines) {
            result.push(import_statement);
            result.push(""); // Add blank line after import
            import_added = true;
        }
        result.push(line);
    }

    // If we didn't find a good place, add at the very beginning
    if !import_added {
        result.insert(0, "");
        result.insert(0, import_statement);
    }

    Ok(result.join("\n"))
}

/// Determine if we should insert the import statement before this line
fn should_insert_import_before_line(line: &str, index: usize, all_lines: &[&str]) -> bool {
    let trimmed = line.trim();
    
    // Don't insert before comments at the start of file
    if trimmed.starts_with("//") || trimmed.starts_with("/*") {
        return false;
    }
    
    // Don't insert before existing use statements
    if trimmed.starts_with("use ") {
        return false;
    }
    
    // Insert before the first non-comment, non-use line
    if !trimmed.is_empty() {
        return true;
    }
    
    // For empty lines, check if this is after all the imports/comments
    for &future_line in &all_lines[index + 1..] {
        let future_trimmed = future_line.trim();
        if future_trimmed.starts_with("//") || future_trimmed.starts_with("/*") || future_trimmed.starts_with("use ") {
            return false; // More imports/comments coming, wait
        }
        if !future_trimmed.is_empty() {
            return true; // Next non-empty line is not import/comment, insert here
        }
    }
    
    false
}

/// Check if an item (function, struct, etc.) appears to be used in the file content
/// This is a heuristic approach - not a full AST parse but good enough for most cases
fn is_item_used_in_content(content: &str, item: &str) -> bool {
    // Split content into tokens (rough approximation)
    let tokens: Vec<&str> = content
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .collect();

    // Check if the item appears as a standalone identifier
    tokens.contains(&item)
}

/// Analyze all files in a crate directory for usage of exported items
pub fn analyze_crate_item_usage(
    crate_path: &Path,
    exported_items: &[String],
) -> Result<Vec<(String, HashSet<String>)>> {
    let src_path = crate_path.join("src");
    let mut results = Vec::new();

    // Extract just the item names from the full crate paths
    // e.g., "crate::core::config::Config" -> "Config"
    let item_names: Vec<String> = exported_items
        .iter()
        .map(|full_path| extract_item_name(full_path))
        .collect();

    // Find all .rs files in the src directory
    let rust_files = find_rust_files(&src_path)?;

    for file_path in rust_files {
        // Skip lib.rs since that's where we're adding the pub use statements
        if file_path.file_name().and_then(|n| n.to_str()) == Some("lib.rs") {
            continue;
        }

        let used_items = analyze_item_usage_in_file(&file_path, &item_names)?;
        
        // Remove items that are defined in this file (to avoid name conflicts)
        let used_items = filter_out_defined_items(&file_path, used_items)?;
        
        if !used_items.is_empty() {
            let relative_path = file_path
                .strip_prefix(crate_path)
                .unwrap_or(&file_path)
                .to_string_lossy()
                .to_string();
            results.push((relative_path, used_items));
        }
    }

    Ok(results)
}

/// Filter out items that are defined in the given file to avoid name conflicts
fn filter_out_defined_items(file_path: &Path, used_items: HashSet<String>) -> Result<HashSet<String>> {
    let content = fs::read_to_string(file_path)?;
    let mut filtered_items = HashSet::new();
    
    for item in used_items {
        // Check if this item is defined in the current file
        if !is_item_defined_in_content(&content, &item) {
            filtered_items.insert(item);
        }
    }
    
    Ok(filtered_items)
}

/// Check if an item is defined in the file content
/// Looks for patterns like "pub struct ItemName", "pub fn item_name", "pub trait ItemName"
fn is_item_defined_in_content(content: &str, item: &str) -> bool {
    let patterns = [
        format!("pub struct {}", item),       // pub struct Config
        format!("pub fn {}", item),           // pub fn load_settings
        format!("pub trait {}", item),        // pub trait Repository  
        format!("pub enum {}", item),         // pub enum Status
        format!("struct {}", item),           // struct Config (without pub)
        format!("fn {}", item),               // fn load_settings (without pub)
        format!("trait {}", item),            // trait Repository (without pub)
        format!("enum {}", item),             // enum Status (without pub)
    ];
    
    // Check if any of these definition patterns exist in the content
    patterns.iter().any(|pattern| content.contains(pattern))
}

/// Extract the bare item name from a full crate path
/// e.g., "crate::core::config::Config" -> "Config"
fn extract_item_name(full_path: &str) -> String {
    full_path
        .split("::")
        .last()
        .unwrap_or(full_path)
        .to_string()
}

/// Recursively find all .rs files in a directory
fn find_rust_files(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut rust_files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively search subdirectories
                rust_files.extend(find_rust_files(&path)?);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                rust_files.push(path);
            }
        }
    }

    Ok(rust_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_is_item_used_in_content() {
        let content = r#"
            pub fn main() {
                let config = Config::new("test");
                let session = SessionManager::new();
                validate_email("test@example.com");
            }
        "#;

        assert!(is_item_used_in_content(content, "Config"));
        assert!(is_item_used_in_content(content, "SessionManager"));
        assert!(is_item_used_in_content(content, "validate_email"));
        assert!(!is_item_used_in_content(content, "NonExistentItem"));
    }

    #[test]
    fn test_analyze_item_usage_in_file() {
        // This would need a real file to test properly
        // For now, just verify the function signature works
        let exported_items = vec!["Config".to_string(), "SessionManager".to_string()];
        let result = analyze_item_usage_in_file(Path::new("nonexistent.rs"), &exported_items);
        assert!(result.is_err()); // Should fail for non-existent file
    }
}