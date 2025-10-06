# Import-Refactor Crate Module Architecture

## Current Architecture Analysis

The `import-refactor` crate currently has a relatively flat module structure with significant namespace pollution and unclear separation of concerns. The main issues identified are:

### Current Issues
1. **Namespace Pollution**: Many internal utilities are exposed as `pub` unnecessarily
2. **Mixed Concerns**: The `utils` module contains unrelated functionality 
3. **Feature Coupling**: AI client code mixed with core refactoring logic
4. **Poor Encapsulation**: Implementation details exposed in public APIs
5. **Unclear Dependencies**: Circular dependencies and unclear module boundaries

## Proposed Modular Architecture

### High-Level Module Organization

```
src/
├── lib.rs                     # Public API surface
├── cli/                       # Command-line interface
│   ├── mod.rs
│   ├── args.rs               # CLI argument parsing
│   └── commands.rs           # Command implementations
├── core/                     # Core refactoring functionality
│   ├── mod.rs
│   ├── api.rs                # Primary refactoring API
│   ├── config.rs             # Configuration types
│   ├── result.rs             # Result and error types
│   └── engine/               # Refactoring engine implementation
│       ├── mod.rs
│       ├── refactor.rs       # Main refactoring logic
│       ├── imports.rs        # Import transformation
│       └── exports.rs        # Export generation
├── analysis/                 # Code analysis functionality
│   ├── mod.rs
│   ├── crates.rs             # Crate discovery and analysis
│   ├── imports.rs            # Import pattern analysis
│   ├── exports.rs            # Export analysis
│   ├── duplication.rs        # Code duplication detection
│   └── compilation.rs        # Compilation checking
├── syntax/                   # Syntax tree manipulation
│   ├── mod.rs
│   ├── parser.rs             # AST parsing utilities
│   ├── transformer.rs        # AST transformation
│   ├── generator.rs          # Code generation
│   └── visitor.rs            # AST visitor patterns
├── io/                       # File system operations
│   ├── mod.rs
│   ├── files.rs              # File read/write operations
│   └── workspace.rs          # Workspace management
├── ai/                       # AI-powered analysis (feature-gated)
│   ├── mod.rs
│   ├── client.rs             # AI client abstraction
│   ├── providers/            # AI service providers
│   │   ├── mod.rs
│   │   ├── openai.rs
│   │   ├── claude.rs
│   │   ├── ollama.rs
│   │   └── embedded.rs
│   └── analysis.rs           # AI-powered code analysis
├── server/                   # Embedded LLM server (feature-gated)
│   ├── mod.rs
│   ├── candle.rs             # Candle-based server
│   ├── config.rs             # Server configuration
│   └── routes.rs             # HTTP API routes
└── common/                   # Shared utilities and types
    ├── mod.rs
    ├── error.rs              # Error types and handling
    ├── path.rs               # Path utilities
    └── format.rs             # Formatting utilities
```

## Module Responsibilities and Public APIs

### 1. Core Module (`core/`)

**Responsibility**: Primary refactoring functionality and public API

**Public Interface**:
```rust
pub use self::api::{RefactorApi, RefactorConfig, RefactorResult};
pub use self::config::{RefactorConfigBuilder, CrateNames};
pub use self::result::{RefactorError, RefactorSuccess};

// Private modules
mod engine;
```

### 2. Analysis Module (`analysis/`)

**Responsibility**: Code analysis, crate discovery, and pattern detection

**Public Interface**:
```rust
pub use self::crates::{CrateAnalyzer, CratePaths};
pub use self::imports::{ImportAnalysis, ImportPattern};
pub use self::exports::{ExportAnalysis, ExportInfo};

// Conditional public interface for AI features
#[cfg(feature = "ai-analysis")]
pub use self::duplication::{DuplicationAnalyzer, DuplicationResults};

// Private implementation
mod compilation;
```

### 3. Syntax Module (`syntax/`)

**Responsibility**: AST manipulation and code generation

**Public Interface**:
```rust
pub use self::parser::{parse_file, parse_string};
pub use self::generator::{generate_pub_use, generate_exports};

// Private implementation details
mod transformer;
mod visitor;
```

### 4. IO Module (`io/`)

**Responsibility**: File system operations and workspace management

**Public Interface**:
```rust
pub use self::files::{read_rust_file, write_rust_file};
pub use self::workspace::{WorkspaceRoot, find_crate_root};

// All implementation kept private
```

### 5. AI Module (`ai/`) - Feature Gated

**Responsibility**: AI-powered code analysis and suggestions

