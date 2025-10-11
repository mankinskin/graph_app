# Test Fixtures Directory

This directory contains test fixtures for the refactor-tool tool testing framework. Each fixture represents a different testing scenario with carefully crafted Rust code to validate specific aspects of the refactoring tool.

## Fixture Overview

### 📁 [basic_workspace/](basic_workspace/)
**Purpose**: Tests fundamental import refactoring functionality  
**Contains**: Source crate with nested modules, target crate with various import patterns  
**Tests**: `test_basic_refactoring()`, `test_detailed_ast_inspection()`  
**Status**: ✅ All tests pass, both crates compile successfully

### 📁 [macro_workspace/](macro_workspace/)  
**Purpose**: Tests macro export handling and conditional compilation  
**Contains**: Source crate with exported/private macros, target crate using macros  
**Tests**: `test_macro_handling()`  
**Status**: 🚨 FAILS - Reveals bug in macro handling (see `BUG_ANALYSIS.md`)

### 📁 [no_imports_workspace/](no_imports_workspace/)
**Purpose**: Tests edge case where target has no imports from source  
**Contains**: Independent source and target crates with no dependencies  
**Tests**: `test_no_imports_scenario()`  
**Status**: ✅ Test passes, validates tool robustness

## Fixture Requirements

### Compilation Status
All fixtures are verified to compile successfully:
- ✅ `basic_workspace`: All crates compile
- ✅ `macro_workspace`: All crates compile  
- ✅ `no_imports_workspace`: All crates compile

### Workspace Structure
Each fixture follows this structure:
```
fixture_name/
├── Cargo.toml (workspace manifest)
├── README.md (fixture documentation)
├── source_crate/
│   ├── Cargo.toml
│   └── src/lib.rs
└── target_crate/ (or dummy_target/)
    ├── Cargo.toml
    └── src/main.rs (or lib.rs)
```

## Usage in Tests

### Modern Testing Framework
All fixtures are designed to work with the `TestWorkspace` and `TestScenario` framework:

```rust
let scenario = &TEST_SCENARIOS[index];
let mut workspace = TestWorkspace::setup(scenario.fixture_name)?;
let result = workspace.run_refactor_with_validation(scenario)?;
```

### Test Scenarios Configuration
Fixtures are configured in `tests.rs`:
```rust
pub const TEST_SCENARIOS: &[TestScenario] = &[
    TestScenario {
        name: "basic_refactoring",
        fixture_name: "basic_workspace",
        // ... configuration
    },
    // ... other scenarios
];
```

## Adding New Fixtures

To add a new test fixture:

1. **Create Directory**: `tests/fixtures/new_fixture_name/`
2. **Add Workspace**: Create `Cargo.toml` with workspace members
3. **Create Crates**: Add source and target crates with appropriate code
4. **Test Compilation**: Verify all crates compile with `cargo check --workspace`
5. **Add Documentation**: Create `README.md` explaining the fixture's purpose
6. **Configure Test**: Add new `TestScenario` in `tests.rs`
7. **Write Test Function**: Create test function using the modern framework

## Best Practices

### Fixture Design
- **Clear Purpose**: Each fixture should test specific functionality
- **Realistic Code**: Use meaningful examples that represent real-world scenarios  
- **Edge Cases**: Include edge cases and boundary conditions
- **Compilation**: Ensure all code compiles before and after refactoring
- **Documentation**: Provide clear README explaining purpose and usage

### Test Design
- **Use Modern Framework**: Always use `TestWorkspace` and `TestScenario`
- **Comprehensive Validation**: Check AST changes, compilation, and expectations
- **Handle Failures**: Account for acceptable failures (e.g., macro limitations)
- **Clear Assertions**: Make test intent clear with descriptive assertions

## Maintenance

### Verification Commands
```bash
# Test individual fixture compilation
cd tests/fixtures/fixture_name && cargo check --workspace

# Run all tests
cargo test

# Run specific test with output
cargo test test_name -- --nocapture
```

### Regular Checks
- Verify fixtures compile independently
- Ensure tests pass with current refactoring tool
- Update documentation when adding new fixtures
- Review and update expected behaviors as tool evolves

## Notes

- **Macro Limitations**: The `macro_workspace` fixture reveals known limitations in macro handling
- **Tool Evolution**: Fixtures may need updates as the refactoring tool improves
- **Backward Compatibility**: Legacy helper functions are maintained for compatibility
- **Isolation**: Each test runs in an isolated temporary workspace