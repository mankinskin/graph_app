# Super Imports Workspace Test Fixture

This comprehensive test fixture is designed to test the super imports normalization functionality in the refactor-tool. It provides a complex nested module structure with various types of `super::` imports that should be normalized to `crate::` imports.

## Feature Flags

The fixture includes multiple feature flags to test conditional compilation scenarios:

### Available Features

- `basic` (default): Basic functionality
- `advanced`: Advanced processing features  
- `parser` (default): Parser-related functionality
- `validator`: Validation capabilities
- `processor`: Processing features
- `analyzer`: Analysis functionality
- `debug`: Debug utilities
- `network`: Network-related features

### Default Features
By default, `basic` and `parser` features are enabled.

## Module Structure

```
super_imports_workspace/
├── Cargo.toml (workspace manifest)
├── README.md (this file)
└── super_imports_crate/
    ├── Cargo.toml (with feature flags)
    └── src/
        ├── lib.rs                          # Root module with feature-gated pub use statements
        ├── config.rs                       # Configuration utilities
        ├── utils/
        │   ├── mod.rs
        │   ├── helper_function.rs          # Helper utilities
        │   └── string_ops/
        │       ├── mod.rs
        │       ├── capitalize.rs           # String capitalization
        │       └── reverse_string.rs       # String reversal (feature-gated)
        └── modules/
            ├── mod.rs
            ├── parser.rs                   # Parser with cfg-gated imports
            ├── validator.rs                # Validator with complex cfg conditions
            └── nested/
                ├── mod.rs
                ├── processor.rs            # Processor with feature-conditional methods
                └── deep/
                    ├── mod.rs
                    └── analyzer.rs         # Analyzer with complex cfg combinations
```

## Super Import Patterns Tested

The fixture includes various patterns of `super::` imports that should be normalized:

### Single Level Super Imports
```rust
use super::config::Config;                    // → use crate::config::Config;
use super::utils::helper_function;            // → use crate::utils::helper_function;
```

### Multiple Level Super Imports
```rust
use super::super::parser::Parser;             // → use crate::modules::parser::Parser;
use super::super::super::config::Config;     // → use crate::config::Config;
```

### Complex Chain Super Imports
```rust
use super::super::super::super::root_function;           // → use crate::root_function;
use super::super::super::super::utils::helper_function;  // → use crate::utils::helper_function;
```

### Feature-Gated Super Imports

The fixture includes various conditional compilation scenarios:

#### Simple Feature Gates
```rust
#[cfg(feature = "processor")]
use super::super::super::utils::string_ops::reverse_string;
```

#### Complex Feature Combinations
```rust
#[cfg(all(feature = "analyzer", feature = "processor"))]
use super::super::super::parser::Parser;

#[cfg(any(feature = "debug", feature = "advanced"))]
use super::super::super::super::config::save_config;

#[cfg(all(feature = "analyzer", not(feature = "basic")))]
use super::super::super::validator::Validator;
```

## Testing Different Feature Combinations

You can test the fixture with different feature combinations:

```bash
# Test with default features (basic + parser)
cargo check

# Test with all features enabled
cargo check --all-features

# Test with specific feature combinations
cargo check --features="processor,validator"
cargo check --features="analyzer,debug"
cargo check --no-default-features --features="advanced,network"
```

## Expected Normalization Results

When the `--keep-super` flag is NOT used, all `super::` imports should be normalized to `crate::` imports:

- `use super::config::Config;` → `use crate::config::Config;`
- `use super::super::parser::Parser;` → `use crate::modules::parser::Parser;`
- `use super::super::super::config::Config;` → `use crate::config::Config;`
- `use super::super::super::super::root_function;` → `use crate::root_function;`

The feature-gated imports should maintain their `#[cfg(...)]` attributes while having their import paths normalized.

## Integration with Test Framework

This fixture integrates with the refactor-tool test framework via:

1. **TestWorkspace**: Creates temporary workspaces containing this fixture
2. **TestScenario**: Defines test scenario #4 for super imports normalization  
3. **Test Function**: `test_super_imports_normalization()` validates the functionality

The test verifies that:
- All `super::` imports are correctly identified
- Normalization preserves functionality
- Feature-gated imports are handled correctly
- Complex nested module structures work properly
- The `--keep-super` flag functionality works as expected

## Compilation Verification

The fixture is designed to compile successfully with:
- ✅ Default features (`basic`, `parser`)
- ✅ All features enabled (`--all-features`)
- ✅ Various feature combinations
- ✅ No default features with specific selections

All super import chains resolve correctly and the module structure supports the intended normalization patterns.

## Usage in Tests

```rust
TestScenario::self_refactor(
    "super_imports_normalization",
    "Test super imports normalization with --keep-super flag",
    "super_imports_crate", 
    "super_imports_workspace",
)
```