use std::path::PathBuf;

use clap::{
    Parser,
    Subcommand,
    command,
};

pub mod index;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Index {
        path: PathBuf,
    },
    #[command(name = "move")]
    MoveModule {
        source: PathBuf,
        target: PathBuf,
    },
}
fn move_module(
    source: PathBuf,
    target: PathBuf,
) {
    println!("{:#?}", source);
    println!("{:#?}", target);
}

fn to_manifest_path(path: PathBuf) -> String {
    if path.ends_with("Cargo.toml") {
        path
    } else {
        path.with_file_name("Cargo.toml")
    }
    .to_string_lossy()
    .into_owned()
}
fn index_workspace(path: PathBuf) {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(to_manifest_path(path))
        .exec()
        .unwrap();
    println!("{:#?}", metadata);
}
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Index { path } => index_workspace(path),
        Command::MoveModule { source, target } => move_module(source, target),
    }
}
