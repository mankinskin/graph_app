use anyhow::{
    bail,
    Context,
    Result,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{
        Path,
        PathBuf,
    },
};

#[derive(Deserialize)]
struct CargoToml {
    package: Option<Package>,
    workspace: Option<Workspace>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

#[derive(Deserialize)]
struct Workspace {
    members: Vec<String>,
}

pub struct CrateAnalyzer {
    workspace_root: PathBuf,
    crate_paths: HashMap<String, PathBuf>,
}

impl CrateAnalyzer {
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let mut analyzer = Self {
            workspace_root: workspace_root.to_path_buf(),
            crate_paths: HashMap::new(),
        };
        analyzer.discover_crates()?;
        Ok(analyzer)
    }

    fn discover_crates(&mut self) -> Result<()> {
        let workspace_toml_path = self.workspace_root.join("Cargo.toml");

        if !workspace_toml_path.exists() {
            bail!(
                "No Cargo.toml found in workspace root: {}",
                self.workspace_root.display()
            );
        }

        let workspace_toml_content = fs::read_to_string(&workspace_toml_path)
            .with_context(|| {
            format!("Failed to read {}", workspace_toml_path.display())
        })?;

        let workspace_toml: CargoToml = toml::from_str(&workspace_toml_content)
            .with_context(|| "Failed to parse workspace Cargo.toml")?;

        // If this is a workspace, process members
        if let Some(workspace) = workspace_toml.workspace {
            for member in workspace.members {
                let member_path = self.workspace_root.join(&member);
                self.process_crate_directory(&member_path)?;
            }
        }

        // Also check if the root has a package (can be both workspace and package)
        if workspace_toml.package.is_some() {
            let workspace_root = self.workspace_root.clone();
            self.process_crate_directory(&workspace_root)?;
        }

        // Additionally, scan for any crates not listed in workspace
        self.scan_for_additional_crates()?;

        Ok(())
    }

    fn process_crate_directory(
        &mut self,
        crate_path: &Path,
    ) -> Result<()> {
        let cargo_toml_path = crate_path.join("Cargo.toml");

        if !cargo_toml_path.exists() {
            return Ok(()); // Skip directories without Cargo.toml
        }

        let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| {
                format!("Failed to read {}", cargo_toml_path.display())
            })?;

        let cargo_toml: CargoToml = toml::from_str(&cargo_toml_content)
            .with_context(|| {
                format!("Failed to parse {}", cargo_toml_path.display())
            })?;

        if let Some(package) = cargo_toml.package {
            self.crate_paths
                .insert(package.name, crate_path.to_path_buf());
        }

        Ok(())
    }

    fn scan_for_additional_crates(&mut self) -> Result<()> {
        use walkdir::WalkDir;

        for entry in WalkDir::new(&self.workspace_root)
            .max_depth(2) // Don't go too deep to avoid target directories
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir()
                && path.join("Cargo.toml").exists()
                && path != self.workspace_root
            {
                // Skip target directories
                if path.file_name().map_or(false, |name| name == "target") {
                    continue;
                }

                self.process_crate_directory(path)?;
            }
        }

        Ok(())
    }

    pub fn find_crate(
        &self,
        crate_name: &str,
    ) -> Result<PathBuf> {
        self.crate_paths.get(crate_name).cloned().with_context(|| {
            let available_crates: Vec<String> =
                self.crate_paths.keys().cloned().collect();
            format!(
                "Crate '{}' not found in workspace. Available crates: {}",
                crate_name,
                available_crates.join(", ")
            )
        })
    }

    pub fn list_crates(&self) -> Vec<&String> {
        self.crate_paths.keys().collect()
    }
}
