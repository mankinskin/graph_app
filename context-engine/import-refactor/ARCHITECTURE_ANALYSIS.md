# Import Refactor Tool - Architecture Analysis & Improvement Proposal

## Current Architecture Overview

### Module Structure
```
src/
├── analysis/           # Code analysis functionality
│   ├── crates.rs      # Workspace/crate discovery
│   ├── imports.rs     # Import analysis & categorization  
│   ├── exports.rs     # Export analysis & merging
│   ├── exports_analyzer.rs # Export scanning utility
│   └── macro_scanning.rs   # Macro export detection
├── syntax/            # AST manipulation & code generation
│   ├── parser.rs      # Import parsing from files
│   ├── visitor.rs     # AST visitor for pub use extraction
│   ├── generator.rs   # Pub use statement generation
│   ├── transformer.rs # Import replacement strategies
│   └── item_info.rs   # Generic item information trait
├── core/              # Core business logic
│   ├── api.rs         # High-level API interface
│   └── engine/        # Refactoring execution engine
│       └── refactor.rs
├── io/                # File operations
├── common/            # Shared utilities
└── cli/               # Command-line interface
```

## Critical Duplication Analysis

### 1. **Use Tree Parsing & Extraction (HIGH SEVERITY)**

**Duplicated Logic Locations:**
- `syntax/parser.rs`: `ImportVisitor::extract_use_info_recursive()` 
- `syntax/visitor.rs`: `extract_use_items()`
- `analysis/exports.rs`: `ExistingExportParser::extract_exported_items()`
- `analysis/mod.rs`: `extract_exported_items_from_use_tree()`
- `syntax/generator.rs`: `collect_existing_pub_uses()` (different function but similar traversal)

**Current Implementation Issues:**
- 5 different functions performing nearly identical `syn::UseTree` recursive traversal
- Each has slightly different return types and collection strategies
- No shared abstraction for use tree navigation
- Different handling of edge cases (aliases, globs, groups)

### 2. **Export Analysis & Collection (MEDIUM SEVERITY)**

**Duplicated Logic Locations:**
- `analysis/exports_analyzer.rs`: `ExportAnalyzer::collect_existing_exports()`
- `analysis/exports.rs`: `ExistingExportParser::parse_existing_exports()`
- `syntax/generator.rs`: `collect_existing_pub_uses()`
- `core/engine/refactor.rs`: `RefactorEngine::update_source_lib_rs()` (inline logic)

**Current Implementation Issues:**
- Multiple approaches to scanning syntax trees for public items
- Different strategies for handling conditional compilation
- Inconsistent treatment of macro exports vs regular exports
- Redundant file reading and AST parsing

### 3. **Pub Use Generation & Merging (MEDIUM SEVERITY)**

**Duplicated Logic Locations:**
- `syntax/visitor.rs`: `merge_pub_uses()`
- `syntax/generator.rs`: `generate_nested_pub_use()` + `build_nested_structure()`
- `analysis/exports.rs`: `ExistingExportParser::merge_exports()`

**Current Implementation Issues:**
- Three different approaches to merging existing and new pub use statements
- Different nesting strategies and output formats
- Inconsistent handling of path normalization
- No unified approach to conflict resolution

### 4. **Import Replacement Strategies (LOW SEVERITY)**

**Well-Factored Areas:**
- `syntax/transformer.rs` successfully uses strategy pattern
- Clear separation between `CrossCrateReplacementStrategy` and `SelfCrateReplacementStrategy`
- Unified `replace_imports_with_strategy()` function

## Hardcoded Limitations

### 1. **String-Based Path Manipulation**
- Heavy reliance on string splitting and prefix matching
- Error-prone crate name normalization (hyphens vs underscores)
- No structured representation of import paths

### 2. **File-Specific Logic**
- Hardcoded `lib.rs` assumptions in multiple places
- Limited to single-file export consolidation
- No support for distributed pub use statements

### 3. **AST Parsing Redundancy**
- Multiple file reads and syntax tree parsing for the same files
- No caching or memoization of parsed ASTs
- Wasteful re-parsing during analysis phases

