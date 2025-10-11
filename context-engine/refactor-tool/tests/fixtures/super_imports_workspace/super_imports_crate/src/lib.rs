pub mod config;
pub mod modules;
pub mod utils;

// Feature-gated imports and functions
#[cfg(feature = "basic")]
pub use config::{load_config, Config};

#[cfg(feature = "advanced")]
pub use utils::string_ops;

#[cfg(feature = "debug")]
pub use modules::validator::ValidationResult;

pub fn hello() -> String {
    "Hello from super imports test crate".to_string()
}

pub fn root_function() -> &'static str {
    "This is a root function for testing"
}

#[cfg(feature = "debug")]
pub fn debug_info() -> &'static str {
    "Debug information enabled"
}
