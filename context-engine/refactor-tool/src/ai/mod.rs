// AI-powered analysis (feature-gated)

#[cfg(feature = "ai")]
pub use self::client::{AiClient, AiClientFactory, CodeSnippet, SimilarityAnalysis, RefactoringAnalysis};

#[cfg(feature = "ai")]
pub mod client;

#[cfg(feature = "ai")]
pub mod providers;