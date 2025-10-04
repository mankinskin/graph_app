use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{parse_file, File};

use crate::import_parser::ImportInfo;

pub struct RefactorEngine {
    source_crate_name: String,
    dry_run: bool,
    verbose: bool,
}

impl RefactorEngine {
    pub fn new(source_crate_name: &str, dry_run: bool, verbose: bool) -> Self {
        Self {
            source_crate_name: source_crate_name.replace('-', "_"), // Convert hyphens to underscores for import matching
            dry_run,
            verbose,
        }
    }

    pub fn refactor_imports(
        &mut self,
        source_crate_path: &Path,
        target_crate_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        // Step 1: Analyze and categorize imports
        let mut all_imported_items = BTreeSet::new();
        let mut glob_imports = 0;
        let mut specific_imports = 0;
        let mut import_types = std::collections::HashMap::new();
        let workspace_root = workspace_root.canonicalize().unwrap_or_else(|_| workspace_root.to_path_buf());
        
        for import in &imports {
            if import.imported_items.contains(&"*".to_string()) {
                glob_imports += 1;
            } else {
                specific_imports += 1;
                for item in &import.imported_items {
                    if item != "*" {
                        all_imported_items.insert(item.clone());
                        // Track which files import this item with relative paths and simplified imports
                        let relative_path = import.file_path.strip_prefix(&workspace_root)
                            .unwrap_or(&import.file_path);
                        
                        // Make import path relative to source crate
                        let simplified_import = import.import_path.strip_prefix(&format!("{}::", &self.source_crate_name))
                            .unwrap_or(&import.import_path);
                        
                        import_types.entry(item.clone())
                            .or_insert_with(Vec::new)
                            .push(format!("{}:{}", 
                                relative_path.display(),
                                simplified_import));
                    }
                }
            }
        }

        // Enhanced output showing analysis results
        println!("📊 Import Analysis Summary:");
        println!("  • Total imports found: {}", imports.len());
        println!("  • Glob imports (use {}::*): {}", self.source_crate_name, glob_imports);
        println!("  • Specific imports: {}", specific_imports);
        println!("  • Unique items imported: {}", all_imported_items.len());
        
        if !all_imported_items.is_empty() {
            println!("\n🔍 Detected imported items from '{}':", self.source_crate_name);
            for item in &all_imported_items {
                if let Some(files) = import_types.get(item) {
                    println!("  • {}", item);
                    for file_info in files.iter().take(3) {
                        println!("    └─ {}", file_info);
                    }
                    if files.len() > 3 {
                        println!("    └─ ... and {} more locations", files.len() - 3);
                    }
                } else {
                    println!("  • {}", item);
                }
            }
            println!();
        } else if glob_imports > 0 {
            println!("\n💡 Note: Only glob imports (use {}::*) found. No specific items to re-export.", self.source_crate_name);
            println!("   This means the target crate is already using the most general import pattern.");
            println!();
        }

        // Step 2: Update source crate's lib.rs with pub use statements
        self.update_source_lib_rs(source_crate_path, &all_imported_items, &workspace_root)?;

        // Step 3: Replace imports in target crate files
        self.replace_target_imports(target_crate_path, imports, &workspace_root)?;

        Ok(())
    }

    fn update_source_lib_rs(
        &self,
        source_crate_path: &Path,
        imported_items: &BTreeSet<String>,
        workspace_root: &Path,
    ) -> Result<()> {
        let lib_rs_path = source_crate_path.join("src").join("lib.rs");
        
        if !lib_rs_path.exists() {
            if self.verbose {
                println!("Warning: lib.rs not found at {}, skipping pub use additions", lib_rs_path.display());
            }
            return Ok(());
        }

        let original_content = fs::read_to_string(&lib_rs_path)
            .with_context(|| format!("Failed to read {}", lib_rs_path.display()))?;

        // Parse the existing file to understand its structure
        let syntax_tree = parse_file(&original_content)
            .with_context(|| format!("Failed to parse {}", lib_rs_path.display()))?;

        // Collect existing pub use statements to avoid duplicates
        let existing_pub_uses = self.collect_existing_pub_uses(&syntax_tree);

        // Generate new pub use statements
        let mut new_pub_uses = Vec::new();
        for item in imported_items {
            if item.starts_with(&format!("{}::", &self.source_crate_name)) {
                // For full paths like "context_trace::graph::vertex::Child", create "pub use crate::graph::vertex::Child;"
                let relative_path = item.strip_prefix(&format!("{}::", &self.source_crate_name))
                    .unwrap_or(item);
                if !existing_pub_uses.contains(&format!("pub use crate::{};", relative_path)) {
                    new_pub_uses.push(format!("pub use crate::{};", relative_path));
                }
            } else if item != "*" && !item.contains(" as ") && !item.contains("::") {
                // For simple items like "Item", we need to figure out the full path
                if !existing_pub_uses.contains(&format!("pub use crate::{};", item)) {
                    new_pub_uses.push(format!("pub use crate::{};", item));
                }
            }
        }

        if new_pub_uses.is_empty() {
            if self.verbose {
                println!("No new pub use statements needed for {}", lib_rs_path.display());
            }
            return Ok(());
        }

        // Insert new pub use statements at the end of the file
        let mut new_content = original_content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push('\n');
        new_content.push_str("// Auto-generated pub use statements\n");
        for pub_use in &new_pub_uses {
            new_content.push_str(&pub_use);
            new_content.push('\n');
        }

        if self.verbose {
            let relative_path = lib_rs_path.strip_prefix(workspace_root)
                .unwrap_or(&lib_rs_path);
            println!("Adding {} pub use statements to {}", new_pub_uses.len(), relative_path.display());
            for pub_use in &new_pub_uses {
                println!("  {}", pub_use);
            }
        }

        if !self.dry_run {
            fs::write(&lib_rs_path, new_content)
                .with_context(|| format!("Failed to write to {}", lib_rs_path.display()))?;
        }

        Ok(())
    }

