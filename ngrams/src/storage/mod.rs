//! Storage abstraction for cross-platform file operations
//!
//! This module provides a unified interface for storing and retrieving data
//! that works both on native platforms (using the filesystem) and in WebAssembly
//! (using browser localStorage).

use ciborium::{
    de::Error as DeError,
    ser::Error as SerError,
};
use serde::{de::DeserializeOwned, Serialize};
use std::io;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use native::NativeStorage as PlatformStorage;
#[cfg(target_arch = "wasm32")]
pub use wasm::WasmStorage as PlatformStorage;

/// Error type for storage operations
#[derive(Debug)]
pub enum StorageError {
    /// IO error (native only)
    Io(io::Error),
    /// Serialization error
    Serialize(String),
    /// Deserialization error
    Deserialize(String),
    /// Key/path not found
    NotFound(String),
    /// Storage unavailable (wasm localStorage not available)
    Unavailable(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "IO error: {}", e),
            StorageError::Serialize(e) => write!(f, "Serialization error: {}", e),
            StorageError::Deserialize(e) => write!(f, "Deserialization error: {}", e),
            StorageError::NotFound(key) => write!(f, "Not found: {}", key),
            StorageError::Unavailable(msg) => write!(f, "Storage unavailable: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<io::Error> for StorageError {
    fn from(e: io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl<T: std::fmt::Debug> From<SerError<T>> for StorageError {
    fn from(e: SerError<T>) -> Self {
        StorageError::Serialize(format!("{:?}", e))
    }
}

impl<T: std::fmt::Debug> From<DeError<T>> for StorageError {
    fn from(e: DeError<T>) -> Self {
        StorageError::Deserialize(format!("{:?}", e))
    }
}

/// Trait for storage operations
pub trait Storage {
    /// Store serializable data at the given key
    fn store<T: Serialize>(&self, key: &str, data: &T) -> Result<(), StorageError>;

    /// Load deserializable data from the given key
    fn load<T: DeserializeOwned>(&self, key: &str) -> Result<T, StorageError>;

    /// Check if a key exists
    fn exists(&self, key: &str) -> bool;

    /// Remove data at the given key
    fn remove(&self, key: &str) -> Result<(), StorageError>;
}

/// Get the default storage implementation for the current platform
pub fn get_storage() -> PlatformStorage {
    PlatformStorage::new()
}

/// Convenience function to store data
pub fn store<T: Serialize>(key: &str, data: &T) -> Result<(), StorageError> {
    get_storage().store(key, data)
}

/// Convenience function to load data
pub fn load<T: DeserializeOwned>(key: &str) -> Result<T, StorageError> {
    get_storage().load(key)
}

/// Convenience function to check if key exists
pub fn exists(key: &str) -> bool {
    get_storage().exists(key)
}

/// Convenience function to remove data
pub fn remove(key: &str) -> Result<(), StorageError> {
    get_storage().remove(key)
}
