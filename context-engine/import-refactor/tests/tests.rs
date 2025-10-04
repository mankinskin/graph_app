use anyhow::Result;

mod common;

use common::{
    AstValidator,
    TestFormatter,
    TestWorkspace,
};

use crate::common::test_utils::{
    ExpectedChanges,
    TestScenario,
};

/// Common test scenarios that can be reused
pub const TEST_SCENARIOS: &[TestScenario] = &[
    TestScenario {
        name: "basic_refactoring",
        description: "Basic import refactoring with nested modules",
        source_crate: "source_crate",
        target_crate: "target_crate",
        fixture_name: "basic_workspace",
        expected_changes: Some(ExpectedChanges {
            source_crate_exports: &["main_function", "Config", "Status"],
            target_crate_wildcards: 1,
            preserved_macros: &[],
            nested_modules: &["math", "utils"],
        }),
    },
    TestScenario {
        name: "macro_handling",
        description: "Handling macro exports and conditional compilation",
        source_crate: "macro_source",
        target_crate: "macro_target",
        fixture_name: "macro_workspace",
        expected_changes: Some(ExpectedChanges {
            source_crate_exports: &["MacroHelper"],
            target_crate_wildcards: 1,
            preserved_macros: &["debug_print", "extra_debug"],
            nested_modules: &[],
        }),
    },
];

#[test]
fn test_basic_refactoring() -> Result<()> {
    let scenario = &TEST_SCENARIOS[0]; // basic_refactoring

    println!("🚀 Starting test: {}", scenario.description);

    // Setup protected workspace
    let mut workspace = TestWorkspace::setup(scenario.fixture_name)?;

    // Run refactor with full validation
    let result = workspace.run_refactor_with_validation(scenario)?;

    // Validate results against expectations
    let validation = AstValidator::validate_refactor_result(
        &result,
        scenario.expected_changes.as_ref(),
    );

    // Format and display comprehensive results
    let formatted_output =
        TestFormatter::format_test_results(scenario.name, &result, &validation);
    println!("{}", formatted_output);

    // Assert overall success
    assert!(validation.passed, "Test validation failed");
    assert!(result.success, "Refactor execution failed");
    assert!(
        result.compilation_results.source_compiles,
        "Source crate compilation failed"
    );
    assert!(
        result.compilation_results.target_compiles,
        "Target crate compilation failed"
    );

    Ok(())
}

#[test]
fn test_macro_handling() -> Result<()> {
    let scenario = &TEST_SCENARIOS[1]; // macro_handling

    println!("🚀 Starting test: {}", scenario.description);

    // Setup protected workspace
    let mut workspace = TestWorkspace::setup(scenario.fixture_name)?;

    // Run refactor with full validation
    let result = workspace.run_refactor_with_validation(scenario)?;

    // Validate results against expectations
    let validation = AstValidator::validate_refactor_result(
        &result,
        scenario.expected_changes.as_ref(),
    );

    // Format and display comprehensive results
    let formatted_output =
        TestFormatter::format_test_results(scenario.name, &result, &validation);
    println!("{}", formatted_output);

    // Assert overall success
    assert!(validation.passed, "Test validation failed");
    assert!(result.success, "Refactor execution failed");
    assert!(
        result.compilation_results.source_compiles,
        "Source crate compilation failed"
    );
    assert!(
        result.compilation_results.target_compiles,
        "Target crate compilation failed"
    );

    Ok(())
}

#[test]
fn test_detailed_ast_inspection() -> Result<()> {
    let scenario = &TEST_SCENARIOS[0]; // basic_refactoring

    let mut workspace = TestWorkspace::setup(scenario.fixture_name)?;
    let result = workspace.run_refactor_with_validation(scenario)?;

    // Display detailed AST analysis
    println!(
        "{}",
        TestFormatter::format_ast_details(
            &result.source_analysis_before,
            "BEFORE"
        )
    );
    println!(
        "{}",
        TestFormatter::format_ast_details(
            &result.source_analysis_after,
            "AFTER"
        )
    );

    if let Some(target_before) = &result.target_analysis_before {
        println!(
            "{}",
            TestFormatter::format_ast_details(target_before, "TARGET BEFORE")
        );
    }

    if let Some(target_after) = &result.target_analysis_after {
        println!(
            "{}",
            TestFormatter::format_ast_details(target_after, "TARGET AFTER")
        );
    }

    // Verify specific transformations
    assert!(
        result.source_analysis_after.pub_use_items.len()
            > result.source_analysis_before.pub_use_items.len(),
        "Expected new pub use statements to be added"
    );

    Ok(())
}
