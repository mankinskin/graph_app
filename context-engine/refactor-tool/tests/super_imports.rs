//! Tests for super imports normalization functionality

use refactor_tool::{is_super_import, ImportParser};
use tempfile::TempDir;

fn create_test_crate_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let crate_root = temp_dir.path();

    // Create directory structure
    std::fs::create_dir_all(crate_root.join("src/module/submodule")).unwrap();

    // Create Cargo.toml
    std::fs::write(
        crate_root.join("Cargo.toml"),
        "[package]\nname = \"test_crate\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    temp_dir
}

#[test]
fn test_is_super_import() {
    assert!(is_super_import("super::module"));
    assert!(is_super_import("super::module::item"));
    assert!(is_super_import("super"));
    assert!(!is_super_import("crate::module"));
    assert!(!is_super_import("std::collections"));
    assert!(!is_super_import("module::item"));
}

#[test]
fn test_find_super_imports_in_crate() {
    let temp_dir = create_test_crate_structure();
    let crate_root = temp_dir.path();

    // Create test file with super imports
    let file_content = r#"
use std::collections::HashMap;
use super::config::Config;
use super::super::utils::helper;
use crate::normal::import;

pub struct MyStruct {
    config: Config,
}
"#;

    let file_path = crate_root.join("src/module/submodule/mod.rs");
    std::fs::write(&file_path, file_content).unwrap();

    let super_imports =
        ImportParser::find_super_imports_in_crate(crate_root).unwrap();

    // Should find imports
    assert!(!super_imports.is_empty());

    // Check that we found the expected imports
    let found_imports: Vec<_> = super_imports
        .iter()
        .filter(|import_info| import_info.file_path == file_path)
        .collect();

    assert_eq!(found_imports.len(), 2);
}

#[test]
fn test_normalize_keeps_non_super_imports() {
    let temp_dir = create_test_crate_structure();
    let crate_root = temp_dir.path();

    let file_content = r#"use std::collections::HashMap;
use crate::module::Item;
use external_crate::External;"#;

    let file_path = crate_root.join("src/lib.rs");
    std::fs::write(&file_path, file_content).unwrap();

    let super_imports =
        ImportParser::find_super_imports_in_crate(crate_root).unwrap();

    // Should find no super imports
    assert!(super_imports.is_empty());
}

#[test]
fn test_multiple_files_with_super_imports() {
    let temp_dir = create_test_crate_structure();
    let crate_root = temp_dir.path();

    std::fs::create_dir_all(crate_root.join("src/another")).unwrap();

    // First file
    let file1_content = "use super::config::Config;";
    let file1_path = crate_root.join("src/module/mod.rs");
    std::fs::write(&file1_path, file1_content).unwrap();

    // Second file
    let file2_content = "use super::other::Thing;";
    let file2_path = crate_root.join("src/another/mod.rs");
    std::fs::write(&file2_path, file2_content).unwrap();

    let super_imports =
        ImportParser::find_super_imports_in_crate(crate_root).unwrap();

    // Should find imports from both files
    let file1_imports: Vec<_> = super_imports
        .iter()
        .filter(|import| import.file_path == file1_path)
        .collect();
    let file2_imports: Vec<_> = super_imports
        .iter()
        .filter(|import| import.file_path == file2_path)
        .collect();

    assert_eq!(file1_imports.len(), 1);
    assert_eq!(file2_imports.len(), 1);
}
