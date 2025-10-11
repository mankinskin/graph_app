# Import Refactor Tool - Features Documentation

## Overview
A Rust command-line tool for refactoring import statements across workspace crates. Supports both cross-crate import consolidation and internal crate import cleanup with intelligent pub use generation.

---

## Core Features

### 1. Cross-Crate Import Refactoring
**Purpose**: Consolidate imports from source crate A into target crate B using glob imports.

**Usage**:
```bash
refactor-tool [OPTIONS] SOURCE_CRATE TARGET_CRATE
refactor-tool --source-crate A --target-crate B
```

**What it does**:
- Scans target crate for imports from source crate
- Generates nested `pub use` statements in source crate's `lib.rs`
- Replaces specific imports with `use source_crate::*;` in target crate
- Maintains compilation integrity

**Edge Cases Handled**:
- **Missing lib.rs**: Gracefully skips pub use generation with warning
- **Existing exports**: Detects and skips already exported items
- **Nested module paths**: Correctly handles `crate::module::submodule::Item`
- **Grouped imports**: Processes `use crate::{Item1, Item2}` correctly
- **Import aliases**: Handles `use crate::Item as Alias`
- **Conditional compilation**: Preserves `#[cfg(...)]` attributes on exported items

### 2. Self-Crate Import Refactoring
**Purpose**: Clean up internal `crate::` imports within a single crate.

**Usage**:
```bash
refactor-tool --self CRATE_NAME
refactor-tool --self --source-crate CRATE_NAME
```

**What it does**:
- Finds all `crate::module::Item` imports within the crate
- Generates `pub use` statements at crate root level
- Removes internal `crate::` imports
- Enables direct access to items without crate:: prefix

**Edge Cases Handled**:
- **Self-referential imports**: Correctly identifies crate:: vs external crate imports
- **Root-level conflicts**: Skips exports that would conflict with existing root items
- **Module visibility**: Respects existing pub/private boundaries

### 3. Workspace Discovery & Analysis
**Purpose**: Automatically locate crates within workspace structure.

**Features**:
- **Workspace.toml parsing**: Reads workspace members from Cargo.toml
- **Recursive scanning**: Finds crates not listed in workspace (up to depth 2)
- **Target directory exclusion**: Automatically skips `target/` directories
- **Multi-root support**: Handles workspace roots that are also packages

**Edge Cases Handled**:
- **Mixed workspace/package**: Supports Cargo.toml with both workspace and package sections
- **Unlisted crates**: Discovers crates not declared in workspace members
- **Invalid Cargo.toml**: Provides clear error messages for parsing failures
- **Missing workspace**: Graceful failure with available crates list

### 4. Intelligent Import Analysis
**Purpose**: Context-aware analysis of import patterns and usage.

**Features**:
- **Glob import detection**: Identifies existing `use crate::*` patterns
- **Import categorization**: Separates specific vs glob imports
- **Usage tracking**: Shows which files import each item
- **Duplicate detection**: Identifies items imported in multiple locations

**Context Support**:
- **CrossCrate**: `source_crate::module::Item` → `source_crate::*`
- **SelfCrate**: `crate::module::Item` → root-level access

**Edge Cases Handled**:
- **Path canonicalization**: Handles relative vs absolute paths consistently
- **Import grouping**: Correctly parses complex grouped imports
- **Wildcard imports**: Distinguishes between `*` and specific items

### 5. Smart Pub Use Generation
**Purpose**: Generate organized, nested pub use statements.

**Features**:
- **Hierarchical organization**: Creates nested `pub use crate::{...}` structures
- **Conditional compilation**: Preserves `#[cfg(...)]` attributes
- **Conflict detection**: Avoids exporting items that already exist
- **Deduplication**: Prevents duplicate exports

**Generated Structure Example**:
```rust
pub use crate::{
    math::{
        add,
        subtract,
        advanced::{
            Calculator,
            scientific::{power, AdvancedCalculator}
        }
    },
    utils::{
        format_string,
        string_ops::{reverse_string, capitalize}
    }
};
```

**Edge Cases Handled**:
- **Existing pub use**: Detects and skips already exported items
- **Feature flags**: Maintains conditional compilation attributes
- **Identifier conflicts**: Prevents export conflicts with root-level items
- **Complex nesting**: Handles deep module hierarchies correctly

---

## Command Line Interface

### Arguments
- **Positional**: `[SOURCE_CRATE] [TARGET_CRATE]` or `[CRATE_NAME]` with `--self`
- **Named**: `--source-crate`/`--source`, `--target-crate`/`--target`
- **Priority**: Named arguments override positional arguments

### Flags
- **`--self`**: Enable self-refactor mode for internal crate imports
- **`--analyze`**: Run duplication analyzer instead of import refactoring
- **`--dry-run`**: Preview changes without modifying files
- **`--verbose`**: Show detailed operation information
- **`-w`/`--workspace-root`**: Specify workspace root (default: current directory)

