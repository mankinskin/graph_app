// Embedded LLM server functionality (feature-gated)

#[cfg(feature = "embedded-llm")]
pub use self::candle::CandleServer;

#[cfg(feature = "embedded-llm")]
pub use self::config::ServerConfig;

#[cfg(feature = "embedded-llm")]
pub mod candle;

#[cfg(feature = "embedded-llm")]
pub mod config;