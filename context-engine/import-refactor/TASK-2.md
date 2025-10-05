# Task 2: Enhanced Import Analysis & Replacement Strategy Unification

**Task Focus**: Address analyzer limitations and implement context-aware duplication detection for import functions  
**Priority**: High - Core functionality improvement  
**Estimated Effort**: 4-6 hours  

## 🚨 Problem Analysis

### **Current Analyzer Limitations**
The current duplication analyzer has significant context sensitivity issues:

1. **✅ Detects**: Exact text/hash matches (like ItemInfo trait implementations)  
2. **❌ Misses**: Logically identical functions with different parameters/contexts
3. **❌ Misses**: Similar algorithmic patterns with different string literals
4. **❌ Misses**: Duplicate logic structures with different variable names

### **Specific Missed Duplications**

#### **Critical: Import Analysis Functions**
```rust
// src/utils/import_analysis.rs:16-74 (60 lines)
pub fn analyze_imports(imports: &[ImportInfo], source_crate_name: &str, workspace_root: &Path)

// src/utils/import_analysis.rs:75-133 (60 lines)  
pub fn analyze_crate_imports(imports: &[ImportInfo], workspace_root: &Path)
```

**Identical Algorithm Pattern**:
- Same variable names: `all_imported_items`, `glob_imports`, `specific_imports`, `import_types`
- Same control flow: `for import in imports` → `if import.imported_items.contains("*")` → `else specific_imports += 1`
- Same data structures: `BTreeSet<String>`, `HashMap<String, Vec<String>>`
- Same canonicalization logic: `workspace_root.canonicalize().unwrap_or_else(...)`

**Only Difference** (2 lines out of 120):
```rust
// analyze_imports:
let simplified_import = import.import_path
    .strip_prefix(&format!("{}::", source_crate_name))
    .unwrap_or(&import.import_path);

// analyze_crate_imports:
let simplified_import = import.import_path
    .strip_prefix("crate::")
    .unwrap_or(&import.import_path);
```

#### **Critical: Import Replacement Functions**
```rust
// src/utils/import_replacement.rs:24-54 (30 lines)
pub fn replace_target_imports(imports, source_crate_name, workspace_root, dry_run, verbose)

// src/utils/import_replacement.rs:57-82 (25 lines)
pub fn replace_crate_imports(imports, workspace_root, dry_run, verbose)
```

**Identical Orchestration Pattern**:
- Same grouping logic: `HashMap<PathBuf, Vec<ImportInfo>>`
- Same iteration: `for import in imports` → `imports_by_file.entry().or_default().push()`
- Same processing: `for (file_path, file_imports) in imports_by_file`
- Same error handling: `Result<()>` return type

**Only Differences**: Function names and parameter passing (function signatures)

#### **Critical: File-Level Replacement Functions**
```rust
// src/utils/import_replacement.rs:87-205 (118 lines)
fn replace_imports_in_file(file_path, imports, source_crate_name, workspace_root, dry_run, verbose)

// src/utils/import_replacement.rs:206-297 (91 lines)
fn replace_crate_imports_in_file(file_path, imports, workspace_root, dry_run, verbose)
```

**Identical Processing Pattern**:
- Same file reading: `fs::read_to_string(file_path).with_context(...)`
- Same setup: `let mut new_content = original_content.clone(); let mut replacements_made = 0;`
- Same sorting: `sorted_imports.sort_by(|a, b| b.line_number.cmp(&a.line_number))`
- Same iteration: `for import in sorted_imports`
- Same pattern matching: `if let Some(import_start) = new_content.find(...)`
- Same line boundary detection: `find('\n').map(|pos| import_start + pos + 1)`
- Same file writing: `write_file(file_path, &new_content)?`

**Different Logic**: Replacement strategies (glob vs removal)

---

## 🎯 Task Objectives

