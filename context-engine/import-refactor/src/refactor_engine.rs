use crate::item_info::{
    has_macro_export_attribute,
    ItemInfo,
};
use anyhow::{
    bail,
    Context,
    Result,
};
use std::{
    collections::{
        BTreeMap,
        BTreeSet,
    },
    fs,
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};
use syn::{
    parse_file,
    File,
};

use crate::import_parser::ImportInfo;

pub struct RefactorEngine {
    source_crate_name: String,
    dry_run: bool,
    verbose: bool,
}

impl RefactorEngine {
    pub fn new(
        source_crate_name: &str,
        dry_run: bool,
        verbose: bool,
    ) -> Self {
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
        let workspace_root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());

        for import in &imports {
            if import.imported_items.contains(&"*".to_string()) {
                glob_imports += 1;
            } else {
                specific_imports += 1;
                for item in &import.imported_items {
                    if item != "*" {
                        all_imported_items.insert(item.clone());
                        // Track which files import this item with relative paths and simplified imports
                        let canonical_file_path = import
                            .file_path
                            .canonicalize()
                            .unwrap_or_else(|_| import.file_path.clone());

                        let relative_path = canonical_file_path
                            .strip_prefix(&workspace_root)
                            .unwrap_or(&import.file_path);

                        // Make import path relative to source crate
                        let simplified_import = import
                            .import_path
                            .strip_prefix(&format!(
                                "{}::",
                                &self.source_crate_name
                            ))
                            .unwrap_or(&import.import_path);

                        import_types
                            .entry(item.clone())
                            .or_insert_with(Vec::new)
                            .push(format!(
                                "{}:{}",
                                relative_path.display(),
                                simplified_import
                            ));
                    }
                }
            }
        }

        // Enhanced output showing analysis results
        println!("📊 Import Analysis Summary:");
        println!("  • Total imports found: {}", imports.len());
        println!(
            "  • Glob imports (use {}::*): {}",
            self.source_crate_name, glob_imports
        );
        println!("  • Specific imports: {}", specific_imports);
        println!("  • Unique items imported: {}", all_imported_items.len());

