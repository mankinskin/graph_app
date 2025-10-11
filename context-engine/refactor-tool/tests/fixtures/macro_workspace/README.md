# Macro Workspace Fixture

## Overview
This fixture tests the handling of macro exports and conditional compilation in refactoring scenarios. It focuses on ensuring that macros are properly handled during the import refactoring process.

## Structure
```
macro_workspace/
├── Cargo.toml (workspace manifest)
├── macro_source/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs (macro definitions and exports)
└── macro_target/
    ├── Cargo.toml
    └── src/
        └── main.rs (imports and uses macros)
```

## Source Crate Features
- **Exported Macros**:
  - `debug_print!` - Basic debug printing macro (always available)
  - `extra_debug!` - Additional debug macro (feature-gated)
- **Non-exported Macros**:
  - `private_macro!` - Internal macro that should not be exported
- **Conditional Functions**:
  - `debug_function()` - Function with different implementations based on features
- **Regular Exports**:
  - `MacroHelper` struct - Regular struct for testing mixed exports
- **Test-only Items**:
  - `test_setup()` - Function only available during testing
- **Feature Gates**:
  - `extra_macros` feature for additional macro exports
  - `debug_mode` feature for conditional function behavior

## Target Crate Usage
The target crate demonstrates:
- Macro usage: `debug_print!("Hello from macro!")`
- Regular struct usage: `MacroHelper::new(100)`
- Conditional function calls: `debug_function()`

## Test Cases Using This Fixture

### `test_macro_handling()`
- **Purpose**: Validates proper handling of macro exports during refactoring
- **Expected Behavior**:
  - Exported macros should be properly re-exported from source crate
  - Private macros should not be included in exports
  - Regular items should be handled normally alongside macros
  - Conditional compilation should be preserved
- **Current Status**: 🚨 **FAILS** - Reveals a bug in the refactoring tool
- **Bug Description**: The tool incorrectly attempts to add `pub use` statements for macros that are already exported with `#[macro_export]`, causing name conflicts
- **Error**: `error[E0255]: the name 'debug_print' is defined multiple times`
- **Root Cause**: Lookup logic in `RefactorEngine::generate_nested_pub_use()` fails to properly detect existing `#[macro_export]` macros
- **Fix Required**: Update the existing item check to properly recognize `#[macro_export]` macros as already globally available
- **See**: `BUG_ANALYSIS.md` for detailed technical analysis and proposed fixes
- **Validates**:
  - Macro export preservation (`debug_print`, `extra_debug`)
  - Mixed item type handling (macros + structs + functions)
  - Feature-gated item handling
  - Tool correctness with macro scenarios (currently failing)

## Expected Changes After Refactoring
- **Source Crate**:
  - New `pub use` statements: `MacroHelper`
  - Macro re-exports should be handled carefully to preserve `#[macro_export]`
- **Target Crate**:
  - ~1 wildcard import for regular items
  - Macro imports may need special handling
- **Preserved Macros**: `debug_print`, `extra_debug` (feature-dependent)

## Compilation Requirements
- **Edition**: 2021
- **Dependencies**: None
- **Features**: 
  - Optional `extra_macros` for additional macro exports
  - Optional `debug_mode` for conditional function behavior

## Special Considerations
- **Macro Handling**: Macros require special import syntax and cannot always use wildcard imports
- **Feature Gates**: Conditional compilation must be preserved during refactoring
- **Mixed Exports**: Tests handling of both macros and regular items in the same crate
- **Scope**: Macro imports have different scoping rules than regular items

## Notes
- This fixture specifically tests edge cases with macro exports
- Demonstrates conditional compilation with feature flags
- Tests the refactoring tool's ability to handle mixed item types
- Validates that macro re-exports maintain proper functionality
- Private macros should remain private and not be included in exports