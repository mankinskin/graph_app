# Import-Refactor Testing Framework

## Requirements

The testing framework addresses these key requirements:

1. **Protected Workspace Testing** - Tests must run in isolated environments without modifying original fixture files
2. **Compilation Verification** - Refactored code must be verified to compile successfully 
3. **AST-Based Validation** - Code structure changes must be validated using abstract syntax tree analysis
4. **Comprehensive Error Reporting** - Test failures must provide detailed, readable error information
5. **Modular Design** - Common testing patterns must be encapsulated to avoid code duplication

## Core Components

### TestWorkspace

The `TestWorkspace` struct provides isolated test environments:

```rust
pub struct TestWorkspace {
    pub temp_dir: TempDir,
    pub source_crate_path: PathBuf,
    pub target_crate_path: PathBuf,
    pub workspace_path: PathBuf,
}

impl TestWorkspace {
    pub fn setup(fixture_name: &str) -> Result<Self>
    pub fn run_refactor_with_validation(&mut self, scenario: &TestScenario) -> Result<RefactorResult>
}
```

### RefactorResult

Captures complete execution results including AST analysis and compilation status:

```rust
pub struct RefactorResult {
    pub success: bool,
    pub source_analysis_before: AstAnalysis,
    pub source_analysis_after: AstAnalysis,
    pub target_analysis_before: Option<AstAnalysis>,
    pub target_analysis_after: Option<AstAnalysis>,
    pub compilation_results: CompilationResults,
}
```

### AstValidator

Validates refactor results against expected outcomes:

```rust
impl AstValidator {
    pub fn validate_refactor_result(
        result: &RefactorResult,
        expected: Option<&ExpectedChanges>,
    ) -> ValidationResult
}
```

### TestFormatter

Formats test results and error information:

```rust
impl TestFormatter {
    pub fn format_test_results(
        scenario_name: &str,
        result: &RefactorResult,
        validation: &ValidationResult,
    ) -> String
    
    pub fn format_ast_details(analysis: &AstAnalysis, title: &str) -> String
}
```

## Usage Patterns

### Basic Test Pattern

```rust
#[test]
fn test_scenario() -> Result<()> {
    let scenario = &TEST_SCENARIOS[0];
    let mut workspace = TestWorkspace::setup(scenario.fixture_name)?;
    let result = workspace.run_refactor_with_validation(scenario)?;
    let validation = AstValidator::validate_refactor_result(&result, scenario.expected_changes.as_ref());
    
    assert!(validation.passed);
    assert!(result.compilation_results.source_compiles);
    assert!(result.compilation_results.target_compiles);
    
    Ok(())
}
```

### Formatted Output Pattern

```rust
#[test]
fn test_with_output() -> Result<()> {
    let mut workspace = TestWorkspace::setup("fixture_name")?;
    let result = workspace.run_refactor_with_validation(&scenario)?;
    let validation = AstValidator::validate_refactor_result(&result, expected)?;
    
    let formatted_output = TestFormatter::format_test_results(
        scenario.name, 
        &result, 
        &validation
    );
    println!("{}", formatted_output);
    
    assert!(validation.passed);
    Ok(())
}
```

### Custom Validation Pattern

```rust
#[test]
fn test_custom_validation() -> Result<()> {
    let mut workspace = TestWorkspace::setup("fixture")?;
    let result = workspace.run_refactor_with_validation(&scenario)?;
    
    // Custom AST assertions
    assert!(result.source_analysis_after.pub_use_items.len() > 
           result.source_analysis_before.pub_use_items.len());
    
    // Detailed AST inspection
    println!("{}", TestFormatter::format_ast_details(
        &result.source_analysis_after, 
        "REFACTOR RESULT"
    ));
    
    Ok(())
}
```

## Module Structure

```
tests/
├── common/
│   ├── mod.rs              # Module declarations and re-exports
│   ├── test_utils.rs       # TestWorkspace, RefactorResult, ExpectedChanges
│   ├── ast_analysis.rs     # AstAnalysis struct and analyze_ast()
│   ├── validation.rs       # AstValidator, TestFormatter
│   └── assertions.rs       # Legacy assertion helpers
├── comprehensive_tests.rs  # Examples using new framework
├── mod.rs                  # Legacy tests
└── fixtures/               # Test workspace fixtures
```

## Test Scenarios

Predefined scenarios in `TEST_SCENARIOS`:

```rust
pub const TEST_SCENARIOS: &[TestScenario] = &[
    TestScenario {
        name: "basic_refactoring",
        description: "Basic import refactoring with nested modules",
        source_crate: "source_crate",
        target_crate: "target_crate",
        fixture_name: "basic_workspace",
        expected_changes: Some(ExpectedChanges { /* ... */ }),
    },
    // ...
];
```

## Migration from Legacy Tests

Legacy functions remain available for backward compatibility:

```rust
// Legacy pattern
let temp_workspace = setup_test_workspace(scenario.fixture_name)?;
run_refactor(workspace_path, source_crate, target_crate)?;
let analysis = analyze_ast(&source_lib_path)?;

// New framework pattern  
let mut workspace = TestWorkspace::setup(scenario.fixture_name)?;
let result = workspace.run_refactor_with_validation(scenario)?;
```