use super::{
    ast_analysis::AstAnalysis,
    test_utils::{
        ExpectedChanges,
        RefactorResult,
    },
};

/// Comprehensive AST validation and reporting framework
pub struct AstValidator;

/// Expected vs actual comparison result
#[derive(Debug)]
pub struct ValidationResult {
    pub passed: bool,
    pub failures: Vec<String>,
    pub successes: Vec<String>,
}

/// Formatted output for test results
pub struct TestFormatter;

impl AstValidator {
    /// Validate refactor results against expectations
    pub fn validate_refactor_result(
        result: &RefactorResult,
        expected: Option<&ExpectedChanges>,
    ) -> ValidationResult {
        let mut failures = Vec::new();
        let mut successes = Vec::new();

        // Basic success validation
        if !result.success {
            failures.push("❌ Refactor tool execution failed".to_string());
        } else {
            successes
                .push("✅ Refactor tool executed successfully".to_string());
        }

        // AST structure validation
        Self::validate_ast_preservation(
            &result.source_analysis_before,
            &result.source_analysis_after,
            &mut failures,
            &mut successes,
        );

        // Expected changes validation
        if let Some(expected) = expected {
            Self::validate_expected_changes(
                result,
                expected,
                &mut failures,
                &mut successes,
            );
        }

        ValidationResult {
            passed: failures.is_empty(),
            failures,
            successes,
        }
    }

    /// Validate that essential AST structure is preserved
    fn validate_ast_preservation(
        before: &AstAnalysis,
        after: &AstAnalysis,
        failures: &mut Vec<String>,
        successes: &mut Vec<String>,
    ) {
        // Check that public items are preserved or increased (not lost)
        if after.public_functions.len() < before.public_functions.len() {
            failures.push(format!(
                "❌ Lost public functions: {} → {} functions",
                before.public_functions.len(),
                after.public_functions.len()
            ));
        } else {
            successes.push(format!(
                "✅ Public functions preserved: {} functions",
                after.public_functions.len()
            ));
        }

        if after.public_structs.len() < before.public_structs.len() {
            failures.push(format!(
                "❌ Lost public structs: {} → {} structs",
                before.public_structs.len(),
                after.public_structs.len()
            ));
        } else {
            successes.push(format!(
                "✅ Public structs preserved: {} structs",
                after.public_structs.len()
            ));
        }

        // Validate macros are preserved
        if after.macro_exports.len() < before.macro_exports.len() {
            failures.push(format!(
                "❌ Lost macro exports: {} → {} macros",
                before.macro_exports.len(),
                after.macro_exports.len()
            ));
        } else if !before.macro_exports.is_empty() {
            successes.push(format!(
                "✅ Macro exports preserved: {} macros",
                after.macro_exports.len()
            ));
        }

        // Check for expected increase in pub use statements
        if after.pub_use_items.len() < before.pub_use_items.len() {
            failures.push(format!(
                "❌ Lost pub use statements: {} → {} statements",
                before.pub_use_items.len(),
                after.pub_use_items.len()
            ));
        } else if after.pub_use_items.len() > before.pub_use_items.len() {
            successes.push(format!(
                "✅ Added pub use statements: {} → {} statements",
                before.pub_use_items.len(),
                after.pub_use_items.len()
            ));
        }
    }

    /// Validate against specific expected changes
    fn validate_expected_changes(
        result: &RefactorResult,
        expected: &ExpectedChanges,
        failures: &mut Vec<String>,
        successes: &mut Vec<String>,
    ) {
        let after = &result.source_analysis_after;

        // Check expected exports are present
        for expected_export in expected.source_crate_exports {
            let found = after
                .public_functions
                .iter()
                .any(|f| f.contains(expected_export))
                || after
                    .public_structs
                    .iter()
                    .any(|s| s.contains(expected_export))
                || after
                    .public_enums
                    .iter()
                    .any(|e| e.contains(expected_export))
                || after
                    .public_traits
                    .iter()
                    .any(|t| t.contains(expected_export))
                || after.pub_use_items.iter().any(|u| {
                    u.items.iter().any(|item| item.contains(expected_export))
                });

            if found {
                successes.push(format!(
                    "✅ Expected export '{}' found",
                    expected_export
                ));
            } else {
                failures.push(format!(
                    "❌ Expected export '{}' not found",
                    expected_export
                ));
            }
        }

        // Check preserved macros
        for expected_macro in expected.preserved_macros {
            let found = after
                .macro_exports
                .iter()
                .any(|m| m.contains(expected_macro));
            if found {
                successes.push(format!(
                    "✅ Expected macro '{}' preserved",
                    expected_macro
                ));
            } else {
                failures.push(format!(
                    "❌ Expected macro '{}' not preserved",
                    expected_macro
                ));
            }
        }

        // Check nested modules
        for expected_module in expected.nested_modules {
            let found = after
                .public_modules
                .iter()
                .any(|m| m.contains(expected_module));
            if found {
                successes.push(format!(
                    "✅ Expected module '{}' found",
                    expected_module
                ));
            } else {
                failures.push(format!(
                    "❌ Expected module '{}' not found",
                    expected_module
                ));
            }
        }
    }
}

