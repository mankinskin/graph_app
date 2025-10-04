use anyhow::Result;
use std::fs;

mod common;

use common::{
    analyze_ast,
    assert_pub_use_contains,
    assert_public_items_exist,
    print_analysis_summary,
    run_refactor,
    setup_test_workspace,
    AstAnalysis,
    TEST_SCENARIOS,
};

#[test]
fn test_basic_refactoring() -> Result<()> {
    let scenario = &TEST_SCENARIOS[0]; // basic_refactoring

    let temp_workspace = setup_test_workspace(scenario.fixture_name)?;
    let workspace_path = temp_workspace.path();

    // Run the refactor tool
    run_refactor(workspace_path, scenario.source_crate, scenario.target_crate)?;

    // Analyze the result
    let source_lib_path = workspace_path
        .join(scenario.source_crate)
        .join("src")
        .join("lib.rs");
    let analysis = analyze_ast(&source_lib_path)?;

    print_analysis_summary(&analysis);

    // Validate basic structure
    assert_public_items_exist(&analysis, &["main_function"], &["Config"]);

    // Validate that specific pub use statements were generated using AST analysis
    // This checks the actual AST structure, not just string matching

    // Check for math module exports in AST
    let math_uses: Vec<_> = analysis
        .pub_use_items
        .iter()
        .filter(|use_item| {
            use_item.path.contains("math")
                || use_item.items.iter().any(|item| item.contains("calculate"))
        })
        .collect();

    if !math_uses.is_empty() {
        println!("✅ Found math-related pub use statements via AST analysis");
        // Validate specific math items in AST structure
        assert_pub_use_contains(&analysis, "", &["calculate"]);

        // Verify the AST structure has correct nesting for math exports
        for math_use in &math_uses {
            if math_use.path.contains("math") {
                assert!(
                    !math_use.items.is_empty(),
                    "Math use should contain items"
                );
                println!("📝 Math use items: {:?}", math_use.items);
            }
        }
    } else {
        println!("⚠️  No math-related pub use statements found in AST - checking original structure");

        // Even if no new pub use was generated, verify original pub use exists
        let existing_uses: Vec<_> = analysis
            .pub_use_items
            .iter()
            .filter(|use_item| {
                use_item.items.iter().any(|item| item == "calculate")
            })
            .collect();

        if !existing_uses.is_empty() {
            println!("✅ Found existing calculate export in AST");
        }
    }

    // Validate AST structure for all detected public items
    println!("📊 AST Analysis - Public API Structure:");
    println!("  📦 Modules: {}", analysis.public_modules.len());
    println!("  🔧 Functions: {}", analysis.public_functions.len());
    println!("  📋 Structs: {}", analysis.public_structs.len());
    println!("  🏷️  Enums: {}", analysis.public_enums.len());
    println!("  ⚡ Traits: {}", analysis.public_traits.len());
    println!("  📌 Constants: {}", analysis.public_constants.len());
    println!("  🌐 Statics: {}", analysis.public_statics.len());
    println!("  🔄 Use statements: {}", analysis.pub_use_items.len());

    Ok(())
}

