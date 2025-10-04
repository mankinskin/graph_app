# Basic Workspace Fixture

## Overview
This fixture provides a comprehensive example of basic Rust code refactoring scenarios. It contains a source crate with multiple modules and types, and a target crate that imports various items from the source crate.

## Structure
```
basic_workspace/
├── Cargo.toml (workspace manifest)
├── source_crate/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs (main library with exports)
│       ├── math.rs (mathematical operations module)
│       └── utils.rs (utility functions module)
└── target_crate/
    ├── Cargo.toml
    └── src/
        └── main.rs (imports from source_crate)
```

## Source Crate Features
- **Public Functions**: `main_function()` - basic function export
- **Public Structs**: `Config` - configuration struct with fields
- **Public Enums**: `Status` - enum with multiple variants  
- **Public Traits**: `Processable` - trait definition
- **Public Constants**: `MAGIC_NUMBER`, `GLOBAL_STATE` - various constant types
- **Nested Modules**: 
  - `math` module with `calculate()`, `add()`, `subtract()` functions
  - `utils` module with `format_string()` function and conditional features
- **Private Items**: Functions and structs that should not be exported
- **Conditional Compilation**: Items behind feature flags

## Target Crate Imports
The target crate imports various items from the source crate:
- Direct function imports: `main_function`
- Struct imports: `Config` 
- Enum imports: `Status`, `Processable`
- Constant imports: `MAGIC_NUMBER`
- Module function imports: `math::{add, subtract}`, `math::advanced::Calculator`
- Utility imports: `utils::format_string`

## Test Cases Using This Fixture

### `test_basic_refactoring()`
- **Purpose**: Validates basic import refactoring functionality
- **Expected Behavior**: 
  - Source crate should gain `pub use` statements for exported items
  - Target crate should use wildcard imports where appropriate
  - Both crates should compile successfully after refactoring
- **Validates**: 
  - Basic function and struct exports
  - Module re-exports
  - Compilation integrity

### `test_detailed_ast_inspection()`
- **Purpose**: Provides detailed AST analysis before and after refactoring
- **Expected Behavior**: 
  - Demonstrates AST changes in detail
  - Shows increase in `pub use` statements
  - Validates structural preservation

## Expected Changes After Refactoring
- **Source Crate**: 
  - New `pub use` statements: `main_function`, `Config`, `Status`
  - Module re-exports for `math` and `utils` items
- **Target Crate**: 
  - ~1 wildcard import (`use source_crate::*`)
  - Simplified import statements

## Compilation Requirements
- **Edition**: 2021
- **Dependencies**: None
- **Features**: Optional `extra_utils` feature for conditional compilation testing

## Notes
- Contains both public and private items to test export filtering
- Includes nested modules to test module re-export functionality  
- Has various Rust item types (functions, structs, enums, traits, constants)
- Uses conditional compilation to test feature-gated exports