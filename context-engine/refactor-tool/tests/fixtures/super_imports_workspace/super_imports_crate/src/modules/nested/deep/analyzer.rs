// This file contains complex super:: import chains for testing normalization

use super::super::super::super::config::Config; // Should be normalized to: use crate::config::Config;
use super::super::super::super::root_function; // Should be normalized to: use crate::root_function;
use super::super::super::super::utils::helper_function; // Should be normalized to: use crate::utils::helper_function;
use super::super::processor::Processor; // Should be normalized to: use crate::modules::nested::processor::Processor;

// Feature-gated imports with complex conditions
#[cfg(feature = "analyzer")]
use super::super::super::super::utils::string_ops::reverse_string; // Should be normalized to: use crate::utils::string_ops::reverse_string;

#[cfg(all(feature = "analyzer", feature = "processor"))]
use super::super::super::parser::Parser; // Should be normalized to: use crate::modules::parser::Parser;

#[cfg(any(feature = "debug", feature = "advanced"))]
use super::super::super::super::config::save_config; // Should be normalized to: use crate::config::save_config;

#[cfg(all(feature = "analyzer", not(feature = "basic")))]
use super::super::super::validator::Validator; // Should be normalized to: use crate::modules::validator::Validator;

pub struct DeepAnalyzer {
    processor: Processor,
    config: Config,
}

impl DeepAnalyzer {
    pub fn new() -> Self {
        Self {
            processor: Processor::new(),
            config: Config::new("analyzer", "1.0").with_debug(),
        }
    }

    pub fn analyze(
        &mut self,
        input: &str,
    ) -> Result<AnalysisReport, String> {
        let processed = self.processor.process(input)?;
        let helper_result = helper_function();
        let root_result = root_function();

        Ok(AnalysisReport {
            input: input.to_string(),
            processed,
            helper_result: helper_result.to_string(),
            root_result: root_result.to_string(),
            config_name: self.config.name.clone(),
        })
    }

    #[cfg(feature = "analyzer")]
    pub fn advanced_analyze(
        &mut self,
        input: &str,
    ) -> Result<AnalysisReport, String> {
        let reversed = reverse_string(input);
        let processed = self.processor.process(&reversed)?;
        let helper_result = helper_function();

        Ok(AnalysisReport {
            input: input.to_string(),
            processed,
            helper_result: helper_result.to_string(),
            root_result: "advanced".to_string(),
            config_name: self.config.name.clone(),
        })
    }

    #[cfg(all(feature = "analyzer", feature = "processor"))]
    pub fn combined_analyze(
        &mut self,
        input: &str,
    ) -> Result<String, String> {
        let mut parser = Parser::new();
        let parsed = parser.parse(input)?;
        let processed = self.processor.process(&parsed)?;
        Ok(processed)
    }

    #[cfg(any(feature = "debug", feature = "advanced"))]
    pub fn debug_analyze(
        &mut self,
        input: &str,
    ) -> Result<AnalysisReport, String> {
        let report = self.analyze(input)?;
        save_config(&self.config);
        Ok(report)
    }

    #[cfg(all(feature = "analyzer", not(feature = "basic")))]
    pub fn validated_analyze(
        &mut self,
        input: &str,
    ) -> Result<AnalysisReport, String> {
        let validator = Validator::new(self.config.clone());
        match validator.validate(input) {
            ValidationResult::Valid(_) => self.analyze(input),
            ValidationResult::Invalid(msg) => {
                Err(format!("Input validation failed: {}", msg))
            },
        }
    }
}

#[derive(Debug)]
pub struct AnalysisReport {
    pub input: String,
    pub processed: String,
    pub helper_result: String,
    pub root_result: String,
    pub config_name: String,
}