### Modes

#### Standard Mode
```bash
refactor-tool source_crate target_crate
refactor-tool --source source_crate --target target_crate
```

#### Self-Refactor Mode
```bash
refactor-tool --self my_crate
refactor-tool --self --source my_crate
```

#### Analysis Mode
```bash
refactor-tool --analyze
refactor-tool --analyze --workspace-root /path/to/workspace
```

#### Super Imports Normalization
```bash
# Default behavior: normalize super:: to crate:: format
refactor-tool imports --self my_crate

# Keep super:: imports as-is
refactor-tool imports --self my_crate --keep-super
```

---

## Error Handling & Edge Cases

### Compilation Safety
- **Pre-check**: Validates workspace structure before starting
- **Post-check**: Runs `cargo check` after modifications
- **Rollback**: Fails fast if compilation breaks
- **Error reporting**: Detailed error messages with file locations

### File Operations
- **Permission errors**: Graceful handling of read-only files
- **Concurrent access**: Safe file reading/writing
- **Backup strategy**: Dry-run mode for safe preview
- **UTF-8 handling**: Proper encoding support

### Import Pattern Edge Cases

#### Complex Grouped Imports
```rust
// Input:
use source_crate::{math::{add, subtract}, utils::format};
use source_crate::network::{tcp::{connect, TcpStream}, http::get};

// Output:
use source_crate::*;
```

#### Conditional Compilation
```rust
// Preserved in pub use:
#[cfg(feature = "advanced")]
pub use crate::advanced_math::complex_calculations;
```

#### Import Aliases
```rust
// Input:
use source_crate::math::Calculator as MathCalc;

// Handled correctly in analysis, preserves alias context
```

#### Nested Module Paths
```rust
// Input:
use source_crate::network::protocols::tls::cipher::default_suite;

// Generates:
pub use crate::{
    network::{
        protocols::{
            tls::{
                cipher::default_suite
            }
        }
    }
};
```

### Workspace Edge Cases

#### Mixed Workspace Structure
```toml
# Cargo.toml with both workspace and package
[workspace]
members = ["crate-a", "crate-b"]

[package]
name = "root-crate"  # Also handled as workspace member
```

#### Unlisted Crates
- Scans for `Cargo.toml` files not in workspace members
- Includes them in available crates list
- Handles subdirectories up to depth 2

#### Missing Files
- **No lib.rs**: Warning message, skips pub use generation
- **No Cargo.toml**: Excludes directory from crate discovery
- **Invalid syntax**: Clear parse error messages

---

## Output & Reporting

### Standard Output
```
🔧 Import Refactor Tool
📦 Source crate (A): source_crate → will export items via pub use
📦 Target crate (B): target_crate → imports will be simplified to use A::*
📂 Workspace: /path/to/workspace

🔎 Scanning for imports of 'source_crate' in 'target_crate'...
✅ Found 31 import statements

📊 Import Analysis Summary:
  • Total imports found: 31
  • Glob imports (use source_crate::*): 0
  • Specific imports: 31
  • Unique items imported: 51
```

### Verbose Output
- Detailed import lists with file locations
- Step-by-step refactoring operations
- Compilation check results
- Warning messages for skipped items

### Dry Run Output
- Complete preview of all changes
- No file modifications
- Clear indication of what would be changed
- Instruction to run without `--dry-run`

---

## Integration & Dependencies

### Rust Ecosystem Integration
- **syn**: AST parsing and manipulation
- **quote**: Code generation
- **clap**: Command-line argument parsing
- **anyhow**: Error handling and context
- **walkdir**: Recursive directory traversal
- **toml/serde**: Cargo.toml parsing

### Cargo Integration
- **cargo check**: Compilation validation
- **Cargo.toml**: Workspace discovery
- **src/ structure**: Standard Rust project layout

### Development Workflow
- **Safe by default**: Dry-run mode prevents accidental changes
- **Fast feedback**: Quick analysis before modifications
- **CI/CD friendly**: Exit codes and error reporting
- **Version control**: Works with any VCS, no special requirements

---

## Performance Characteristics

### Scalability
- **File scanning**: Efficient recursive traversal with depth limits
- **Memory usage**: Streaming file processing, not loading entire workspace
- **Compilation checks**: Only runs cargo check when necessary

### Typical Performance
- **Small workspace** (< 10 crates): < 1 second
- **Medium workspace** (10-50 crates): 1-5 seconds  
- **Large workspace** (50+ crates): 5-30 seconds

### Performance Considerations
- **Target directory exclusion**: Prevents scanning compiled artifacts
- **Depth-limited scanning**: Avoids deep nested dependencies
- **Incremental processing**: Only processes files with relevant imports

---

This documentation covers all implemented features and edge cases as of the current version. The tool is designed to be safe, efficient, and handle real-world Rust workspace complexity.