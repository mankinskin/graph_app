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

// New nested modules for utilities
pub mod string_ops {
    pub fn reverse_string(s: &str) -> String {
        s.chars().rev().collect()
    }

    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) =>
                first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    // Nested module for advanced string operations
    pub mod encoding {
        pub fn base64_encode(input: &str) -> String {
            // Simple mock implementation
            format!("base64:{}", input)
        }

        pub fn url_encode(input: &str) -> String {
            input.replace(' ', "%20").replace('&', "%26")
        }

        pub struct Encoder {
            pub algorithm: String,
        }
    }

    // Another nested module
    pub mod parsing {
        pub fn extract_numbers(s: &str) -> Vec<i32> {
            s.chars()
                .filter(|c| c.is_ascii_digit())
                .map(|c| c.to_digit(10).unwrap() as i32)
                .collect()
        }

        pub struct Parser {
            pub delimiter: char,
        }
    }
}

pub mod file_ops {
    pub fn get_extension(filename: &str) -> Option<&str> {
        filename.split('.').last()
    }

    pub fn join_path(parts: &[&str]) -> String {
        parts.join("/")
    }

    // Nested file operations
    pub mod compression {
        pub fn estimate_compression_ratio(size: usize) -> f64 {
            // Mock implementation
            0.7
        }

        pub struct Compressor {
            pub level: u8,
        }
    }

    pub mod metadata {
        pub struct FileInfo {
            pub size: u64,
            pub created: String,
            pub modified: String,
        }

        pub fn get_size_category(size: u64) -> &'static str {
            match size {
                0..=1024 => "small",
                1025..=1048576 => "medium",
                _ => "large",
            }
        }
    }
}
