use super::{
    ast_analysis::AstAnalysis,
    test_utils::{ExpectedChanges, RefactorResult},
};
use quote;
use syn::ItemUse;

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
            failures.push("❌ Refactor tool execution failed - the tool encountered errors during processing".to_string());
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
            let lost =
                before.public_functions.len() - after.public_functions.len();
            failures.push(format!(
                "❌ Lost {} public function(s): {} → {} functions - refactoring should not remove public API",
                lost,
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
            let lost = before.public_structs.len() - after.public_structs.len();
            failures.push(format!(
                "❌ Lost {} public struct(s): {} → {} structs - refactoring should not remove public API",
                lost,
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
            let lost = before.macro_exports.len() - after.macro_exports.len();
            failures.push(format!(
                "❌ Lost {} macro export(s): {} → {} macros - refactoring should not remove macro_export items",
                lost,
                before.macro_exports.len(),
                after.macro_exports.len()
            ));
        } else if !before.macro_exports.is_empty() {
            successes.push(format!(
                "✅ Macro exports preserved: {} macros",
                after.macro_exports.len()
            ));
        }

        // Check pub use statement changes (note: refactoring may consolidate statements)
        if after.pub_use_items.len() == 0 && before.pub_use_items.len() > 0 {
            failures.push(format!(
                "❌ All pub use statements removed: {} → 0 statements - refactoring may have failed to generate expected exports",
                before.pub_use_items.len()
            ));
        } else if after.pub_use_items.len() < before.pub_use_items.len() {
            successes.push(format!(
                "✅ Consolidated pub use statements: {} → {} statements (expected behavior for refactoring)",
                before.pub_use_items.len(),
                after.pub_use_items.len()
            ));
        } else if after.pub_use_items.len() > before.pub_use_items.len() {
            let added = after.pub_use_items.len() - before.pub_use_items.len();
            successes.push(format!(
                "✅ Added {} pub use statement(s): {} → {} statements (refactor tool generated new exports)",
                added,
                before.pub_use_items.len(),
                after.pub_use_items.len()
            ));
        } else {
            successes.push(format!(
                "✅ Maintained pub use statements: {} statements",
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

        // Validate pub use structure if specified
        if let Some(expected_pub_use) = &expected.expected_pub_use {
            Self::validate_pub_use_structure(
                after,
                Some(expected_pub_use),
                failures,
                successes,
            );
        } else {
            Self::validate_pub_use_structure(after, None, failures, successes);
        }

        // Validate expected number of glob imports in target crate after refactoring
        let actual_glob_imports = result.target_glob_imports_after;
        let expected_glob_imports = expected.target_crate_wildcards;

        if actual_glob_imports == expected_glob_imports {
            successes.push(format!(
                "✅ Target crate glob imports: {} (as expected)",
                actual_glob_imports
            ));
        } else {
            let violation = if actual_glob_imports > expected_glob_imports {
                format!(
                    "too many glob imports (+{})",
                    actual_glob_imports - expected_glob_imports
                )
            } else {
                format!(
                    "too few glob imports (-{})",
                    expected_glob_imports - actual_glob_imports
                )
            };
            failures.push(format!(
                "❌ Target crate glob imports mismatch: expected {}, found {} - {}",
                expected_glob_imports,
                actual_glob_imports,
                violation
            ));
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
                let available_macros = if after.macro_exports.is_empty() {
                    "no macros found".to_string()
                } else {
                    format!("available: [{}]", after.macro_exports.join(", "))
                };
                failures.push(format!(
                    "❌ Expected macro '{}' not preserved - {}",
                    expected_macro, available_macros
                ));
            }
        }
    }

    /// Validate the structure of pub use statements against expected ItemUse
    fn validate_pub_use_structure(
        source_analysis: &AstAnalysis,
        expected_item_use: Option<&ItemUse>,
        failures: &mut Vec<String>,
        successes: &mut Vec<String>,
    ) {
        let actual_pub_use_count = source_analysis.pub_use_items.len();

        if let Some(expected) = expected_item_use {
            if actual_pub_use_count == 0 {
                failures.push(format!(
                    "❌ Expected pub use statements but found none - the refactor tool should have generated at least one pub use statement"
                ));
                return;
            }

            successes.push(format!(
                "✅ Found {} pub use statement(s) as expected",
                actual_pub_use_count
            ));

            // Check if the expected ItemUse contains the expected patterns
            let expected_tree_str = quote::quote!(#expected).to_string();

            if expected_tree_str.contains("crate::") {
                // Validate that actual pub use statements also use crate:: pattern
                let has_crate_rooted = source_analysis
                    .pub_use_items
                    .iter()
                    .any(|item| item.path.starts_with("crate::"));

                if has_crate_rooted {
                    successes.push(
                        "✅ Expected crate-rooted pub use structure found"
                            .to_string(),
                    );
                } else {
                    let paths = source_analysis
                        .pub_use_items
                        .iter()
                        .map(|item| item.path.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    failures.push(format!(
                        "❌ Expected crate-rooted pub use structure not found - actual paths: {}",
                        paths
                    ));
                }
            }

            // Validate specific expected patterns exist
            let expected_patterns =
                Self::extract_expected_patterns(&expected_tree_str);
            for pattern in expected_patterns {
                let found = source_analysis.pub_use_items.iter().any(|item| {
                    item.path.contains(&pattern)
                        || item.items.iter().any(|i| i.contains(&pattern))
                });

                if found {
                    successes.push(format!(
                        "✅ Expected pattern '{}' found in pub use statements",
                        pattern
                    ));
                } else {
                    failures.push(format!(
                        "❌ Expected pattern '{}' not found in pub use statements",
                        pattern
                    ));
                }
            }
        } else {
            if actual_pub_use_count == 0 {
                successes
                    .push("✅ No pub use statements (as expected)".to_string());
            } else {
                successes.push(format!(
                    "✅ Found {} pub use statement(s)",
                    actual_pub_use_count
                ));
            }
        }
    }

    /// Extract key patterns from expected ItemUse for validation
    fn extract_expected_patterns(expected_str: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        // Extract module names that should appear in the pub use
        if expected_str.contains("math::") {
            patterns.push("math".to_string());
        }
        if expected_str.contains("network::") {
            patterns.push("network".to_string());
        }
        if expected_str.contains("utils::") {
            patterns.push("utils".to_string());
        }
        if expected_str.contains("core::") {
            patterns.push("core".to_string());
        }
        if expected_str.contains("services::") {
            patterns.push("services".to_string());
        }

        patterns
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

    ///// Format detailed AST structure for debugging
    //pub fn format_ast_details(
    //    analysis: &AstAnalysis,
    //    title: &str,
    //) -> String {
    //    let mut output = String::new();

    //    output.push_str(&format!("\n📊 {} AST Analysis:\n", title));
    //    output.push_str("─".repeat(40).as_str());
    //    output.push('\n');

    //    if !analysis.public_functions.is_empty() {
    //        output.push_str("🔧 Public Functions:\n");
    //        for func in &analysis.public_functions {
    //            output.push_str(&format!("   • {}\n", func));
    //        }
    //    }

    //    if !analysis.public_structs.is_empty() {
    //        output.push_str("📋 Public Structs:\n");
    //        for strukt in &analysis.public_structs {
    //            output.push_str(&format!("   • {}\n", strukt));
    //        }
    //    }

    //    if !analysis.public_enums.is_empty() {
    //        output.push_str("🏷️  Public Enums:\n");
    //        for enm in &analysis.public_enums {
    //            output.push_str(&format!("   • {}\n", enm));
    //        }
    //    }

    //    if !analysis.macro_exports.is_empty() {
    //        output.push_str("🎭 Macro Exports:\n");
    //        for mac in &analysis.macro_exports {
    //            output.push_str(&format!("   • {}\n", mac));
    //        }
    //    }

    //    if !analysis.pub_use_items.is_empty() {
    //        output.push_str("🔄 Pub Use Statements:\n");
    //        for (i, use_item) in analysis.pub_use_items.iter().enumerate() {
    //            output.push_str(&format!(
    //                "   {}. {} → {:?} (nested: {})\n",
    //                i + 1,
    //                use_item.path,
    //                use_item.items,
    //                use_item.is_nested
    //            ));
    //        }
    //    }

    //    output
    //}
}