## Common Patterns Analysis

### Self-Refactoring vs Cross-Refactoring Commonalities

Both modes share the following core operations:

1. **Import Discovery**: Scan target crate files for relevant imports
2. **Export Analysis**: Analyze source crate for existing exports
3. **Conflict Resolution**: Determine which items need new pub use statements
4. **Pub Use Generation**: Create consolidated export statements
5. **Import Replacement**: Replace imports with simpler forms
6. **Compilation Validation**: Ensure changes don't break builds

**Key Differences:**
- **Scope**: Cross-crate operates between two crates, self-crate within one
- **Target Format**: Cross-crate → `use source::*`, self-crate → direct access
- **Import Sources**: Cross-crate scans target crate, self-crate scans source crate

## Proposed Improved Architecture

### Core Abstractions

#### 1. **Unified Use Tree Navigator**
```rust
pub struct UseTreeNavigator;

impl UseTreeNavigator {
    pub fn extract_items<T>(&self, tree: &UseTree, collector: &mut T) 
    where T: UseTreeItemCollector;
    
    pub fn find_patterns<P>(&self, tree: &UseTree, pattern: P) -> Vec<UseTreeMatch>
    where P: UseTreePattern;
}

pub trait UseTreeItemCollector {
    fn collect_name(&mut self, name: &str, path: &[String]);
    fn collect_glob(&mut self, path: &[String]);
    fn collect_rename(&mut self, original: &str, renamed: &str, path: &[String]);
}
```

#### 2. **Structured Import Path**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportPath {
    crate_name: String,
    segments: Vec<String>,
    final_item: String,
}

impl ImportPath {
    pub fn parse(path_str: &str) -> Result<Self>;
    pub fn normalize_crate_name(&mut self);
    pub fn relative_to_crate(&self) -> String;
    pub fn is_glob(&self) -> bool;
}
```

#### 3. **Cached AST Manager**
```rust
pub struct AstManager {
    cache: HashMap<PathBuf, (SystemTime, syn::File)>,
}

impl AstManager {
    pub fn get_or_parse(&mut self, path: &Path) -> Result<&syn::File>;
    pub fn invalidate(&mut self, path: &Path);
    pub fn get_cached_exports(&mut self, path: &Path) -> Result<ExportInfo>;
}
```

#### 4. **Generic Refactor Context**
```rust
pub struct RefactorContext {
    pub mode: RefactorMode,
    pub source_crate: CrateInfo,
    pub target_crate: Option<CrateInfo>,
    pub workspace_root: PathBuf,
    pub ast_manager: AstManager,
}

pub enum RefactorMode {
    CrossCrate { target_crate: CrateInfo },
    SelfCrate,
}
```

#### 5. **Pluggable Analysis Pipeline**
```rust
pub trait RefactorAnalyzer {
    fn analyze(&self, context: &RefactorContext) -> Result<AnalysisResult>;
}

pub struct AnalysisPipeline {
    analyzers: Vec<Box<dyn RefactorAnalyzer>>,
}

impl AnalysisPipeline {
    pub fn new() -> Self;
    pub fn add_analyzer<A: RefactorAnalyzer + 'static>(mut self, analyzer: A) -> Self;
    pub fn run(&self, context: &RefactorContext) -> Result<CombinedAnalysis>;
}
```

### Improved Module Organization

```
src/
├── core/                  # Core abstractions & business logic
│   ├── context.rs        # RefactorContext and mode definitions
│   ├── path.rs           # ImportPath structured representation  
│   ├── ast_manager.rs    # Cached AST parsing and management
│   └── pipeline.rs       # Analysis pipeline orchestration
├── analysis/              # Specialized analyzers (pluggable)
│   ├── import_analyzer.rs # Import discovery and categorization
│   ├── export_analyzer.rs # Export scanning and conflict detection
│   ├── usage_analyzer.rs  # Usage pattern analysis
│   └── dependency_analyzer.rs # Crate dependency analysis
├── syntax/                # AST manipulation primitives
│   ├── navigator.rs      # Unified UseTree navigation
│   ├── extractor.rs      # Generic item extraction
│   ├── generator.rs      # Pub use statement generation
│   └── transformer.rs    # Import replacement (existing, good)
├── strategies/            # Refactoring strategy implementations
│   ├── cross_crate.rs    # Cross-crate refactoring logic
│   ├── self_crate.rs     # Self-crate refactoring logic
│   └── common.rs         # Shared strategy utilities
├── execution/             # Refactoring execution
│   ├── planner.rs        # Refactoring plan generation
│   ├── executor.rs       # Plan execution with validation
│   └── validation.rs     # Compilation and correctness checks
└── api/                   # Public interfaces
    ├── refactor_api.rs   # High-level refactoring API
    └── analysis_api.rs   # Analysis-only API
