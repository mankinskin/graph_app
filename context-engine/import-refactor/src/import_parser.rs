use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{visit::Visit, UseTree, spanned::Spanned};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub file_path: PathBuf,
    pub import_path: String,
    pub line_number: usize,
    pub full_use_statement: String,
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

    pub fn find_imports_in_crate(&self, crate_path: &Path) -> Result<Vec<ImportInfo>> {
        let mut imports = Vec::new();
        let src_path = crate_path.join("src");

        if !src_path.exists() {
            return Ok(imports);
        }

        for entry in WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "rs") {
                let file_imports = self.parse_file_imports(path)?;
                imports.extend(file_imports);
            }
        }

        Ok(imports)
    }

    fn parse_file_imports(&self, file_path: &Path) -> Result<Vec<ImportInfo>> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let syntax_tree = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse Rust file: {}", file_path.display()))?;

        let mut visitor = ImportVisitor::new(&self.source_crate_name, file_path.to_path_buf());
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
    fn new(source_crate_name: &str, file_path: PathBuf) -> Self {
        Self {
            source_crate_name: source_crate_name.to_string(),
            file_path,
            imports: Vec::new(),
        }
    }

    fn extract_use_info(&self, use_tree: &UseTree) -> Option<(String, Vec<String>)> {
        match use_tree {
            UseTree::Path(path) => {
                let ident = path.ident.to_string();
                if ident == self.source_crate_name {
                    if let Some((rest, items)) = self.extract_use_info(&path.tree) {
                        Some((format!("{}::{}", ident, rest), items))
                    } else {
                        Some((ident, vec![]))
                    }
                } else {
                    None
                }
            }
            UseTree::Name(name) => {
                Some((name.ident.to_string(), vec![name.ident.to_string()]))
            }
            UseTree::Rename(rename) => {
                Some((
                    rename.ident.to_string(),
                    vec![format!("{} as {}", rename.ident, rename.rename)],
                ))
            }
            UseTree::Glob(_) => {
                Some(("*".to_string(), vec!["*".to_string()]))
            }
            UseTree::Group(group) => {
                let mut all_items = Vec::new();
                let mut paths = Vec::new();
                
                for item in &group.items {
                    if let Some((path, items)) = self.extract_use_info(item) {
                        paths.push(path);
                        all_items.extend(items);
                    }
                }
                
                if !paths.is_empty() {
                    Some((format!("{{{}}}", paths.join(", ")), all_items))
                } else {
                    None
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for ImportVisitor {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        if let Some((import_path, imported_items)) = self.extract_use_info(&node.tree) {
            let full_statement = quote::quote!(#node).to_string();
            
            self.imports.push(ImportInfo {
                file_path: self.file_path.clone(),
                import_path,
                line_number: 0, // We'll rely on string matching instead of line numbers
                full_use_statement: full_statement,
                imported_items,
            });
        }
    }
}