### **Primary Goals**
1. **Unify import analysis functions** into a single parameterized function
2. **Unify import replacement strategies** using strategy pattern
3. **Eliminate 95% algorithmic duplication** while maintaining all functionality
4. **Improve extensibility** for future import types and replacement strategies

### **Success Metrics**
- ✅ `analyze_imports()` and `analyze_crate_imports()` → 1 unified function
- ✅ `replace_target_imports()` and `replace_crate_imports()` → 1 strategy-based function  
- ✅ `replace_imports_in_file()` and `replace_crate_imports_in_file()` → 1 strategy-based function
- ✅ ~150 lines of code eliminated (~60 analysis + ~90 replacement)
- ✅ Zero functional regression
- ✅ Easy to add new import contexts (e.g., `workspace::crate::module`)

---

## 🔧 Implementation Plan

### **Phase 1: Import Analysis Unification** (2-3 hours)

#### **Step 1.1: Create Import Context Abstraction**
**Target**: `src/utils/import_analysis.rs`

```rust
/// Context for import analysis - defines how imports should be processed
#[derive(Debug, Clone)]
pub enum ImportContext {
    /// Cross-crate imports: source_crate::module::Item
    CrossCrate { 
        source_crate_name: String 
    },
    /// Self-crate imports: crate::module::Item  
    SelfCrate,
}

impl ImportContext {
    /// Get the prefix to strip from import paths
    fn get_prefix_to_strip(&self) -> String {
        match self {
            ImportContext::CrossCrate { source_crate_name } => 
                format!("{}::", source_crate_name),
            ImportContext::SelfCrate => 
                "crate::".to_string(),
        }
    }
    
    /// Get label for summary display
    fn get_summary_label(&self) -> String {
        match self {
            ImportContext::CrossCrate { source_crate_name } => source_crate_name.clone(),
            ImportContext::SelfCrate => "crate".to_string(),
        }
    }
    
    /// Get glob import pattern description
    fn get_glob_pattern_description(&self) -> String {
        match self {
            ImportContext::CrossCrate { source_crate_name } => 
                format!("use {}::*", source_crate_name),
            ImportContext::SelfCrate => 
                "use crate::*".to_string(),
        }
    }
}
```

#### **Step 1.2: Implement Unified Analysis Function**
```rust
/// Unified import analysis supporting multiple import contexts
pub fn analyze_imports_unified(
    imports: &[ImportInfo],
    context: ImportContext,
    workspace_root: &Path,
) -> ImportAnalysisResult {
    let mut all_imported_items = BTreeSet::new();
    let mut glob_imports = 0;
    let mut specific_imports = 0;
    let mut import_types = std::collections::HashMap::new();

    let workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());

    let prefix_to_strip = context.get_prefix_to_strip();

    for import in imports {
        if import.imported_items.contains(&"*".to_string()) {
            glob_imports += 1;
        } else {
            specific_imports += 1;
            for item in &import.imported_items {
                if item != "*" {
                    all_imported_items.insert(item.clone());

                    let canonical_file_path = import
                        .file_path
                        .canonicalize()
                        .unwrap_or_else(|_| import.file_path.clone());

                    let relative_path = format_relative_path(&canonical_file_path, &workspace_root);

                    // Context-specific prefix stripping (THE ONLY DIFFERENCE!)
                    let simplified_import = import
                        .import_path
                        .strip_prefix(&prefix_to_strip)
                        .unwrap_or(&import.import_path);

                    import_types
                        .entry(item.clone())
                        .or_insert_with(Vec::new)
                        .push(format!("{}:{}", relative_path, simplified_import));
                }
            }
        }
    }

    ImportAnalysisResult {
        all_imported_items,
        glob_imports,
        specific_imports,
        import_types,
    }
}
```

