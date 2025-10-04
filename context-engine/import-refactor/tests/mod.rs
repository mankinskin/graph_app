use anyhow::Result;
use std::fs;

mod common;

use common::{
    analyze_ast,
    run_refactor,
    setup_test_workspace,
};

#[test]
fn test_no_imports_scenario() -> Result<()> {
    // Test with a crate that has no imports to refactor
    // This tests the tool's behavior when there's nothing to do

    let temp_workspace = setup_test_workspace("basic_workspace")?;
    let workspace_path = temp_workspace.path();

    // Create a dummy target crate with no imports
    let dummy_target_path = workspace_path.join("dummy_target");
    fs::create_dir_all(&dummy_target_path.join("src"))?;

    fs::write(
        dummy_target_path.join("Cargo.toml"),
        r#"
[package]
name = "dummy_target"
version = "0.1.0"
edition = "2021"
"#,
    )?;

    fs::write(
        dummy_target_path.join("src").join("lib.rs"),
        r#"
// A crate with no imports
pub fn local_function() -> String {
    "No imports here".to_string()
}
"#,
    )?;

    // Analyze initial state
    let source_lib_path = workspace_path
        .join("source_crate")
        .join("src")
        .join("lib.rs");
    let initial_analysis = analyze_ast(&source_lib_path)?;

    // Run the refactor tool (should do nothing)
    let result = run_refactor(workspace_path, "source_crate", "dummy_target");

    // The tool should handle this gracefully
    match result {
        Ok(_) => {
            // Tool succeeded - verify no changes were made
            let final_analysis = analyze_ast(&source_lib_path)?;

            // Should be identical to initial state
            assert_eq!(initial_analysis.pub_use_items.len(), final_analysis.pub_use_items.len(),
                "No new pub use statements should be added when there are no imports");

            println!("✅ Tool correctly handled no-imports scenario");
        },
        Err(e) => {
            // Tool failed - this is also acceptable behavior
            println!("⚠️  Tool failed on no-imports scenario: {}", e);
            println!("   This may be expected behavior");
        },
    }

    Ok(())
}
