// Common test utilities shared across integration tests
// This module is accessible by all test files in the tests directory

pub mod ast_analysis;
pub mod test_utils;
pub mod validation;

// Re-export commonly used items with explicit visibility
pub use ast_analysis::analyze_ast;
pub use test_utils::{
    run_refactor,
    setup_test_workspace,
    TestWorkspace,
};
pub use validation::{
    AstValidator,
    TestFormatter,
};

// Mark the module as used for tests to help Rust Analyzer
#[allow(dead_code)]
const _: () = ();
