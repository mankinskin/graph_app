// This file contains various super:: import patterns for testing normalization

use super::super::config::{load_config, Config}; // Should be normalized to: use crate::config::{Config, load_config};
use super::super::utils::format_string; // Should be normalized to: use crate::utils::format_string;
use crate::utils::validate_input;
use std::collections::HashMap; // Should remain unchanged

// Feature-gated super imports for testing cfg attributes
#[cfg(feature = "advanced")]
use super::super::utils::string_ops::capitalize; // Should be normalized to: use crate::utils::string_ops::capitalize;

#[cfg(feature = "debug")]
use super::super::config::save_config; // Should be normalized to: use crate::config::save_config;

#[cfg(all(feature = "parser", feature = "debug"))]
use super::validator::ValidationResult; // Should be normalized to: use crate::modules::validator::ValidationResult;

pub struct Parser {
    config: Config,
    data: HashMap<String, String>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            config: load_config(),
            data: HashMap::new(),
        }
    }

    pub fn parse(
        &mut self,
        input: &str,
    ) -> Result<String, String> {
        if !validate_input(input) {
            return Err("Invalid input".to_string());
        }

        let formatted = format_string(input);

        // Feature-gated advanced processing
        #[cfg(feature = "advanced")]
        let formatted = { capitalize(&formatted) };

        self.data.insert(input.to_string(), formatted.clone());

        // Feature-gated debug saving
        #[cfg(feature = "debug")]
        if let Err(e) = save_config(&self.config) {
            eprintln!("Debug: Failed to save config: {}", e);
        }

        Ok(formatted)
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    #[cfg(all(feature = "parser", feature = "debug"))]
    pub fn validate_parse_result(
        &self,
        result: &str,
    ) -> ValidationResult {
        if result.is_empty() {
            ValidationResult::Invalid("Empty result".to_string())
        } else {
            ValidationResult::Valid(result.to_string())
        }
    }
}
