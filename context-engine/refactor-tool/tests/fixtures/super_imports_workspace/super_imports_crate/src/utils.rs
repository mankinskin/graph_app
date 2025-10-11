pub fn format_string(input: &str) -> String {
    format!("Formatted: {}", input.trim())
}

pub fn validate_input(input: &str) -> bool {
    !input.is_empty() && input.len() > 3
}

pub fn helper_function() -> &'static str {
    "This is a helper function"
}

pub mod string_ops {
    pub fn capitalize(s: &str) -> String {
        if s.is_empty() {
            return String::new();
        }
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                first.to_uppercase().collect::<String>() + chars.as_str()
            },
        }
    }

    pub fn reverse_string(s: &str) -> String {
        s.chars().rev().collect()
    }
}