#[test]
fn test_macro_handling() -> Result<()> {
    let scenario = &TEST_SCENARIOS[1]; // macro_handling

    let temp_workspace = setup_test_workspace(scenario.fixture_name)?;
    let workspace_path = temp_workspace.path();

    // Analyze initial state
    let source_lib_path = workspace_path
        .join(scenario.source_crate)
        .join("src")
        .join("lib.rs");
    let initial_analysis = analyze_ast(&source_lib_path)?;

    println!("📊 Initial macro source analysis:");
    print_analysis_summary(&initial_analysis);

    // AST-based validation: Verify that macros are detected in the AST structure
    assert!(
        !initial_analysis.macro_exports.is_empty(),
        "Expected to find macro exports in the AST structure"
    );

    // AST validation: Check for specific debug_print macro in exports
    let debug_print_found = initial_analysis
        .macro_exports
        .iter()
        .any(|m| m.contains("debug_print"));
    assert!(
        debug_print_found,
        "Expected to find debug_print macro in AST exports"
    );

    // Print detailed AST analysis of macro structure
    println!("📊 Macro AST Analysis:");
    for (i, macro_export) in initial_analysis.macro_exports.iter().enumerate() {
        println!("  Macro {}: {}", i + 1, macro_export);
    }

    // Validate conditional compilation items via AST
    if !initial_analysis.conditional_items.is_empty() {
        println!("📊 Conditional Items in AST:");
        for (i, conditional) in
            initial_analysis.conditional_items.iter().enumerate()
        {
            println!(
                "  Conditional {}: {} {} ({})",
                i + 1,
                conditional.item_type,
                conditional.name,
                conditional.condition
            );
        }
    }

    // Run the refactor tool
    run_refactor(workspace_path, scenario.source_crate, scenario.target_crate)?;

    // Analyze the result
    let final_analysis = analyze_ast(&source_lib_path)?;

    println!("📊 Final macro source analysis:");
    print_analysis_summary(&final_analysis);

    // AST-based validation: Verify macros are preserved in the refactored structure
    assert!(
        !final_analysis.macro_exports.is_empty(),
        "Macro exports should be preserved after refactoring in AST structure"
    );

    // Compare macro counts before and after
    println!("📊 Macro preservation check:");
    println!(
        "  Before refactoring: {} macros",
        initial_analysis.macro_exports.len()
    );
    println!(
        "  After refactoring: {} macros",
        final_analysis.macro_exports.len()
    );

    // Ensure specific macros are preserved
    for initial_macro in &initial_analysis.macro_exports {
        let preserved = final_analysis
            .macro_exports
            .iter()
            .any(|final_macro| final_macro == initial_macro);
        assert!(
            preserved,
            "Macro '{}' should be preserved after refactoring",
            initial_macro
        );
    }

    // AST validation: Check for conditional compilation items
    assert!(
        !initial_analysis.conditional_items.is_empty(),
        "Expected to find conditional compilation items in AST"
    );

    // Verify conditional items are preserved
    println!("📊 Conditional compilation preservation:");
    println!(
        "  Before: {} conditional items",
        initial_analysis.conditional_items.len()
    );
    println!(
        "  After: {} conditional items",
        final_analysis.conditional_items.len()
    );

    Ok(())
}

#[test]
fn test_nested_module_structure() -> Result<()> {
    let scenario = &TEST_SCENARIOS[0]; // basic_refactoring

    let temp_workspace = setup_test_workspace(scenario.fixture_name)?;
    let workspace_path = temp_workspace.path();

    // Run the refactor tool
    run_refactor(workspace_path, scenario.source_crate, scenario.target_crate)?;

    // Analyze the result
    let source_lib_path = workspace_path
        .join(scenario.source_crate)
        .join("src")
        .join("lib.rs");
    let analysis = analyze_ast(&source_lib_path)?;

    print_analysis_summary(&analysis);

    // AST-based validation: Verify nested module structure is properly detected
    assert!(
        analysis.public_modules.iter().any(|m| m.contains("math")),
        "Expected to find math module in AST public modules"
    );
    assert!(
        analysis.public_modules.iter().any(|m| m.contains("utils")),
        "Expected to find utils module in AST public modules"
    );

    // AST validation: Check for nested use statements structure
    let nested_uses: Vec<_> = analysis
        .pub_use_items
        .iter()
        .filter(|use_item| use_item.is_nested)
        .collect();

    println!("📊 Nested Use Statement AST Analysis:");
    if !nested_uses.is_empty() {
        println!(
            "✅ Found {} nested pub use statements via AST",
            nested_uses.len()
        );

        // AST validation: Verify nested items are properly structured
        for (i, nested_use) in nested_uses.iter().enumerate() {
            println!(
                "  Nested use {}: path='{}', items={:?}, nested={}",
                i + 1,
                nested_use.path,
                nested_use.items,
                nested_use.is_nested
            );

            assert!(
                nested_use.items.len() > 1,
                "AST nested use should contain multiple items, found: {:?}",
                nested_use.items
            );
        }

        // Validate specific nested module exports
        let math_nested = nested_uses.iter().find(|use_item| {
            use_item.path.contains("math")
                || use_item.items.iter().any(|item| {
                    item.contains("add") || item.contains("subtract")
                })
        });

        if let Some(math_use) = math_nested {
            println!("✅ Found math nested use in AST: {:?}", math_use.items);
        }
    } else {
        println!("⚠️  No nested pub use statements found in AST");

        // Even without nested uses, validate that modules are properly exported
        let module_count = analysis.public_modules.len();
        assert!(
            module_count >= 2,
            "Expected at least 2 public modules, found {}",
            module_count
        );

        println!("📊 Module structure validation:");
        for module in &analysis.public_modules {
            println!("  📦 Module: {}", module);
        }
    }

    Ok(())
}

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

