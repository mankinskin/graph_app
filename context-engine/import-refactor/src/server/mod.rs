// Embedded LLM server functionality (feature-gated)

#[cfg(feature = "embedded-llm")]
pub use self::candle::{CandleServer, ServerResult};

#[cfg(feature = "embedded-llm")]
pub use self::config::{ServerConfig, ModelConfig};

#[cfg(feature = "embedded-llm")]
pub mod candle;

#[cfg(feature = "embedded-llm")]
pub mod config;