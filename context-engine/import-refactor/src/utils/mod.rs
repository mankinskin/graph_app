pub mod exports;
pub mod file_operations;
pub mod import_analysis;
pub mod import_replacement;
pub mod pub_use_generation;
pub mod pub_use_merger;

// Unified modules for improved code reuse
pub mod analyzer_cli;
pub mod common;
pub mod duplication_analyzer;
pub mod refactoring_analyzer;

// AI-powered analysis
pub mod ai_client;

// Local LLM management
pub mod ollama_manager;

// Embedded LLM support (feature-gated)
#[cfg(feature = "embedded-llm")]
pub mod candle_server;

#[cfg(feature = "embedded-llm")]
pub mod candle_config;

#[cfg(not(feature = "embedded-llm"))]
pub mod candle_server {
    use super::ai_client::{
        AiClient,
        CodeSnippet,
        RefactoringAnalysis,
        SimilarityAnalysis,
    };
    use anyhow::Result;

    pub struct CandleServer;

    impl CandleServer {
        pub async fn new() -> Result<Self> {
            Err(anyhow::anyhow!(
                "Embedded LLM feature not compiled. Rebuild with --features embedded-llm to enable."
            ))
        }
    }

    #[async_trait::async_trait]
    impl AiClient for CandleServer {
        async fn analyze_code_similarity(
            &self,
            _code_snippets: &[CodeSnippet],
            _analysis_prompt: &str,
        ) -> Result<SimilarityAnalysis> {
            Err(anyhow::anyhow!("Embedded LLM not available"))
        }

        async fn suggest_refactoring(
            &self,
            _code_context: &str,
            _analysis_prompt: &str,
        ) -> Result<RefactoringAnalysis> {
            Err(anyhow::anyhow!("Embedded LLM not available"))
        }
    }
}
