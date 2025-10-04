use anyhow::{
    Context,
    Result,
};
use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};
use syn::{
    visit::Visit,
    UseTree,
};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub file_path: PathBuf,
    pub import_path: String,
    pub line_number: usize,
    pub imported_items: Vec<String>,
}

pub struct ImportParser {
    source_crate_name: String,
}

impl ImportParser {
    pub fn new(source_crate_name: &str) -> Self {
        Self {
            source_crate_name: source_crate_name.replace('-', "_"), // Convert hyphens to underscores for import matching
        }
    }

    pub fn find_imports_in_crate(
        &self,
        crate_path: &Path,
    ) -> Result<Vec<ImportInfo>> {
        let mut imports = Vec::new();
        let src_path = crate_path.join("src");

        if !src_path.exists() {
            return Ok(imports);
        }

        for entry in WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                let file_imports = self.parse_file_imports(path)?;
                imports.extend(file_imports);
            }
        }

        Ok(imports)
    }

    fn parse_file_imports(
        &self,
        file_path: &Path,
    ) -> Result<Vec<ImportInfo>> {
        let content = fs::read_to_string(file_path).with_context(|| {
            format!("Failed to read file: {}", file_path.display())
        })?;

        let syntax_tree = syn::parse_file(&content).with_context(|| {
            format!("Failed to parse Rust file: {}", file_path.display())
        })?;

        let mut visitor = ImportVisitor::new(
            &self.source_crate_name,
            file_path.to_path_buf(),
        );
        visitor.visit_file(&syntax_tree);

        Ok(visitor.imports)
    }
}

struct ImportVisitor {
    source_crate_name: String,
    file_path: PathBuf,
    imports: Vec<ImportInfo>,
}

impl ImportVisitor {
    fn new(
        source_crate_name: &str,
        file_path: PathBuf,
    ) -> Self {
        Self {
            source_crate_name: source_crate_name.to_string(),
            file_path,
            imports: Vec::new(),
        }
    }

    fn extract_use_info(
        &self,
        use_tree: &UseTree,
    ) -> Option<(String, Vec<String>)> {
        self.extract_use_info_recursive(use_tree, "")
    }

    fn extract_use_info_recursive(
        &self,
        use_tree: &UseTree,
        current_path: &str,
    ) -> Option<(String, Vec<String>)> {
        match use_tree {
            UseTree::Path(path) => {
                let ident = path.ident.to_string();
                let new_path = if current_path.is_empty() {
                    ident.clone()
                } else {
                    format!("{}::{}", current_path, ident)
                };

                // Check if we've found our source crate
                if current_path.is_empty() && ident == self.source_crate_name {
                    // This is the root of our source crate, continue parsing
                    self.extract_use_info_recursive(&path.tree, &ident)
                } else if current_path.starts_with(&self.source_crate_name) {
                    // We're already inside the source crate path, continue
                    self.extract_use_info_recursive(&path.tree, &new_path)
                } else {
                    None
                }
            },
            UseTree::Name(name) => {
                if current_path.starts_with(&self.source_crate_name) {
                    let ident = name.ident.to_string();
                    let full_path = format!("{}::{}", current_path, ident);
                    Some((full_path.clone(), vec![full_path]))
                } else {
                    None
                }
            },
            UseTree::Rename(rename) => {
                if current_path.starts_with(&self.source_crate_name) {
                    let ident = rename.ident.to_string();
                    let alias = rename.rename.to_string();
                    let full_path = format!("{}::{}", current_path, ident);
                    Some((
                        format!("{} as {}", full_path, alias),
                        vec![full_path],
                    ))
                } else {
                    None
                }
            },
            UseTree::Glob(_) => {
                if current_path.starts_with(&self.source_crate_name) {
                    let glob_path = format!("{}::*", current_path);
                    Some((glob_path, vec!["*".to_string()]))
                } else {
                    None
                }
            },
            UseTree::Group(group) => {
                if current_path.starts_with(&self.source_crate_name) {
                    let mut all_items = Vec::new();
                    let mut all_paths = Vec::new();

                    for item in &group.items {
                        if let Some((path, items)) =
                            self.extract_use_info_recursive(item, current_path)
                        {
                            all_paths.push(path);
                            all_items.extend(items);
                        }
                    }

                    if !all_paths.is_empty() {
                        Some((
                            format!(
                                "{}::{{{}}}",
                                current_path,
                                all_paths.join(", ")
                            ),
                            all_items,
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        }
    }
}

impl<'ast> Visit<'ast> for ImportVisitor {
    fn visit_item_use(
        &mut self,
        node: &'ast syn::ItemUse,
    ) {
        if let Some((import_path, imported_items)) =
            self.extract_use_info(&node.tree)
        {
            //let full_statement = quote::quote!(#node).to_string();

            self.imports.push(ImportInfo {
                file_path: self.file_path.clone(),
                import_path,
                line_number: 0, // We'll rely on string matching instead of line numbers
                imported_items,
            });
        }
    }
}
