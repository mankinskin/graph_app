use anyhow::{Context, Result};
use std::{
    fs,
    path::Path,
    process::Command,
};
use syn::{File, parse_file};

/// Read and parse a Rust source file
pub fn read_and_parse_file(file_path: &Path) -> Result<(String, File)> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path.display()))?;
    
    let syntax_tree = parse_file(&content)
        .with_context(|| format!("Failed to parse {}", file_path.display()))?;
    
    Ok((content, syntax_tree))
}

/// Write content to a file
pub fn write_file(file_path: &Path, content: &str) -> Result<()> {
    fs::write(file_path, content)
        .with_context(|| format!("Failed to write to {}", file_path.display()))
}

/// Check if a crate compiles, providing detailed error information
pub fn check_crate_compilation(crate_path: &Path, verbose: bool) -> Result<bool> {
    let output = Command::new("cargo")
        .arg("check")
        .arg("--quiet")
        .current_dir(crate_path)
        .output()
        .context("Failed to execute cargo check")?;

    if !output.status.success() && verbose {
        eprintln!("Compilation failed for crate at {:?}", crate_path);
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(output.status.success())
}