        if !all_imported_items.is_empty() {
            println!(
                "\n🔍 Detected imported items from '{}':",
                self.source_crate_name
            );
            for item in &all_imported_items {
                if let Some(files) = import_types.get(item) {
                    println!("  • {}", item);
                    for file_info in files.iter().take(3) {
                        println!("      └─ {}", file_info);
                    }
                    if files.len() > 3 {
                        println!(
                            "      └─ ... and {} more locations",
                            files.len() - 3
                        );
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
        self.update_source_lib_rs(
            source_crate_path,
            &all_imported_items,
            &workspace_root,
        )?;

        // Step 3: Replace imports in target crate files
        self.replace_target_imports(
            target_crate_path,
            imports,
            &workspace_root,
        )?;

        // Always check compilation after refactoring to ensure we didn't break anything
        if !self.dry_run {
            println!("🔧 Checking compilation after modifications...");
            let source_compiles =
                self.check_crate_compilation(source_crate_path)?;
            let target_compiles =
                self.check_crate_compilation(target_crate_path)?;

            if !source_compiles {
                bail!("Source crate failed to compile after refactoring. This indicates a bug in the refactor tool.");
            }

            if !target_compiles {
                bail!("Target crate failed to compile after refactoring. This indicates a bug in the refactor tool.");
            }

            if self.verbose {
                println!("✅ Both source and target crates compile successfully after refactoring");
            }
        }

        Ok(())
    }

    /// Refactor internal crate:: imports within a single crate
    /// This moves crate:: imports to be pub use exports at the crate root level
    pub fn refactor_self_imports(
        &mut self,
        crate_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        // Step 1: Analyze and categorize crate:: imports
        let mut all_imported_items = BTreeSet::new();
        let mut glob_imports = 0;
        let mut specific_imports = 0;
        let mut import_types = std::collections::HashMap::new();
        let workspace_root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());

        for import in &imports {
            if import.imported_items.contains(&"*".to_string()) {
                glob_imports += 1;
            } else {
                specific_imports += 1;
                for item in &import.imported_items {
                    if item != "*" {
                        all_imported_items.insert(item.clone());
                        // Track which files import this item with relative paths and simplified imports
                        let canonical_file_path = import
                            .file_path
                            .canonicalize()
                            .unwrap_or_else(|_| import.file_path.clone());
                        
                        let relative_path = canonical_file_path
                            .strip_prefix(&workspace_root)
                            .unwrap_or(&import.file_path);

                        // Make import path relative to crate (remove "crate::" prefix)
                        let simplified_import = import
                            .import_path
                            .strip_prefix("crate::")
                            .unwrap_or(&import.import_path);

                        import_types
                            .entry(item.clone())
                            .or_insert_with(Vec::new)
                            .push(format!(
                                "{}:{}",
                                relative_path.display(),
                                simplified_import
                            ));
                    }
                }
            }
        }

        // Enhanced output showing analysis results
        println!("📊 Import Analysis Summary:");
        println!("  • Total imports found: {}", imports.len());
        println!("  • Glob imports (use crate::*): {}", glob_imports);
        println!("  • Specific imports: {}", specific_imports);
        println!("  • Unique items imported: {}", all_imported_items.len());

        if !all_imported_items.is_empty() {
            println!("\n🔍 Detected crate:: imports:");
            for item in &all_imported_items {
                if let Some(files) = import_types.get(item) {
                    println!("  • {}", item);
                    for file_info in files.iter().take(3) {
                        println!("      └─ {}", file_info);
                    }
                    if files.len() > 3 {
                        println!(
                            "      └─ ... and {} more locations",
                            files.len() - 3
                        );
                    }
                } else {
                    println!("  • {}", item);
                }
            }
            println!();
        } else if glob_imports > 0 {
            println!("\n💡 Note: Only glob imports (use crate::*) found. No specific items to re-export.");
            println!("   This means the crate is already using the most general import pattern.");
            println!();
        }

        // Step 2: Update the crate's lib.rs with pub use statements  
        self.update_source_lib_rs(
            crate_path,
            &all_imported_items,
            &workspace_root,
        )?;

        // Step 3: Replace crate:: imports in the same crate
        self.replace_crate_imports(crate_path, imports, &workspace_root)?;

        // Always check compilation after refactoring to ensure we didn't break anything
        if !self.dry_run {
            println!("🔧 Checking compilation after modifications...");
            let crate_compiles = self.check_crate_compilation(crate_path)?;

            if !crate_compiles {
                bail!("Crate failed to compile after self-refactoring. This indicates a bug in the refactor tool.");
            }

            if self.verbose {
                println!("✅ Crate compiles successfully after self-refactoring");
            }
        }

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

        let original_content =
            fs::read_to_string(&lib_rs_path).with_context(|| {
                format!("Failed to read {}", lib_rs_path.display())
            })?;

        // Parse the existing file to understand its structure
        let syntax_tree = parse_file(&original_content).with_context(|| {
            format!("Failed to parse {}", lib_rs_path.display())
        })?;

        // Use improved existing pub use collection
        let existing_exports =
            self.collect_existing_exports(&syntax_tree, source_crate_path);

        if self.verbose {
            println!(
                "🔍 Found {} existing exported items:",
                existing_exports.len()
            );
            for item in &existing_exports {
                println!("  • {}", item);
            }
        }

        // Filter out items that are already exported
        let items_to_add: BTreeSet<String> = imported_items
            .iter()
            .filter(|item| {
                let item_name = if item
                    .starts_with(&format!("{}::", &self.source_crate_name))
                {
                    item.strip_prefix(&format!("{}::", &self.source_crate_name))
                        .unwrap_or(item)
                } else {
                    item
                };

                // Check if the final identifier is already exported
                let final_ident =
                    item_name.split("::").last().unwrap_or(item_name);

                if existing_exports.contains(final_ident) {
                    if self.verbose {
                        println!(
                            "  ⚠️  Skipping '{}' - already exported",
                            final_ident
                        );
                    }
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        if items_to_add.is_empty() {
            if self.verbose {
                let relative_path = lib_rs_path
                    .strip_prefix(workspace_root)
                    .unwrap_or(&lib_rs_path);
                println!(
                    "✅ No new pub use statements needed for {} (all items already exported)",
                    relative_path.display()
                );
            }
            return Ok(());
        }

        // Collect conditional items for feature flag grouping
        let (_, conditional_items) =
            self.collect_existing_pub_uses(&syntax_tree);

        // Generate nested pub use statements for the filtered items
        let nested_pub_use = self.generate_nested_pub_use(
            &items_to_add,
            &BTreeSet::new(), // Empty since we already filtered
            &conditional_items,
        );

        if nested_pub_use.is_empty() {
            if self.verbose {
                let relative_path = lib_rs_path
                    .strip_prefix(workspace_root)
                    .unwrap_or(&lib_rs_path);
                println!(
                    "✅ No new pub use statements needed for {}",
                    relative_path.display()
                );
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
        new_content.push_str(&nested_pub_use);

        if self.verbose {
            let relative_path = lib_rs_path
                .strip_prefix(workspace_root)
                .unwrap_or(&lib_rs_path);
            println!(
                "Adding nested pub use statement to {}",
                relative_path.display()
            );
            println!("{}", nested_pub_use.trim());
        }

        if !self.dry_run {
            fs::write(&lib_rs_path, new_content).with_context(|| {
                format!("Failed to write to {}", lib_rs_path.display())
            })?;
        }

        Ok(())
    }

    fn generate_nested_pub_use(
        &self,
        imported_items: &BTreeSet<String>,
        existing_pub_uses: &BTreeSet<String>,
        conditional_items: &BTreeMap<String, Option<syn::Attribute>>,
    ) -> String {
        // Build a tree structure for the imports using a simplified approach
        let mut paths_to_export = Vec::new();
        let mut conditional_exports = Vec::new();

        for item in imported_items {
            if item.starts_with(&format!("{}::", &self.source_crate_name)) {
                let relative_path = item
                    .strip_prefix(&format!("{}::", &self.source_crate_name))
                    .unwrap_or(item);

                // Extract the final identifier to check for conflicts and conditions
                let final_ident =
                    relative_path.split("::").last().unwrap_or(relative_path);

                // Skip if already exists as a use statement
                if existing_pub_uses
                    .contains(&format!("pub use crate::{};", relative_path))
                {
                    if self.verbose {
                        println!(
                            "  ⚠️  Skipping '{}' - already exists as pub use",
                            final_ident
                        );
                    }
                    continue;
                }

                // Skip if this identifier already exists in the crate
                if existing_pub_uses.contains(final_ident) {
                    if self.verbose {
                        println!("  ⚠️  Skipping '{}' - already exists in source crate", final_ident);
                    }
                    continue;
                }

                // Check if this item has conditional compilation
                if let Some(Some(attr)) = conditional_items.get(final_ident) {
                    // This is a conditionally compiled item
                    conditional_exports
                        .push((relative_path.to_string(), attr.clone()));
                    if self.verbose {
                        println!(
                            "  📝 Found conditional item '{}' with cfg: {}",
                            final_ident,
                            quote::quote!(#attr)
                        );
                    }
                } else {
                    paths_to_export.push(relative_path.to_string());
                }
            } else if item != "*"
                && !item.contains(" as ")
                && !item.contains("::")
            {
                // Handle simple items
                if existing_pub_uses.contains(item) {
                    if self.verbose {
                        println!("  ⚠️  Skipping '{}' - already exists in source crate", item);
                    }
                    continue;
                }

                // Check if this item has conditional compilation
                if let Some(Some(attr)) = conditional_items.get(item) {
                    // This is a conditionally compiled item
                    conditional_exports.push((item.clone(), attr.clone()));
                    if self.verbose {
                        println!(
                            "  📝 Found conditional item '{}' with cfg: {}",
                            item,
                            quote::quote!(#attr)
                        );
                    }
                } else {
                    paths_to_export.push(item.clone());
                }
            }
        }

        // Generate the combined result
        let mut result = String::new();

        // First add conditional exports
        for (path, cfg_attr) in conditional_exports {
            result.push_str(&format!(
                "{}\npub use crate::{};\n",
                quote::quote!(#cfg_attr),
                path
            ));
        }

        // Then add unconditional exports if any
        if !paths_to_export.is_empty() {
            // Sort and deduplicate
            paths_to_export.sort();
            paths_to_export.dedup();

            // Generate nested structure for unconditional items
            result.push_str(&self.build_nested_structure(paths_to_export));
        }

        result
    }

    fn build_nested_structure(
        &self,
        paths: Vec<String>,
    ) -> String {
        if paths.is_empty() {
            return String::new();
        }

        // Group paths by their first component
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut direct_exports = Vec::new();

        for path in paths {
            let components: Vec<&str> = path.split("::").collect();
            if components.len() == 1 {
                direct_exports.push(path);
            } else {
                let first = components[0].to_string();
                let rest = components[1..].join("::");
                groups.entry(first).or_default().push(rest);
            }
        }

        let mut result = String::new();
        result.push_str("pub use crate::{\n");

        // Add direct exports first
        for (i, export) in direct_exports.iter().enumerate() {
            result.push_str("    ");
            result.push_str(export);
            if i < direct_exports.len() - 1 || !groups.is_empty() {
                result.push(',');
            }
            result.push('\n');
        }

        // Add grouped exports
        let group_entries: Vec<_> = groups.iter().collect();
        for (i, (module, subpaths)) in group_entries.iter().enumerate() {
            if subpaths.len() == 1 && !subpaths[0].contains("::") {
                // Simple case: module::item
                result.push_str("    ");
                result.push_str(module);
                result.push_str("::");
                result.push_str(&subpaths[0]);
            } else {
                // Complex case: nested structure
                result.push_str("    ");
                result.push_str(module);
                result.push_str("::{\n");

                // Recursively handle subpaths
                let sub_result =
                    Self::build_nested_substructure(subpaths.to_vec(), 2);
                result.push_str(&sub_result);

                result.push_str("    }");
            }

            if i < group_entries.len() - 1 {
                result.push(',');
            }
            result.push('\n');
        }

        result.push_str("};\n");
        result
    }

    fn build_nested_substructure(
        paths: Vec<String>,
        indent_level: usize,
    ) -> String {
        let indent = "    ".repeat(indent_level);
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut direct_exports = Vec::new();

        for path in paths {
            let components: Vec<&str> = path.split("::").collect();
            if components.len() == 1 {
                direct_exports.push(path);
            } else {
                let first = components[0].to_string();
                let rest = components[1..].join("::");
                groups.entry(first).or_default().push(rest);
            }
        }

        let mut result = String::new();

        // Add direct exports
        for (i, export) in direct_exports.iter().enumerate() {
            result.push_str(&indent);
            result.push_str(export);
            if i < direct_exports.len() - 1 || !groups.is_empty() {
                result.push(',');
            }
            result.push('\n');
        }

        // Add grouped exports
        let group_entries: Vec<_> = groups.iter().collect();
        for (i, (module, subpaths)) in group_entries.iter().enumerate() {
            if subpaths.len() == 1 && !subpaths[0].contains("::") {
                result.push_str(&indent);
                result.push_str(module);
                result.push_str("::");
                result.push_str(&subpaths[0]);
            } else {
                result.push_str(&indent);
                result.push_str(module);
                result.push_str("::{\n");

                let sub_result = Self::build_nested_substructure(
                    subpaths.to_vec(),
                    indent_level + 1,
                );
                result.push_str(&sub_result);

                result.push_str(&indent);
                result.push('}');
            }

            if i < group_entries.len() - 1 {
                result.push(',');
            }
            result.push('\n');
        }

        result
    }

    fn collect_existing_pub_uses(
        &self,
        syntax_tree: &File,
    ) -> (BTreeSet<String>, BTreeMap<String, Option<syn::Attribute>>) {
        let mut existing = BTreeSet::new();
        let mut conditional_items = BTreeMap::new();

        for item in &syntax_tree.items {
            // Handle pub use statements separately
            if let syn::Item::Use(use_item) = item {
                if matches!(use_item.vis, syn::Visibility::Public(_)) {
                    let use_str = quote::quote!(#use_item).to_string();
                    existing.insert(use_str);
                }
                continue;
            }

            // Process other public items using the trait
            if item.is_public() {
                if let Some(name) = item.get_identifier() {
                    existing.insert(name.clone());

                    // Check for conditional compilation attributes
                    let cfg_attr =
                        self.extract_cfg_attribute(item.get_attributes());
                    if cfg_attr.is_some() {
                        conditional_items.insert(name, cfg_attr);
                    }
                }
            }
        }

        (existing, conditional_items)
    }

    fn extract_cfg_attribute(
        &self,
        attrs: &[syn::Attribute],
    ) -> Option<syn::Attribute> {
        for attr in attrs {
            if attr.path().is_ident("cfg") {
                return Some(attr.clone());
            }
        }
        None
    }

    fn replace_target_imports(
        &self,
        _target_crate_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        // Group imports by file
        let mut imports_by_file: std::collections::HashMap<
            PathBuf,
            Vec<ImportInfo>,
        > = std::collections::HashMap::new();

        for import in imports {
            imports_by_file
                .entry(import.file_path.clone())
                .or_default()
                .push(import);
        }

        // Process each file
        for (file_path, file_imports) in &imports_by_file {
            self.replace_imports_in_file(
                file_path,
                file_imports.clone(),
                workspace_root,
            )?;
        }

        Ok(())
    }

    fn replace_imports_in_file(
        &self,
        file_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        let original_content =
            fs::read_to_string(file_path).with_context(|| {
                format!("Failed to read {}", file_path.display())
            })?;

        let mut new_content = original_content.clone();
        let mut replacements_made = 0;

        // Sort imports by line number in reverse order to avoid offset issues
        let mut sorted_imports = imports;
        sorted_imports.sort_by(|a, b| b.line_number.cmp(&a.line_number));

        for import in sorted_imports {
            // Look for the import statement in the content
            if let Some(import_start) =
                new_content.find(&format!("use {};", import.import_path))
            {
                // Find the end of the line
                let import_end = new_content[import_start..]
                    .find('\n')
                    .map(|pos| import_start + pos + 1)
                    .unwrap_or(new_content.len());

                // Replace with the new import
                let replacement = format!("use {}::*;", self.source_crate_name);
                new_content.replace_range(
                    import_start..import_end,
                    &format!("{}\n", replacement),
                );
                replacements_made += 1;

                if self.verbose {
                    println!(
                        "  Replaced: use {}; -> {}",
                        import.import_path, replacement
                    );
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
                        if let Some(semicolon_pos) =
                            new_content[pattern_start..].find(';')
                        {
                            let use_end = pattern_start + semicolon_pos + 1;

                            // Find the start of the line
                            let line_start = new_content[..pattern_start]
                                .rfind('\n')
                                .map(|pos| pos + 1)
                                .unwrap_or(0);

                            // Replace the entire use statement
                            let replacement =
                                format!("use {}::*;", self.source_crate_name);
                            let full_replacement = format!(
                                "{}{}",
                                &new_content[line_start..pattern_start],
                                replacement
                            );

                            new_content.replace_range(
                                line_start..use_end,
                                &full_replacement,
                            );
                            replacements_made += 1;
                            found = true;

                            if self.verbose {
                                println!(
                                    "  Replaced pattern: {} -> {}",
                                    pattern, replacement
                                );
                            }
                            break;
                        }
                    }
                }

                if !found && self.verbose {
                    println!(
                        "  Warning: Could not find import to replace: {}",
                        import.import_path
                    );
                }
            }
        }

        if replacements_made > 0 {
            if self.verbose {
                let relative_path =
                    file_path.strip_prefix(workspace_root).unwrap_or(file_path);
                println!(
                    "Made {} replacements in {}",
                    replacements_made,
                    relative_path.display()
                );
            }

            if !self.dry_run {
                fs::write(file_path, new_content).with_context(|| {
                    format!("Failed to write to {}", file_path.display())
                })?;
            }
        }

        Ok(())
    }

    /// Replace crate:: imports within the same crate files (for self-refactor mode)
    fn replace_crate_imports(
        &self,
        crate_path: &Path,
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

        for (file_path, file_imports) in imports_by_file {
            self.replace_crate_imports_in_file(&file_path, file_imports, workspace_root)?;
        }

        Ok(())
    }

    /// Replace crate:: imports in a specific file (for self-refactor mode)
    fn replace_crate_imports_in_file(
        &self,
        file_path: &Path,
        imports: Vec<ImportInfo>,
        workspace_root: &Path,
    ) -> Result<()> {
        let original_content =
            fs::read_to_string(file_path).with_context(|| {
                format!("Failed to read {}", file_path.display())
            })?;

        let mut new_content = original_content.clone();
        let mut replacements_made = 0;

        // Sort imports by line number in reverse order to avoid offset issues
        let mut sorted_imports = imports;
        sorted_imports.sort_by(|a, b| b.line_number.cmp(&a.line_number));

        for import in sorted_imports {
            // For crate:: imports, we want to replace them with direct imports (remove "crate::" prefix)
            // For example: "use crate::module::Item;" becomes "use module::Item;" or just the item name
            
            // Look for the import statement in the content
            if let Some(import_start) =
                new_content.find(&format!("use {};", import.import_path))
            {
                // Find the end of the line
                let import_end = new_content[import_start..]
                    .find('\n')
                    .map(|pos| import_start + pos + 1)
                    .unwrap_or(new_content.len());

                // Since we're adding pub use statements to the root, we can remove the crate:: imports
                // and they'll be available at the root level
                let relative_path = file_path.strip_prefix(workspace_root).unwrap_or(file_path);
                
                if self.verbose {
                    println!(
                        "  Removing crate:: import '{}' from {}",
                        import.import_path,
                        relative_path.display()
                    );
                }

                // Remove the import line entirely since items will be available at root
                new_content.replace_range(import_start..import_end, "");
                replacements_made += 1;
            } else {
                // Try variations of the import statement format
                let patterns = [
                    format!("use {}::{{", import.import_path),
                    format!("use {{\n    {}", import.import_path),
                ];

                let mut found = false;
                for pattern in &patterns {
                    if let Some(pattern_start) = new_content.find(pattern) {
                        if self.verbose {
                            let relative_path = file_path.strip_prefix(workspace_root).unwrap_or(file_path);
                            println!(
                                "  Found crate:: import pattern '{}' in {}",
                                pattern,
                                relative_path.display()
                            );
                        }
                        found = true;
                        break;
                    }
                }

                if !found && self.verbose {
                    println!(
                        "  Warning: Could not find crate:: import to remove: {}",
                        import.import_path
                    );
                }
            }
        }

        if replacements_made > 0 {
            if self.verbose {
                let relative_path =
                    file_path.strip_prefix(workspace_root).unwrap_or(file_path);
                println!(
                    "Removed {} crate:: imports from {}",
                    replacements_made,
                    relative_path.display()
                );
            }

            if !self.dry_run {
                fs::write(file_path, new_content).with_context(|| {
                    format!("Failed to write to {}", file_path.display())
                })?;
            }
        }

        Ok(())
    }

    /// Collect existing exported items from pub use statements and direct definitions
    fn collect_existing_exports(
        &self,
        syntax_tree: &File,
        source_crate_path: &Path,
    ) -> BTreeSet<String> {
        let mut exported_items = BTreeSet::new();

        // Collect from lib.rs (direct definitions and pub use statements)
        self.collect_exports_from_file(syntax_tree, &mut exported_items);

        // Scan all source files for exported macros
        if let Ok(crate_exported_macros) =
            self.scan_crate_for_exported_macros(source_crate_path)
        {
            for macro_name in crate_exported_macros {
                exported_items.insert(macro_name);
            }
        }

        exported_items
    }

    /// Collect exported items from a single file's syntax tree
    fn collect_exports_from_file(
        &self,
        syntax_tree: &File,
        exported_items: &mut BTreeSet<String>,
    ) {
        for item in &syntax_tree.items {
            match item {
                // Collect from pub use statements
                syn::Item::Use(use_item) =>
                    if use_item.is_public() {
                        Self::extract_exported_items_from_use_tree(
                            &use_item.tree,
                            exported_items,
                        );
                    },
                item =>
                    if let Some(ident) = item
                        .is_public()
                        .then(|| item.get_identifier())
                        .flatten()
                    {
                        exported_items.insert(ident);
                    },
            }
        }
    }

    /// Scan all .rs files in the source crate for exported macros
    fn scan_crate_for_exported_macros(
        &self,
        source_crate_path: &Path,
    ) -> Result<BTreeSet<String>> {
        let mut exported_macros = BTreeSet::new();
        let src_dir = source_crate_path.join("src");

        if !src_dir.exists() {
            return Ok(exported_macros);
        }

        self.scan_directory_for_macros(&src_dir, &mut exported_macros)?;

        if self.verbose {
            println!(
                "🔍 Found {} exported macros across source crate: {:?}",
                exported_macros.len(),
                exported_macros
            );
        }

        Ok(exported_macros)
    }

    /// Recursively scan a directory for .rs files and extract exported macros
    fn scan_directory_for_macros(
        &self,
        dir_path: &Path,
        exported_macros: &mut BTreeSet<String>,
    ) -> Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory_for_macros(&path, exported_macros)?;
            } else if let Some(extension) = path.extension() {
                if extension == "rs" {
                    self.scan_file_for_exported_macros(&path, exported_macros)?;
                }
            }
        }

        Ok(())
    }

    /// Scan a single .rs file for exported macros
    fn scan_file_for_exported_macros(
        &self,
        file_path: &Path,
        exported_macros: &mut BTreeSet<String>,
    ) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;

        let syntax_tree: syn::File =
            syn::parse_file(&content).with_context(|| {
                format!("Failed to parse file: {:?}", file_path)
            })?;

        for item in &syntax_tree.items {
            if let syn::Item::Macro(macro_item) = item {
                if has_macro_export_attribute(&macro_item.attrs) {
                    if let Some(ident) = &macro_item.ident {
                        exported_macros.insert(ident.to_string());
                        if self.verbose {
                            println!(
                                "  📝 Found exported macro '{}' in {}",
                                ident,
                                file_path.display()
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Recursively extract exported item names from a use tree
    fn extract_exported_items_from_use_tree(
        tree: &syn::UseTree,
        exported_items: &mut BTreeSet<String>,
    ) {
        match tree {
            syn::UseTree::Path(path) => {
                Self::extract_exported_items_from_use_tree(
                    &path.tree,
                    exported_items,
                );
            },
            syn::UseTree::Name(name) => {
                exported_items.insert(name.ident.to_string());
            },
            syn::UseTree::Rename(rename) => {
                exported_items.insert(rename.rename.to_string());
            },
            syn::UseTree::Glob(_) => {
                exported_items.insert("*".to_string());
            },
            syn::UseTree::Group(group) =>
                for item in &group.items {
                    Self::extract_exported_items_from_use_tree(
                        item,
                        exported_items,
                    );
                },
        }
    }

    /// Check if a crate compiles, providing detailed error information
    fn check_crate_compilation(
        &self,
        crate_path: &Path,
    ) -> Result<bool> {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--quiet")
            .current_dir(crate_path)
            .output()
            .context("Failed to execute cargo check")?;

        if !output.status.success() && self.verbose {
            eprintln!("Compilation failed for crate at {:?}", crate_path);
            eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
            eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(output.status.success())
    }
}