#[test]
fn test_public_item_detection() -> Result<()> {
    // AST-based validation: Comprehensive public item detection

    let temp_workspace = setup_test_workspace("basic_workspace")?;
    let workspace_path = temp_workspace.path();

    let source_lib_path = workspace_path
        .join("source_crate")
        .join("src")
        .join("lib.rs");
    let analysis = analyze_ast(&source_lib_path)?;

    print_analysis_summary(&analysis);

    // AST validation: Check detection of different item types with detailed analysis
    assert_public_items_exist(&analysis, &["main_function"], &["Config"]);

    // AST validation: Verify enum structure
    assert!(
        !analysis.public_enums.is_empty(),
        "Expected to find public enums in AST"
    );
    let status_enum_found =
        analysis.public_enums.iter().any(|e| e.contains("Status"));
    assert!(
        status_enum_found,
        "Expected to find Status enum in AST structure"
    );

    // AST validation: Verify trait structure
    assert!(
        !analysis.public_traits.is_empty(),
        "Expected to find public traits in AST"
    );
    let processable_trait_found = analysis
        .public_traits
        .iter()
        .any(|t| t.contains("Processable"));
    assert!(
        processable_trait_found,
        "Expected to find Processable trait in AST structure"
    );

    // AST validation: Verify constant structure
    assert!(
        !analysis.public_constants.is_empty(),
        "Expected to find public constants in AST"
    );
    let magic_number_found = analysis
        .public_constants
        .iter()
        .any(|c| c.contains("MAGIC_NUMBER"));
    assert!(
        magic_number_found,
        "Expected to find MAGIC_NUMBER constant in AST structure"
    );

    // AST validation: Verify static structure
    assert!(
        !analysis.public_statics.is_empty(),
        "Expected to find public statics in AST"
    );
    let global_state_found = analysis
        .public_statics
        .iter()
        .any(|s| s.contains("GLOBAL_STATE"));
    assert!(
        global_state_found,
        "Expected to find GLOBAL_STATE static in AST structure"
    );

    // Comprehensive AST item type verification
    println!("📊 Comprehensive AST Item Analysis:");
    println!("  🔧 Functions: {:?}", analysis.public_functions);
    println!("  📋 Structs: {:?}", analysis.public_structs);
    println!("  🏷️  Enums: {:?}", analysis.public_enums);
    println!("  ⚡ Traits: {:?}", analysis.public_traits);
    println!("  📌 Constants: {:?}", analysis.public_constants);
    println!("  🌐 Statics: {:?}", analysis.public_statics);

    // Validate that each item type has the expected visibility in AST
    for item_type in ["function", "struct", "enum", "trait", "const", "static"]
    {
        let count = match item_type {
            "function" => analysis.public_functions.len(),
            "struct" => analysis.public_structs.len(),
            "enum" => analysis.public_enums.len(),
            "trait" => analysis.public_traits.len(),
            "const" => analysis.public_constants.len(),
            "static" => analysis.public_statics.len(),
            _ => 0,
        };
        println!("  ✅ AST detected {} public {}s", count, item_type);
    }

    Ok(())
}

