// Common test utilities shared across integration tests
// This module is accessible by all test files in the tests directory

pub mod assertions;
pub mod ast_analysis;
pub mod test_utils;

// Re-export commonly used items for convenience
pub use assertions::{
    assert_pub_use_contains,
    assert_public_items_exist,
    print_analysis_summary,
};
pub use ast_analysis::{
    analyze_ast,
    AstAnalysis,
};
pub use test_utils::{
    run_refactor,
    setup_test_workspace,
    TEST_SCENARIOS,
};
