# Refactoring Steps System

This document describes the refactoring steps determination and dependency management system implemented in the refactor-tool.

## Overview

The refactor-tool now uses a sophisticated step-based system to determine what refactoring operations should be performed based on the CLI flags provided. This enables:

1. **Early exit when no work is needed** - True no-op behavior when `--keep-exports --keep-super` are both specified
2. **Dependency-aware execution** - Steps are executed in the correct order based on their dependencies
3. **Selective operations** - Only perform the steps that are actually requested
4. **Clear user feedback** - Show exactly what operations will be performed

## Refactoring Steps

The following refactoring steps are available:

| Step | Description | Dependencies |
|------|-------------|--------------|
| `ParseImports` | Parse import statements from crate files | None |
| `AnalyzeImports` | Analyze and categorize imports | `ParseImports` |
| `NormalizeSuperImports` | Normalize super:: imports to crate:: format | `ParseImports` |
| `GenerateExports` | Generate pub use statements in lib.rs | `ParseImports`, `AnalyzeImports` |
| `ReplaceImports` | Replace imports in target files | `ParseImports`, `AnalyzeImports` |
| `ValidateCompilation` | Validate compilation after changes | `GenerateExports`, `ReplaceImports` |

## CLI Flag Mapping

The CLI flags determine which steps are executed:

| Flag | Effect | Steps Enabled |
|------|--------|---------------|
| `--keep-super` | Skip super:: normalization | Excludes `NormalizeSuperImports` |
| `--keep-exports` | Skip export generation and import replacement | Excludes `GenerateExports`, `ReplaceImports` |
| `--dry-run` | Skip compilation validation | Excludes `ValidateCompilation` |

## Step Determination Logic

1. **True No-Op Detection**: If both `--keep-super` and `--keep-exports` are enabled, no refactoring steps are performed. The tool exits early with a message.

2. **Dependency Resolution**: Steps are automatically ordered based on their dependencies using topological sorting.

3. **Conditional Execution**: Steps are only executed if they are required by the requested operations.

## Examples

### No-Op Mode
```bash
# No refactoring operations performed
refactor-tool imports --self --keep-exports --keep-super my_crate
# Output: "No refactoring steps requested - analysis only mode"
```

### Partial Operations
```bash
# Only normalize super:: imports, no exports or replacements
refactor-tool imports --self --keep-exports my_crate
# Output: "Will normalize super:: imports"
```

### Full Refactoring
```bash
# All operations enabled
refactor-tool imports --self my_crate
# Output: "Will normalize super:: imports, generate pub use statements, and replace import statements"
```

### Using =true/=false Syntax
```bash
# Explicit true/false values
refactor-tool imports --self --keep-exports=true --keep-super=false my_crate
# Output: "Will normalize super:: imports"
```

## Implementation Details

### RefactorStep Enum
Defines the individual operations that can be performed, with methods for getting descriptions and dependencies.

### RefactorStepsConfig
Configuration structure that determines which steps should be executed based on CLI flags.

### RefactorStepsManager
Manages the execution order and provides utilities for querying the execution plan.

### RefactorSummary
Provides a human-readable description of what operations will be performed.

## Dependency Management

The system automatically resolves dependencies between steps:

- If `GenerateExports` is requested, `ParseImports` and `AnalyzeImports` are automatically included
- If `ReplaceImports` is requested, `ParseImports` and `AnalyzeImports` are automatically included  
- If `ValidateCompilation` is requested, it only runs if actual changes (`GenerateExports` or `ReplaceImports`) are made

This ensures that prerequisites are always executed in the correct order without requiring manual specification.

## Benefits

1. **Performance**: Avoid unnecessary work when no refactoring is requested
2. **Clarity**: Users see exactly what operations will be performed
3. **Flexibility**: Support partial refactoring workflows
4. **Reliability**: Automatic dependency resolution prevents ordering issues
5. **Maintainability**: Easy to add new steps or modify dependencies