#### **Step 1.3: Implement Unified Summary Function**
```rust
/// Unified summary printing for any import context
pub fn print_analysis_summary_unified(
    result: &ImportAnalysisResult,
    imports: &[ImportInfo],
    context: &ImportContext,
) {
    let summary_label = context.get_summary_label();
    let glob_pattern = context.get_glob_pattern_description();
    
    println!("📊 Import Analysis Summary:");
    println!("  • Total imports found: {}", imports.len());
    println!("  • Glob imports ({}): {}", glob_pattern, result.glob_imports);
    println!("  • Specific imports: {}", result.specific_imports);
    println!("  • Unique items imported: {}", result.all_imported_items.len());

    if !result.all_imported_items.is_empty() {
        match context {
            ImportContext::CrossCrate { source_crate_name } => {
                println!("\n🔍 Detected imported items from '{}':", source_crate_name);
            }
            ImportContext::SelfCrate => {
                println!("\n🔍 Detected crate:: imports:");
            }
        }
        print_imported_items(&result.all_imported_items, &result.import_types);
        println!();
    } else if result.glob_imports > 0 {
        match context {
            ImportContext::CrossCrate { source_crate_name } => {
                println!("\n💡 Note: Only glob imports (use {}::*) found. No specific items to re-export.", source_crate_name);
            }
            ImportContext::SelfCrate => {
                println!("\n💡 Note: Only glob imports (use crate::*) found. No specific items to re-export.");
            }
        }
        println!("   This means the target crate is already using the most general import pattern.");
        println!();
    }
}
```

#### **Step 1.4: Update Call Sites**
```rust
// In refactor_engine.rs:
// BEFORE:
let analysis_result = analyze_imports(&imports, &self.source_crate_name, workspace_root);

// AFTER:
let context = ImportContext::CrossCrate { 
    source_crate_name: self.source_crate_name.clone() 
};
let analysis_result = analyze_imports_unified(&imports, context, workspace_root);

// BEFORE:
let analysis_result = analyze_crate_imports(&imports, workspace_root);

// AFTER:
let context = ImportContext::SelfCrate;
let analysis_result = analyze_imports_unified(&imports, context, workspace_root);
```

### **Phase 2: Import Replacement Strategy Unification** (2-3 hours)

#### **Step 2.1: Create Replacement Strategy Trait**
**Target**: `src/utils/import_replacement.rs`

```rust
/// Strategy for determining how to replace import statements
pub trait ImportReplacementStrategy {
    /// Create replacement text for an import, or None to remove it
    fn create_replacement(&self, import: &ImportInfo) -> Option<String>;
    
    /// Whether this import should be removed entirely
    fn should_remove_import(&self, import: &ImportInfo) -> bool {
        self.create_replacement(import).is_none()
    }
    
    /// Description of this replacement strategy
    fn get_description(&self) -> &str;
    
    /// Format verbose log message for this replacement
    fn format_verbose_message(
        &self, 
        import: &ImportInfo, 
        action: ReplacementAction, 
        file_path: &Path, 
        workspace_root: &Path
    ) -> String;
}

#[derive(Debug)]
pub enum ReplacementAction {
    Replaced { from: String, to: String },
    Removed { original: String },
    NotFound { searched_for: String },
}
```

