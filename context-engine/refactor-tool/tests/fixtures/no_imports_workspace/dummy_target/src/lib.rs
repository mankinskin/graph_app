// A crate with no imports from the source crate
// This tests the tool's behavior when there's nothing to refactor

pub fn local_function() -> String {
    "No imports here".to_string()
}

pub struct LocalConfig {
    pub local_setting: i32,
}

// This crate works independently and doesn't import anything from source_crate
