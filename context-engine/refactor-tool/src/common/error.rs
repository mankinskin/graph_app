//! Common error types and handling

use anyhow;

/// Common result type used throughout the crate
pub type Result<T> = anyhow::Result<T>;

/// Common error type
pub type Error = anyhow::Error;