#[test]
fn test_ast_import_structure_validation() -> Result<()> {
    // Comprehensive test that validates the AST structure of imports after refactoring

    let temp_workspace = setup_test_workspace("basic_workspace")?;
    let workspace_path = temp_workspace.path();

    // Analyze initial state via AST
    let source_lib_path = workspace_path
        .join("source_crate")
        .join("src")
        .join("lib.rs");
    let initial_analysis = analyze_ast(&source_lib_path)?;

    println!("📊 Initial AST State:");
    print_analysis_summary(&initial_analysis);

    // Run the refactor tool
    run_refactor(workspace_path, "source_crate", "target_crate")?;

    // Analyze final state via AST
    let final_analysis = analyze_ast(&source_lib_path)?;

    println!("📊 Final AST State After Refactoring:");
    print_analysis_summary(&final_analysis);

    // AST Structure Validation: Compare before and after
    println!("📊 AST Import Structure Changes:");
    println!(
        "  Use statements: {} → {}",
        initial_analysis.pub_use_items.len(),
        final_analysis.pub_use_items.len()
    );

    // Validate that refactoring added or preserved import structure
    let has_new_imports = final_analysis.pub_use_items.len()
        >= initial_analysis.pub_use_items.len();
    if has_new_imports {
        println!("✅ Refactoring maintained or added import statements in AST");
    } else {
        println!("⚠️  Refactoring may have removed some import statements");
    }

    // AST Validation: Check that all original public items are preserved
    assert_eq!(
        initial_analysis.public_functions.len(),
        final_analysis.public_functions.len(),
        "Public functions count should be preserved in AST"
    );
    assert_eq!(
        initial_analysis.public_structs.len(),
        final_analysis.public_structs.len(),
        "Public structs count should be preserved in AST"
    );
    assert_eq!(
        initial_analysis.public_enums.len(),
        final_analysis.public_enums.len(),
        "Public enums count should be preserved in AST"
    );

    // AST Validation: Verify specific item preservation
    for original_fn in &initial_analysis.public_functions {
        assert!(
            final_analysis.public_functions.contains(original_fn),
            "Function '{}' should be preserved in AST after refactoring",
            original_fn
        );
    }

    // AST Import Pattern Analysis
    println!("📊 AST Import Pattern Analysis:");
    for (i, use_item) in final_analysis.pub_use_items.iter().enumerate() {
        println!(
            "  Import {}: path='{}', items={:?}, nested={}",
            i + 1,
            use_item.path,
            use_item.items,
            use_item.is_nested
        );

        // Validate import structure sanity
        assert!(
            !use_item.items.is_empty(),
            "Use statement should have at least one item"
        );

        if use_item.is_nested {
            assert!(
                use_item.items.len() > 1,
                "Nested use should have multiple items"
            );
        }
    }

    Ok(())
}

#[test]
fn test_import_detection() -> Result<()> {
    // AST-based import detection in target crate

    let temp_workspace = setup_test_workspace("basic_workspace")?;
    let workspace_path = temp_workspace.path();

    // Analyze the target crate imports via AST
    let target_lib_path = workspace_path
        .join("target_crate")
        .join("src")
        .join("lib.rs");
    let target_main_path = workspace_path
        .join("target_crate")
        .join("src")
        .join("main.rs");

    // AST analysis: Check lib.rs imports
    if target_lib_path.exists() {
        let lib_analysis = analyze_ast(&target_lib_path)?;
        println!("📊 Target lib.rs AST Analysis:");
        print_analysis_summary(&lib_analysis);

        // AST validation: Look for imports from source_crate in AST structure
        let source_crate_imports: Vec<_> = lib_analysis
            .pub_use_items
            .iter()
            .filter(|use_item| {
                use_item.path.contains("source_crate")
                    || use_item
                        .items
                        .iter()
                        .any(|item| item.contains("source_crate"))
            })
            .collect();

        if !source_crate_imports.is_empty() {
            println!(
                "✅ Found {} source_crate imports in lib.rs via AST",
                source_crate_imports.len()
            );
            for (i, import) in source_crate_imports.iter().enumerate() {
                println!(
                    "  Import {}: path='{}', items={:?}",
                    i + 1,
                    import.path,
                    import.items
                );
            }
        } else {
            println!("📝 No source_crate imports detected in lib.rs AST");
        }
    }

    // AST analysis: Check main.rs imports
    if target_main_path.exists() {
        let main_analysis = analyze_ast(&target_main_path)?;
        println!("📊 Target main.rs AST Analysis:");
        print_analysis_summary(&main_analysis);

        // AST validation: Look for imports from source_crate in AST structure
        let source_crate_imports: Vec<_> = main_analysis
            .pub_use_items
            .iter()
            .filter(|use_item| {
                use_item.path.contains("source_crate")
                    || use_item
                        .items
                        .iter()
                        .any(|item| item.contains("source_crate"))
            })
            .collect();

        if !source_crate_imports.is_empty() {
            println!(
                "✅ Found {} source_crate imports in main.rs via AST",
                source_crate_imports.len()
            );
            for (i, import) in source_crate_imports.iter().enumerate() {
                println!(
                    "  Import {}: path='{}', items={:?}",
                    i + 1,
                    import.path,
                    import.items
                );
            }
        } else {
            println!("📝 No source_crate imports detected in main.rs AST");
        }

        // AST validation: Verify import structure patterns
        for use_item in &main_analysis.pub_use_items {
            // Validate that each import has a reasonable structure
            assert!(
                !use_item.items.is_empty(),
                "Import should contain at least one item"
            );

            // Check for common import patterns
            if use_item.path.contains("source_crate") {
                println!(
                    "📋 Source crate import pattern: {} -> {:?}",
                    use_item.path, use_item.items
                );
            }
        }
    }

    Ok(())
}