```

### Key Architectural Improvements

#### 1. **Eliminate AST Parsing Redundancy**
- Single `AstManager` with intelligent caching
- Parse each file exactly once per refactoring session
- Invalidation based on file modification times
- Memory-efficient caching with LRU eviction

#### 2. **Unified Use Tree Processing**
- Single `UseTreeNavigator` with visitor pattern
- Pluggable collectors for different extraction needs
- Consistent handling of all use tree variants
- Comprehensive test coverage for edge cases

#### 3. **Structured Path Handling**
- Replace string manipulation with `ImportPath` struct
- Proper crate name normalization
- Path canonicalization and comparison
- Support for complex path patterns

#### 4. **Parameterized Refactoring Logic**
- Generic `RefactorContext` containing all necessary state
- Mode-specific strategies inheriting from common base
- Pluggable analyzer pipeline for extensibility
- Clear separation of analysis vs execution phases

#### 5. **Improved Error Handling**
- Structured error types with context
- Graceful degradation for parsing failures
- Better user feedback for common issues
- Recovery strategies for partial failures

### Migration Strategy

#### Phase 1: Core Infrastructure (2-3 weeks)
1. Implement `ImportPath` and path utilities
2. Create `AstManager` with caching
3. Build `UseTreeNavigator` abstraction
4. Add comprehensive tests

#### Phase 2: Analysis Consolidation (2-3 weeks)
1. Refactor export analysis to use new abstractions
2. Consolidate import discovery logic
3. Create pluggable analyzer interfaces
4. Migrate existing analyzers

#### Phase 3: Strategy Refactoring (1-2 weeks)
1. Extract common refactoring logic
2. Parameterize cross-crate vs self-crate differences
3. Implement unified execution pipeline
4. Add validation framework

#### Phase 4: API Cleanup (1 week)
1. Simplify public API surface
2. Improve error messages and user experience
3. Add configuration options
4. Performance optimization

### Expected Benefits

#### Code Quality
- **~60% reduction** in duplicated logic
- **Improved maintainability** through clear abstractions
- **Better test coverage** with focused unit tests
- **Enhanced extensibility** for future features

#### Performance
- **Faster execution** through AST caching
- **Reduced memory usage** with intelligent cache eviction
- **Parallel analysis** potential with improved architecture

#### Robustness
- **Better error handling** with structured error types
- **More reliable path handling** with structured representations
- **Comprehensive edge case coverage** through unified processing

#### Developer Experience
- **Clearer module boundaries** and responsibilities
- **Easier debugging** with better abstractions
- **Simpler testing** through dependency injection
- **Better documentation** with focused APIs

## Implementation Priority

1. **HIGH**: Use tree navigation unification (eliminates most duplication)
2. **HIGH**: AST caching system (major performance improvement)
3. **MEDIUM**: Structured path handling (reduces error-prone string manipulation)
4. **MEDIUM**: Analysis pipeline refactoring (improves extensibility)
5. **LOW**: Strategy pattern improvements (already well-factored)

This architecture would transform the current codebase from a collection of similar but distinct implementations into a cohesive, extensible framework suitable for complex refactoring operations while maintaining the existing feature set.