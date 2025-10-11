// This file contains super:: imports to test normalization functionality

use super::super::config::Config; // Should be normalized to: use crate::config::Config;
use super::super::utils::{helper_function, validate_input}; // Should be normalized to: use crate::utils::{validate_input, helper_function};
use std::fmt;

// Feature-gated super imports
#[cfg(feature = "advanced")]
use super::super::utils::string_ops::{capitalize, reverse_string}; // Should be normalized to: use crate::utils::string_ops::{capitalize, reverse_string};

#[cfg(feature = "network")]
use super::super::config::save_config; // Should be normalized to: use crate::config::save_config;

#[cfg(any(feature = "validator", feature = "debug"))]
use super::parser::Parser; // Should be normalized to: use crate::modules::parser::Parser;

pub struct Validator {
    config: Config,
}

impl Validator {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn validate(
        &self,
        input: &str,
    ) -> ValidationResult {
        if !validate_input(input) {
            return ValidationResult::Invalid("Input too short".to_string());
        }

        if self.config.debug {
            println!("Validating: {}", input);
        }

        let mut result = helper_function().to_string();

        // Advanced feature processing
        #[cfg(feature = "advanced")]
        {
            result = capitalize(&result);
            if input.len() > 10 {
                result = reverse_string(&result);
            }
        }

        // Network feature config saving
        #[cfg(feature = "network")]
        if let Err(e) = save_config(&self.config) {
            return ValidationResult::Invalid(format!(
                "Network save failed: {}",
                e
            ));
        }

        ValidationResult::Valid(result)
    }

    #[cfg(any(feature = "validator", feature = "debug"))]
    pub fn validate_with_parser(
        &self,
        input: &str,
    ) -> ValidationResult {
        let mut parser = Parser::new();
        match parser.parse(input) {
            Ok(parsed) => {
                ValidationResult::Valid(format!("Parsed: {}", parsed))
            },
            Err(e) => ValidationResult::Invalid(format!("Parse error: {}", e)),
        }
    }
}

#[derive(Debug)]
pub enum ValidationResult {
    Valid(String),
    Invalid(String),
}

impl fmt::Display for ValidationResult {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            ValidationResult::Valid(msg) => write!(f, "Valid: {}", msg),
            ValidationResult::Invalid(msg) => write!(f, "Invalid: {}", msg),
        }
    }
}
