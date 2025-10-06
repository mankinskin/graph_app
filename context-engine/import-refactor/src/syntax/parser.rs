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

use crate::{
    analysis::crates::CratePaths,
    syntax::navigator::{UseTreeNavigator, UseTreeItemCollector},
    core::path::ImportPath,
};

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub file_path: PathBuf,
    pub import_path: String,
    pub line_number: usize,
    pub imported_items: Vec<String>,
}

pub struct ImportParser {
    crate_name: String,
}

impl ImportParser {
    pub fn new(source_crate_name: &str) -> Self {
        Self {
            crate_name: source_crate_name.replace('-', "_"), // Convert hyphens to underscores for import matching
        }
    }

    pub fn find_imports_in_crates(
        &self,
        crate_paths: &CratePaths,
    ) -> Result<Vec<ImportInfo>> {
        match crate_paths {
            CratePaths::SelfRefactor { crate_path } =>
                self.find_imports_in_crate(crate_path),
            CratePaths::CrossRefactor {
                source_crate_path: _,
                target_crate_path,
            } => self.find_imports_in_crate(target_crate_path),
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

        let mut visitor =
            ImportVisitor::new(&self.crate_name, file_path.to_path_buf());
        visitor.visit_file(&syntax_tree);

        Ok(visitor.imports)
    }
}

struct ImportVisitor {
    source_crate_name: String,
    file_path: PathBuf,
    imports: Vec<ImportInfo>,
    navigator: UseTreeNavigator,
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
            navigator: UseTreeNavigator,
        }
    }
}

/// Collector that filters for specific crate imports and creates ImportInfo
struct CrateFilteredCollector {
    target_crate: String,
    collected_imports: Vec<(String, Vec<String>)>,
}

impl CrateFilteredCollector {
    fn new(crate_name: &str) -> Self {
        Self {
            target_crate: crate_name.replace('-', "_"),
            collected_imports: Vec::new(),
        }
    }
}

impl UseTreeItemCollector for CrateFilteredCollector {
    fn collect_name(&mut self, name: &str, path: &[String]) {
        if path.is_empty() || path[0] != self.target_crate {
            return;
        }
        
        let full_path = if path.len() == 1 {
            format!("{}::{}", path[0], name)
        } else {
            format!("{}::{}::{}", path[0], path[1..].join("::"), name)
        };
        
        self.collected_imports.push((full_path.clone(), vec![full_path]));
    }

    fn collect_glob(&mut self, path: &[String]) {
        if path.is_empty() || path[0] != self.target_crate {
            return;
        }
        
        let glob_path = format!("{}::*", path.join("::"));
        self.collected_imports.push((glob_path, vec!["*".to_string()]));
    }

    fn collect_rename(&mut self, original: &str, renamed: &str, path: &[String]) {
        if path.is_empty() || path[0] != self.target_crate {
            return;
        }
        
        let full_path = if path.len() == 1 {
            format!("{}::{}", path[0], original)
        } else {
            format!("{}::{}::{}", path[0], path[1..].join("::"), original)
        };
        
        let display_path = format!("{} as {}", full_path, renamed);
        self.collected_imports.push((display_path, vec![full_path]));
    }
}

impl<'ast> Visit<'ast> for ImportVisitor {
    fn visit_item_use(
        &mut self,
        node: &'ast syn::ItemUse,
    ) {
        // Use the navigator to collect imports filtered by crate
        let mut collector = CrateFilteredCollector::new(&self.source_crate_name);
        self.navigator.extract_items(&node.tree, &mut collector);
        
        // Convert collected imports to ImportInfo objects
        for (import_path, imported_items) in collector.collected_imports {
            self.imports.push(ImportInfo {
                file_path: self.file_path.clone(),
                import_path,
                line_number: 0, // We'll rely on string matching instead of line numbers
                imported_items,
            });
        }
    }
}
