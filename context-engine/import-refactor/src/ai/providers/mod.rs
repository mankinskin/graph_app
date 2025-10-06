// AI provider implementations

#[cfg(feature = "ai")]
pub use self::openai::OpenAiClient;

#[cfg(feature = "ai")]
pub mod openai;

// We'll add other providers (claude, ollama, embedded) here as needed