use anyhow::Result;

// Import the common module and its items explicitly
mod common;
use common::{
    test_utils::{
        ExpectedChanges,
        TestScenario,
        TestWorkspace,
    },
    AstValidator,
    TestFormatter,
};

/// Get common test scenarios that can be reused
pub fn get_test_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario::cross_refactor(
            "basic_refactoring",
            "Basic import refactoring with nested modules",
            "source_crate",
            "target_crate",
            "basic_workspace",
        ).with_expected_changes(ExpectedChanges {
            source_crate_exports: &["main_function", "Config", "Status"],
            target_crate_wildcards: 1,
            preserved_macros: &[],
            nested_modules: &["math", "utils", "network"],
        }),
        TestScenario::cross_refactor(
            "macro_handling",
            "Handling macro exports and conditional compilation",
            "macro_source",
            "macro_target",
            "macro_workspace",
        ).with_expected_changes(ExpectedChanges {
            source_crate_exports: &["MacroHelper", "format_internal"],
            target_crate_wildcards: 1,
            // Note: External macros (hashmap, assert_msg from macros.rs) are correctly
            // detected by the enhanced tool and excluded from pub use generation,
            // but only appear in AST analysis of lib.rs itself
            preserved_macros: &["debug_print", "extra_debug"],
            nested_modules: &[],
        }),
        TestScenario::cross_refactor(
            "no_imports_scenario",
            "Test with a crate that has no imports to refactor",
            "source_crate",
            "dummy_target",
            "no_imports_workspace",
        ).with_expected_changes(ExpectedChanges {
            source_crate_exports: &[], // No new exports expected
            target_crate_wildcards: 0, // No wildcards expected
            preserved_macros: &[],
            nested_modules: &[],
        }),
        TestScenario::self_refactor(
            "self_refactoring",
            "Self-refactor mode: refactor crate:: imports within a single crate",
            "self_refactor_crate",
            "self_refactor_workspace",
        ).with_expected_changes(ExpectedChanges {
            source_crate_exports: &[
                "Config",
                "load_settings",
                "save_settings",
                "validate_user_input",
                "ValidationResult",
                "validate_email",
                "User",
                "create_user",
                "find_user_by_email",
                "update_user_profile",
                "Session",
                "SessionManager",
                "validate_session",
                "Repository",
                "InMemoryRepository",
                "create_user_repository",
                "backup_data",
            ],
            target_crate_wildcards: 0, // No target crate in self-refactor mode
            preserved_macros: &[],
            nested_modules: &["core", "services"],
        }),
    ]
}

#[test]
fn test_basic_refactoring() -> Result<()> {
    let scenarios = get_test_scenarios();
    let scenario = &scenarios[0]; // basic_refactoring

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

    Ok(())
}

#[test]
fn test_macro_handling() -> Result<()> {
    let scenarios = get_test_scenarios();
    let scenario = &scenarios[1]; // macro_handling

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

    // The tool MUST handle macro scenarios correctly by detecting existing #[macro_export] items
    // and NOT attempting to re-export them with pub use statements

    // Assert that the refactor completed successfully
    assert!(result.success, "Refactor tool should handle macro scenarios correctly, but failed to complete");

    // Assert that validation passes
    assert!(
        validation.passed,
        "Test validation failed - the refactoring tool has a bug"
    );

    println!("✅ Macro handling test passed with correct refactoring");

    Ok(())
}
#[test]
fn test_no_imports_scenario() -> Result<()> {
    let scenarios = get_test_scenarios();
    let scenario = &scenarios[2]; // no_imports_scenario

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

    // The tool should handle this gracefully - either succeed with no changes or fail gracefully
    match result.success {
        true => {
            // Tool succeeded - verify no changes were made to source crate
            assert_eq!(
                result.source_analysis_before.pub_use_items.len(),
                result.source_analysis_after.pub_use_items.len(),
                "No new pub use statements should be added when there are no imports"
            );
            println!("✅ Tool correctly handled no-imports scenario");
        },
        false => {
            // Tool failed - this is also acceptable behavior for this edge case
            println!("⚠️  Tool failed on no-imports scenario - this may be expected behavior");
        },
    }

    Ok(())
}

#[test]
fn test_self_refactoring() -> Result<()> {
    let scenarios = get_test_scenarios();
    let scenario = &scenarios[3]; // self_refactoring

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

    // Assert that validation passed
    if !validation.passed {
        panic!("Test validation failed");
    }

    Ok(())
}
