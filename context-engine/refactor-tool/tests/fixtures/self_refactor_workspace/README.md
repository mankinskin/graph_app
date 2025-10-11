# Self-Refactor Workspace Test Fixture

This fixture tests the `--self` mode of the import refactor tool.

## Structure
- `self_refactor_crate/`: A single crate with internal `crate::` imports that should be refactored to root-level exports

## Test Scenario
The crate contains:
- `src/main.rs` with multiple `crate::` imports
- `src/lib.rs` with existing module exports
- `src/utils.rs` and `src/math.rs` with utility functions

The tool should:
1. Detect all `crate::` imports in `main.rs`
2. Add `pub use` statements to `lib.rs` for imported items (avoiding duplicates)
3. Remove or update the `crate::` imports to use the root-level exports