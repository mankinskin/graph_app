// File system operations and workspace management

pub use self::files::{read_and_parse_file, write_file, check_crate_compilation, check_crates_compilation, CompileResults};
pub use self::workspace::{WorkspaceRoot, find_crate_root};

pub mod files;
pub mod workspace;