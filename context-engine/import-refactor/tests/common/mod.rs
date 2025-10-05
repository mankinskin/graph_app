// Common test utilities shared across integration tests
// This module is accessible by all test files in the tests directory

pub mod ast_analysis;
pub mod test_utils;
pub mod validation;

// Mark the module as used for tests to help Rust Analyzer
#[allow(dead_code)]
const _: () = ();