    fn collect_existing_pub_uses(&self, syntax_tree: &File) -> BTreeSet<String> {
        let mut existing = BTreeSet::new();
        
        for item in &syntax_tree.items {
            if let syn::Item::Use(use_item) = item {
                if matches!(use_item.vis, syn::Visibility::Public(_)) {
                    // Extract the use path - this is simplified
                    let use_str = quote::quote!(#use_item).to_string();
                    existing.insert(use_str);
                }
            }
        }

        existing
    }

    fn replace_target_imports(
        &self,
        _target_crate_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        // Group imports by file
        let mut imports_by_file: std::collections::HashMap<PathBuf, Vec<ImportInfo>> = 
            std::collections::HashMap::new();

        for import in imports {
            imports_by_file
                .entry(import.file_path.clone())
                .or_insert_with(Vec::new)
                .push(import);
        }

        // Process each file
        for (file_path, file_imports) in imports_by_file {
            self.replace_imports_in_file(&file_path, file_imports, workspace_root)?;
        }

        Ok(())
    }

    fn replace_imports_in_file(
        &self,
        file_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        let original_content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let mut new_content = original_content.clone();
        let mut replacements_made = 0;

        // Sort imports by line number in reverse order to avoid offset issues
        let mut sorted_imports = imports;
        sorted_imports.sort_by(|a, b| b.line_number.cmp(&a.line_number));

        for import in sorted_imports {
            // Look for the import statement in the content
            if let Some(import_start) = new_content.find(&format!("use {};", import.import_path)) {
                // Find the end of the line
                let import_end = new_content[import_start..]
                    .find('\n')
                    .map(|pos| import_start + pos + 1)
                    .unwrap_or(new_content.len());

                // Replace with the new import
                let replacement = format!("use {}::*;", self.source_crate_name);
                new_content.replace_range(import_start..import_end, &format!("{}\n", replacement));
                replacements_made += 1;

                if self.verbose {
                    println!("  Replaced: use {}; -> {}", import.import_path, replacement);
                }
            } else {
                // Try to find a more general pattern
                let patterns = [
                    format!("use {}", import.import_path),
                    format!("use {}::", self.source_crate_name),
                ];

                let mut found = false;
                for pattern in &patterns {
                    if let Some(pattern_start) = new_content.find(pattern) {
                        // Find the semicolon that ends this use statement
                        if let Some(semicolon_pos) = new_content[pattern_start..].find(';') {
                            let use_end = pattern_start + semicolon_pos + 1;
                            
                            // Find the start of the line
                            let line_start = new_content[..pattern_start]
                                .rfind('\n')
                                .map(|pos| pos + 1)
                                .unwrap_or(0);

                            // Replace the entire use statement
                            let replacement = format!("use {}::*;", self.source_crate_name);
                            let full_replacement = format!("{}{}",
                                &new_content[line_start..pattern_start],
                                replacement
                            );
                            
                            new_content.replace_range(line_start..use_end, &full_replacement);
                            replacements_made += 1;
                            found = true;

                            if self.verbose {
                                println!("  Replaced pattern: {} -> {}", pattern, replacement);
                            }
                            break;
                        }
                    }
                }

                if !found && self.verbose {
                    println!("  Warning: Could not find import to replace: {}", import.import_path);
                }
            }
        }

        if replacements_made > 0 {
            if self.verbose {
                let relative_path = file_path.strip_prefix(workspace_root)
                    .unwrap_or(file_path);
                println!("Made {} replacements in {}", replacements_made, relative_path.display());
            }

            if !self.dry_run {
                fs::write(file_path, new_content)
                    .with_context(|| format!("Failed to write to {}", file_path.display()))?;
            }
        }

        Ok(())
    }
}