#### **Step 2.2: Implement Concrete Strategies**
```rust
/// Cross-crate replacement: A::module::Item -> A::*
pub struct CrossCrateReplacementStrategy {
    pub source_crate_name: String,
}

impl ImportReplacementStrategy for CrossCrateReplacementStrategy {
    fn create_replacement(&self, _import: &ImportInfo) -> Option<String> {
        Some(format!("use {}::*;", self.source_crate_name))
    }
    
    fn get_description(&self) -> &str {
        "Replace with glob import from source crate"
    }
    
    fn format_verbose_message(
        &self, 
        import: &ImportInfo, 
        action: ReplacementAction, 
        file_path: &Path, 
        workspace_root: &Path
    ) -> String {
        match action {
            ReplacementAction::Replaced { from, to } => {
                format!(
                    "  Replaced: {} -> {} in {}",
                    from, to, format_relative_path(file_path, workspace_root)
                )
            }
            ReplacementAction::NotFound { searched_for } => {
                format!(
                    "  Warning: Could not find import to replace: {} in {}",
                    searched_for, format_relative_path(file_path, workspace_root)
                )
            }
            _ => unreachable!(),
        }
    }
}

/// Self-crate replacement: crate::module::Item -> (remove, use root exports)
pub struct SelfCrateReplacementStrategy;

impl ImportReplacementStrategy for SelfCrateReplacementStrategy {
    fn create_replacement(&self, _import: &ImportInfo) -> Option<String> {
        None // Remove the import entirely
    }
    
    fn should_remove_import(&self, _import: &ImportInfo) -> bool {
        true
    }
    
    fn get_description(&self) -> &str {
        "Remove crate:: imports, use root-level exports"
    }
    
    fn format_verbose_message(
        &self, 
        import: &ImportInfo, 
        action: ReplacementAction, 
        file_path: &Path, 
        workspace_root: &Path
    ) -> String {
        match action {
            ReplacementAction::Removed { original } => {
                format!(
                    "  Removed crate:: import '{}' from {}",
                    original, format_relative_path(file_path, workspace_root)
                )
            }
            ReplacementAction::NotFound { searched_for } => {
                format!(
                    "  Warning: Could not find crate:: import to remove: {} in {}",
                    searched_for, format_relative_path(file_path, workspace_root)
                )
            }
            _ => unreachable!(),
        }
    }
}
```

#### **Step 2.3: Implement Unified Replacement Functions**
```rust
/// Unified import replacement using strategy pattern
pub fn replace_imports_with_strategy<S: ImportReplacementStrategy>(
    imports: Vec<ImportInfo>,
    strategy: S,
    workspace_root: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Group imports by file (EXACT SAME LOGIC AS BEFORE)
    let mut imports_by_file: HashMap<PathBuf, Vec<ImportInfo>> = HashMap::new();
    for import in imports {
        imports_by_file
            .entry(import.file_path.clone())
            .or_default()
            .push(import);
    }

    // Process each file
    for (file_path, file_imports) in imports_by_file {
        replace_imports_in_file_with_strategy(
            &file_path,
            file_imports,
            &strategy,
            workspace_root,
            dry_run,
            verbose,
        )?;
    }

    Ok(())
}

/// Unified file-level replacement with strategy
fn replace_imports_in_file_with_strategy<S: ImportReplacementStrategy>(
    file_path: &Path,
    imports: Vec<ImportInfo>,
    strategy: &S,
    workspace_root: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let original_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;

    let mut new_content = original_content.clone();
    let mut replacements_made = 0;

    // Sort imports by line number in reverse order to avoid offset issues
    let mut sorted_imports = imports;
    sorted_imports.sort_by(|a, b| b.line_number.cmp(&a.line_number));

    for import in sorted_imports {
        let replacement_result = process_import_replacement(
            &mut new_content,
            &import,
            strategy,
            file_path,
            workspace_root,
            verbose,
        );
        
        if replacement_result {
            replacements_made += 1;
        }
    }

    if replacements_made > 0 {
        if verbose {
            println!(
                "Made {} replacements in {}",
                replacements_made,
                format_relative_path(file_path, workspace_root)
            );
        }

        if !dry_run {
            write_file(file_path, &new_content)?;
        }
    }

    Ok(())
}

/// Process a single import replacement using strategy
fn process_import_replacement<S: ImportReplacementStrategy>(
    content: &mut String,
    import: &ImportInfo,
    strategy: &S,
    file_path: &Path,
    workspace_root: &Path,
    verbose: bool,
) -> bool {
    // Try exact match first
    let exact_pattern = format!("use {};", import.import_path);
    if let Some(import_start) = content.find(&exact_pattern) {
        return apply_replacement_at_position(
            content, import_start, &exact_pattern, import, strategy, file_path, workspace_root, verbose
        );
    }

    // Try pattern variations (UNIFIED LOGIC FROM BOTH PREVIOUS FUNCTIONS)
    let patterns = vec![
        format!("use {}", import.import_path),
        format!("use {}::", import.import_path.split("::").next().unwrap_or("")),
    ];

    for pattern in patterns {
        if let Some(pattern_start) = content.find(&pattern) {
            if let Some(semicolon_pos) = content[pattern_start..].find(';') {
                let use_end = pattern_start + semicolon_pos + 1;
                return apply_replacement_in_range(
                    content, pattern_start, use_end, import, strategy, file_path, workspace_root, verbose
                );
            }
        }
    }

    // Not found
    if verbose {
        let action = ReplacementAction::NotFound { 
            searched_for: import.import_path.clone() 
        };
        println!("{}", strategy.format_verbose_message(import, action, file_path, workspace_root));
    }
    false
}

fn apply_replacement_at_position<S: ImportReplacementStrategy>(
    content: &mut String,
    start: usize,
    original_text: &str,
    import: &ImportInfo,
    strategy: &S,
    file_path: &Path,
    workspace_root: &Path,
    verbose: bool,
) -> bool {
    let end = start + original_text.len();
    
    if let Some(replacement) = strategy.create_replacement(import) {
        // Replace with new import
        content.replace_range(start..end, &replacement);
        
        if verbose {
            let action = ReplacementAction::Replaced {
                from: original_text.to_string(),
                to: replacement,
            };
            println!("{}", strategy.format_verbose_message(import, action, file_path, workspace_root));
        }
    } else {
        // Remove import entirely
        // Find line boundaries to remove the entire line
        let line_start = content[..start].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
        let line_end = content[start..].find('\n').map(|pos| start + pos + 1).unwrap_or(content.len());
        
        content.replace_range(line_start..line_end, "");
        
        if verbose {
            let action = ReplacementAction::Removed {
                original: original_text.to_string(),
            };
            println!("{}", strategy.format_verbose_message(import, action, file_path, workspace_root));
        }
    }
    
    true
}
```

