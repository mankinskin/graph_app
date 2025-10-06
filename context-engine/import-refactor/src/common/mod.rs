// Shared utilities and common types

pub use self::error::{Result, Error};
pub use self::path::{relative_path, normalize_path, format_relative_path};
pub use self::format::{format_rust_code, print_summary, print_file_location, print_file_location_with_info};

pub mod error;
pub mod path;
pub mod format;