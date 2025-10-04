pub fn format_string(input: &str) -> String {
    format!("Formatted: {}", input)
}

pub fn validate_input(input: &str) -> bool {
    !input.is_empty()
}

#[cfg(feature = "extra_utils")]
pub fn extra_utility() -> String {
    "Extra utility function".to_string()
}

#[cfg(test)]
pub fn test_helper() -> bool {
    true
}