#### **Step 2.4: Update Call Sites**
```rust
// In refactor_engine.rs:
// BEFORE:
replace_target_imports(imports, &self.source_crate_name, workspace_root, self.dry_run, self.verbose)?;

// AFTER:
let strategy = CrossCrateReplacementStrategy {
    source_crate_name: self.source_crate_name.clone(),
};
replace_imports_with_strategy(imports, strategy, workspace_root, self.dry_run, self.verbose)?;

// BEFORE:
replace_crate_imports(imports, workspace_root, self.dry_run, self.verbose)?;

// AFTER:
let strategy = SelfCrateReplacementStrategy;
replace_imports_with_strategy(imports, strategy, workspace_root, self.dry_run, self.verbose)?;
```

### **Phase 3: Legacy Function Removal & Testing** (1 hour)

#### **Step 3.1: Remove Old Functions**
- Remove `analyze_imports()` from `import_analysis.rs`
- Remove `analyze_crate_imports()` from `import_analysis.rs` 
- Remove `print_import_analysis_summary()` from `import_analysis.rs`
- Remove `print_crate_analysis_summary()` from `import_analysis.rs`
- Remove `replace_target_imports()` from `import_replacement.rs`
- Remove `replace_crate_imports()` from `import_replacement.rs`
- Remove `replace_imports_in_file()` from `import_replacement.rs`
- Remove `replace_crate_imports_in_file()` from `import_replacement.rs`

#### **Step 3.2: Compilation & Functional Testing**
```bash
# Compile and check for errors
cargo check

# Test with existing functionality
cargo run -- source_crate target_crate --dry-run --verbose
cargo run -- crate_name --self --dry-run --verbose
cargo run -- --analyze
```

