// Source crate with basic functionality
// This crate should not be modified when the target has no imports

pub fn source_function() -> String {
    "Source functionality".to_string()
}

pub struct SourceConfig {
    pub setting: bool,
}

pub const SOURCE_CONSTANT: i32 = 100;
