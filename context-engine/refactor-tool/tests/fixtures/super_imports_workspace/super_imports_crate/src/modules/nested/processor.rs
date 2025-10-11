// This file contains more complex super:: imports for testing

use super::super::super::config::{save_config, Config}; // Should be normalized to: use crate::config::{Config, save_config};
use super::super::super::utils::string_ops::capitalize; // Should be normalized to: use crate::utils::string_ops::capitalize;
use super::super::parser::Parser; // Should be normalized to: use crate::modules::parser::Parser;

// Feature-gated complex super imports
#[cfg(feature = "processor")]
use super::super::super::utils::string_ops::reverse_string; // Should be normalized to: use crate::utils::string_ops::reverse_string;

#[cfg(feature = "validator")]
use super::super::validator::{ValidationResult, Validator}; // Should be normalized to: use crate::modules::validator::{Validator, ValidationResult};

#[cfg(all(feature = "processor", feature = "debug"))]
use super::super::super::root_function; // Should be normalized to: use crate::root_function;

#[cfg(not(feature = "basic"))]
use super::super::super::utils::helper_function; // Should be normalized to: use crate::utils::helper_function;

pub struct Processor {
    parser: Parser,
    config: Config,
}

impl Processor {
    pub fn new() -> Self {
        let config = Config::new("processor", "1.0");
        Self {
            parser: Parser::new(),
            config,
        }
    }

    pub fn process(
        &mut self,
        input: &str,
    ) -> Result<String, String> {
        let parsed = self.parser.parse(input)?;
        let capitalized = capitalize(&parsed);

        if let Err(e) = save_config(&self.config) {
            return Err(format!("Config save failed: {}", e));
        }

        Ok(capitalized)
    }

    #[cfg(feature = "processor")]
    pub fn advanced_process(
        &self,
        input: &str,
    ) -> Result<String, String> {
        let reversed = reverse_string(input);
        let processed = capitalize(&reversed);
        Ok(processed)
    }

    #[cfg(feature = "validator")]
    pub fn validate_and_process(
        &mut self,
        input: &str,
    ) -> Result<String, String> {
        let validator = Validator::new(self.config.clone());
        let result = validator.validate(input);
        match result {
            ValidationResult::Valid(_) => self.process(input),
            ValidationResult::Invalid(msg) => Err(msg),
        }
    }

    #[cfg(all(feature = "processor", feature = "debug"))]
    pub fn debug_process(
        &mut self,
        input: &str,
    ) -> Result<String, String> {
        root_function();
        self.process(input)
    }

    #[cfg(not(feature = "basic"))]
    pub fn helper_process(
        &self,
        input: &str,
    ) -> String {
        helper_function(input)
    }
}