#### **Step 3.3: Update Documentation**
- Update function documentation to reflect new unified approach
- Add examples of using different import contexts
- Document the strategy pattern for future extension

---

## 🚀 Expected Results

### **Lines of Code Reduction**
- **Import Analysis**: 120 lines → 60 lines (**-50% reduction**)
- **Import Replacement**: 240 lines → 120 lines (**-50% reduction**)
- **Total Reduction**: ~150 lines of duplicate logic eliminated

### **Architectural Improvements**
- ✅ **Single source of truth** for import analysis algorithm
- ✅ **Strategy pattern** enables easy addition of new replacement types
- ✅ **Context enum** supports future import contexts (workspace, external crates)
- ✅ **Type-safe parameterization** instead of string-based differentiation
- ✅ **Consistent error handling** across all replacement strategies
- ✅ **Unified verbose logging** with context-specific messages

### **Future Extensibility**
With this design, adding new import types becomes trivial:

```rust
// Future: Workspace-level imports
ImportContext::Workspace { 
    workspace_name: String, 
    crate_name: String 
}

// Future: External crate imports with version constraints
ImportContext::ExternalCrate { 
    crate_name: String, 
    version: String 
}

// Future: Conditional replacement based on feature flags
ConditionalReplacementStrategy { 
    condition: Box<dyn Fn(&ImportInfo) -> bool>,
    inner_strategy: Box<dyn ImportReplacementStrategy> 
}
```

### **Verification Steps**
1. **Functional Equivalence**: All existing CLI commands produce identical results
2. **Performance**: No performance regression (same or better due to reduced code paths)
3. **Maintainability**: Single place to modify import analysis or replacement logic
4. **Extensibility**: Easy to add new import contexts or replacement strategies

---

## 📋 Implementation Checklist

### **Phase 1: Analysis Unification**
- [ ] Create `ImportContext` enum with methods
- [ ] Implement `analyze_imports_unified()` function
- [ ] Implement `print_analysis_summary_unified()` function  
- [ ] Update `refactor_engine.rs` call sites
- [ ] Test cross-crate analysis functionality
- [ ] Test self-crate analysis functionality

### **Phase 2: Replacement Unification**
- [ ] Create `ImportReplacementStrategy` trait
- [ ] Implement `CrossCrateReplacementStrategy` struct
- [ ] Implement `SelfCrateReplacementStrategy` struct
- [ ] Implement `replace_imports_with_strategy()` function
- [ ] Implement `replace_imports_in_file_with_strategy()` function
- [ ] Implement helper functions for replacement logic
- [ ] Update `refactor_engine.rs` call sites
- [ ] Test cross-crate replacement functionality
- [ ] Test self-crate replacement functionality

### **Phase 3: Cleanup & Validation**
- [ ] Remove all legacy functions
- [ ] Clean up unused imports
- [ ] Update documentation
- [ ] Compile without warnings
- [ ] Test all CLI modes with `--dry-run --verbose`
- [ ] Verify analyzer still works: `cargo run -- --analyze`
- [ ] Confirm no functional regression

---

## 🎯 Success Criteria

### **Must Have**
- ✅ Zero functional regression - all existing behavior preserved
- ✅ ~150 lines of duplicate code eliminated
- ✅ Single parameterized function for import analysis
- ✅ Strategy pattern for import replacement
- ✅ Clean compilation without warnings

### **Should Have**  
- ✅ Improved verbose logging with context-specific messages
- ✅ Better error handling consistency
- ✅ Clear documentation of new patterns

### **Nice to Have**
- ✅ Performance improvement due to reduced code paths
- ✅ Foundation for future import context types
- ✅ Example of good Rust architecture patterns

**This task addresses the core analyzer limitation by focusing on the most impactful duplications that can be detected and eliminated through careful architectural improvements rather than trying to enhance the generic duplication detection algorithm.**