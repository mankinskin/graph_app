use super::ast_analysis::AstAnalysis;

/// Print a summary of AST analysis for debugging
pub fn print_analysis_summary(analysis: &AstAnalysis) {
    println!("  Public functions: {:?}", analysis.public_functions);
    println!("  Public structs: {:?}", analysis.public_structs);
    println!("  Public enums: {:?}", analysis.public_enums);
    println!("  Public traits: {:?}", analysis.public_traits);
    println!("  Public constants: {:?}", analysis.public_constants);
    println!("  Public statics: {:?}", analysis.public_statics);
    println!("  Public modules: {:?}", analysis.public_modules);
    println!("  Macro exports: {:?}", analysis.macro_exports);
    println!("  Pub use items: {} found", analysis.pub_use_items.len());
    for use_item in &analysis.pub_use_items {
        println!("    - Path: '{}', Items: {:?}, Nested: {}", 
            use_item.path, use_item.items, use_item.is_nested);
    }
    println!("  Conditional items: {:?}", analysis.conditional_items);
}

/// Validate that specific items are present in pub use statements
pub fn assert_pub_use_contains(analysis: &AstAnalysis, path: &str, expected_items: &[&str]) {
    let matching_uses: Vec<_> = analysis.pub_use_items.iter()
        .filter(|use_item| use_item.path.contains(path))
        .collect();
    
    assert!(!matching_uses.is_empty(), 
        "Expected to find pub use statements for path '{}', but found none", path);
    
    for expected_item in expected_items {
        let found = matching_uses.iter().any(|use_item| 
            use_item.items.iter().any(|item| item.contains(expected_item))
        );
        assert!(found, 
            "Expected to find '{}' in pub use statements for path '{}', but it was not found", 
            expected_item, path);
    }
}

/// Validate that specific public items exist 
pub fn assert_public_items_exist(analysis: &AstAnalysis, expected_functions: &[&str], expected_structs: &[&str]) {
    for expected_fn in expected_functions {
        assert!(analysis.public_functions.iter().any(|f| f.contains(expected_fn)),
            "Expected to find public function '{}', but it was not found", expected_fn);
    }
    
    for expected_struct in expected_structs {
        assert!(analysis.public_structs.iter().any(|s| s.contains(expected_struct)),
            "Expected to find public struct '{}', but it was not found", expected_struct);
    }
}