impl TestFormatter {
    /// Format comprehensive test results with nice output
    pub fn format_test_results(
        scenario_name: &str,
        result: &RefactorResult,
        validation: &ValidationResult,
    ) -> String {
        let mut output = String::new();

        output
            .push_str(&format!("\n🧪 Test Results for '{}'\n", scenario_name));
        output.push_str("═".repeat(50).as_str());
        output.push('\n');

        // Execution status
        output.push_str("\n📋 Execution Status:\n");
        if result.success {
            output.push_str("   ✅ Refactor tool completed successfully\n");
        } else {
            output.push_str("   ❌ Refactor tool failed\n");
        }

        // AST changes summary
        output.push_str("\n📊 AST Changes Summary:\n");
        output.push_str(&Self::format_ast_comparison(
            &result.source_analysis_before,
            &result.source_analysis_after,
        ));

        // Validation results
        output.push_str("\n✨ Validation Results:\n");
        for success in &validation.successes {
            output.push_str(&format!("   {}\n", success));
        }
        for failure in &validation.failures {
            output.push_str(&format!("   {}\n", failure));
        }

        // Overall result
        output.push_str(&format!(
            "\n🎯 Overall Result: {}\n",
            if validation.passed {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            }
        ));
        output.push_str("═".repeat(50).as_str());
        output.push('\n');

        output
    }

    /// Format AST comparison in a readable way
    fn format_ast_comparison(
        before: &AstAnalysis,
        after: &AstAnalysis,
    ) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "   📦 Modules:        {} → {}\n",
            before.public_modules.len(),
            after.public_modules.len()
        ));
        output.push_str(&format!(
            "   🔧 Functions:      {} → {}\n",
            before.public_functions.len(),
            after.public_functions.len()
        ));
        output.push_str(&format!(
            "   📋 Structs:        {} → {}\n",
            before.public_structs.len(),
            after.public_structs.len()
        ));
        output.push_str(&format!(
            "   🏷️  Enums:          {} → {}\n",
            before.public_enums.len(),
            after.public_enums.len()
        ));
        output.push_str(&format!(
            "   ⚡ Traits:         {} → {}\n",
            before.public_traits.len(),
            after.public_traits.len()
        ));
        output.push_str(&format!(
            "   🎭 Macros:         {} → {}\n",
            before.macro_exports.len(),
            after.macro_exports.len()
        ));
        output.push_str(&format!(
            "   🔄 Pub Use:        {} → {}\n",
            before.pub_use_items.len(),
            after.pub_use_items.len()
        ));

        // Show detailed pub use changes if significant
        if after.pub_use_items.len() > before.pub_use_items.len() {
            output.push_str("   📝 New pub use statements:\n");
            for (i, use_item) in after
                .pub_use_items
                .iter()
                .skip(before.pub_use_items.len())
                .enumerate()
            {
                output.push_str(&format!(
                    "      {}. {} → {:?}\n",
                    i + 1,
                    use_item.path,
                    use_item.items
                ));
            }
        }

        output
    }

    /// Format detailed AST structure for debugging
    pub fn format_ast_details(
        analysis: &AstAnalysis,
        title: &str,
    ) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n📊 {} AST Analysis:\n", title));
        output.push_str("─".repeat(40).as_str());
        output.push('\n');

        if !analysis.public_functions.is_empty() {
            output.push_str("🔧 Public Functions:\n");
            for func in &analysis.public_functions {
                output.push_str(&format!("   • {}\n", func));
            }
        }

        if !analysis.public_structs.is_empty() {
            output.push_str("📋 Public Structs:\n");
            for strukt in &analysis.public_structs {
                output.push_str(&format!("   • {}\n", strukt));
            }
        }

        if !analysis.public_enums.is_empty() {
            output.push_str("🏷️  Public Enums:\n");
            for enm in &analysis.public_enums {
                output.push_str(&format!("   • {}\n", enm));
            }
        }

        if !analysis.macro_exports.is_empty() {
            output.push_str("🎭 Macro Exports:\n");
            for mac in &analysis.macro_exports {
                output.push_str(&format!("   • {}\n", mac));
            }
        }

        if !analysis.pub_use_items.is_empty() {
            output.push_str("🔄 Pub Use Statements:\n");
            for (i, use_item) in analysis.pub_use_items.iter().enumerate() {
                output.push_str(&format!(
                    "   {}. {} → {:?} (nested: {})\n",
                    i + 1,
                    use_item.path,
                    use_item.items,
                    use_item.is_nested
                ));
            }
        }

        output
    }
}
