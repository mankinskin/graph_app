use anyhow::Result;
use std::path::{Path, PathBuf};

/// Represents a workspace root
pub struct WorkspaceRoot {
    path: PathBuf,
}

impl WorkspaceRoot {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Find the root of a crate by looking for Cargo.toml
pub fn find_crate_root(start_path: impl AsRef<Path>) -> Result<PathBuf> {
    let mut current = start_path.as_ref();
    
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(current.to_path_buf());
        }
        
        match current.parent() {
            Some(parent) => current = parent,
            None => {
                return Err(anyhow::anyhow!(
                    "Could not find Cargo.toml in {} or any parent directory",
                    start_path.as_ref().display()
                ));
            }
        }
    }
}