**Public Interface**:
```rust
#[cfg(feature = "ai")]
pub use self::client::{AiClient, AiClientFactory};
#[cfg(feature = "ai")]
pub use self::analysis::{SimilarityAnalysis, RefactoringAnalysis};

// Provider implementations kept private
mod providers;
```

### 6. Server Module (`server/`) - Feature Gated

**Responsibility**: Embedded LLM server functionality

**Public Interface**:
```rust
#[cfg(feature = "embedded-llm")]
pub use self::candle::{CandleServer, ServerResult};
#[cfg(feature = "embedded-llm")]
pub use self::config::{ServerConfig, ModelConfig};

// Implementation details kept private
mod routes;
```

### 7. CLI Module (`cli/`)

**Responsibility**: Command-line interface (used only by main.rs)

**Public Interface**:
```rust
pub use self::args::{Args, Command};
pub use self::commands::{run_refactor, run_analysis, run_server};

// Internal CLI logic kept private
```

### 8. Common Module (`common/`)

**Responsibility**: Shared utilities and common types

**Public Interface**:
```rust
pub use self::error::{Result, Error};
pub use self::path::{relative_path, normalize_path};
pub use self::format::{format_rust_code, print_summary};
```

## Public API Surface (`lib.rs`)

The main `lib.rs` should provide a clean, minimal public API:

```rust
// Core refactoring functionality - always available
pub use crate::core::{RefactorApi, RefactorConfig, RefactorResult, RefactorConfigBuilder};
pub use crate::analysis::{CrateAnalyzer, CratePaths};
pub use crate::common::{Result, Error};

// Feature-gated APIs
#[cfg(feature = "ai")]
pub use crate::ai::{AiClient, AiClientFactory};

#[cfg(feature = "embedded-llm")]
pub use crate::server::{CandleServer, ServerConfig};

// CLI module is NOT re-exported - only used by main.rs
// Internal modules are private by default
```

## Migration Strategy

### Phase 1: Core Extraction
1. Create `core/` module with main API
2. Move `RefactorApi`, `RefactorConfig`, etc. to `core/api.rs`
3. Update `lib.rs` to re-export only from `core`

### Phase 2: Analysis Separation
1. Create `analysis/` module
2. Move crate analysis functionality from current modules
3. Extract duplication analysis from `utils`

### Phase 3: Syntax Abstraction
1. Create `syntax/` module
2. Move AST manipulation utilities
3. Create clean parsing/generation interfaces

### Phase 4: Feature Modularization
1. Move AI client code to `ai/` module with feature gates
2. Move server code to `server/` module with feature gates
3. Update Cargo.toml features accordingly

### Phase 5: Utility Consolidation
1. Create `common/` module for shared utilities
2. Create `io/` module for file operations
3. Remove the catch-all `utils/` module

### Phase 6: CLI Separation
1. Move CLI-specific code to `cli/` module
2. Keep CLI module private to library users
3. Update main.rs to use CLI module

## Benefits of This Architecture

### 1. Clear Separation of Concerns
- Core refactoring logic is isolated
- AI functionality is cleanly separated and feature-gated
- File operations are centralized
- CLI is separate from library functionality

### 2. Improved Encapsulation
- Internal implementation details are hidden
- Public APIs are minimal and focused
- Feature-gated code doesn't pollute the main API

### 3. Better Testability
- Each module can be tested independently
- Clear interfaces make mocking easier
- Reduced coupling enables better unit tests

### 4. Easier Maintenance
- Related functionality is co-located
- Dependencies are explicit and minimal
- Changes have localized impact

### 5. Scalability
- New AI providers can be added to `ai/providers/`
- New analysis types can be added to `analysis/`
- Server functionality can be extended independently

## Implementation Guidelines

### Public vs Private Rules
1. **Default to Private**: Only expose what's necessary for the public API
2. **Feature Gates**: Use feature gates for optional functionality
3. **Re-exports**: Use selective re-exports in module roots
4. **Documentation**: Document all public interfaces thoroughly

### Dependency Guidelines
1. **Core Independence**: Core module should not depend on AI or server modules
2. **Common Dependencies**: All modules can depend on `common/`
3. **Feature Isolation**: Feature-gated modules should not be dependencies of core
4. **One-Way Dependencies**: Avoid circular dependencies between modules

### Error Handling
1. **Centralized Errors**: Define error types in `common/error.rs`
2. **Module-Specific Errors**: Each module can extend the base error type
3. **Feature-Specific Errors**: Feature-gated modules define their own error extensions

This modular architecture provides clear separation of concerns, improved encapsulation, and better organization while maintaining backward compatibility during migration.