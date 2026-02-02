//! WebAssembly localStorage-based storage implementation

use super::{Storage, StorageError};
use serde::{de::DeserializeOwned, Serialize};

/// Storage key prefix for ngrams data in localStorage
const STORAGE_PREFIX: &str = "ngrams_storage_";

/// WebAssembly localStorage-based storage
#[derive(Debug, Clone, Default)]
pub struct WasmStorage {
    prefix: String,
}

impl WasmStorage {
    /// Create a new wasm storage with the default prefix
    pub fn new() -> Self {
        Self {
            prefix: STORAGE_PREFIX.to_string(),
        }
    }

    /// Create a new wasm storage with a custom prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    /// Get the full storage key for a key
    fn make_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// Get the localStorage object
    fn get_local_storage(&self) -> Result<web_sys::Storage, StorageError> {
        let window = web_sys::window()
            .ok_or_else(|| StorageError::Unavailable("No window object".to_string()))?;
        
        window
            .local_storage()
            .map_err(|_| StorageError::Unavailable("localStorage access denied".to_string()))?
            .ok_or_else(|| StorageError::Unavailable("localStorage not available".to_string()))
    }
}

impl Storage for WasmStorage {
    fn store<T: Serialize>(&self, key: &str, data: &T) -> Result<(), StorageError> {
        let storage = self.get_local_storage()?;
        let storage_key = self.make_key(key);
        
        web_sys::console::log_1(&format!("Write to localStorage: {}", storage_key).into());
        
        // Serialize to CBOR bytes, then base64 encode for localStorage
        let mut bytes = Vec::new();
        ciborium::into_writer(data, &mut bytes)?;
        
        // Base64 encode the bytes for storage as a string
        let encoded = base64_encode(&bytes);
        
        storage
            .set_item(&storage_key, &encoded)
            .map_err(|_| StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write to localStorage",
            )))?;
        
        Ok(())
    }

    fn load<T: DeserializeOwned>(&self, key: &str) -> Result<T, StorageError> {
        let storage = self.get_local_storage()?;
        let storage_key = self.make_key(key);
        
        web_sys::console::log_1(&format!("Read from localStorage: {}", storage_key).into());
        
        let encoded = storage
            .get_item(&storage_key)
            .map_err(|_| StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to read from localStorage",
            )))?
            .ok_or_else(|| StorageError::NotFound(key.to_string()))?;
        
        // Base64 decode then deserialize from CBOR
        let bytes = base64_decode(&encoded)
            .map_err(|e| StorageError::Deserialize(format!("Base64 decode error: {}", e)))?;
        
        let data = ciborium::from_reader(&bytes[..])?;
        
        Ok(data)
    }

    fn exists(&self, key: &str) -> bool {
        self.get_local_storage()
            .and_then(|storage| {
                storage
                    .get_item(&self.make_key(key))
                    .map_err(|_| StorageError::Unavailable("Failed to check key".to_string()))
            })
            .map(|v| v.is_some())
            .unwrap_or(false)
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        let storage = self.get_local_storage()?;
        let storage_key = self.make_key(key);
        
        storage
            .remove_item(&storage_key)
            .map_err(|_| StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to remove from localStorage",
            )))?;
        
        Ok(())
    }
}

/// Simple base64 encoding (no external dependency needed)
fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = String::new();
    let mut i = 0;
    
    while i < bytes.len() {
        let b0 = bytes[i] as usize;
        let b1 = bytes.get(i + 1).copied().unwrap_or(0) as usize;
        let b2 = bytes.get(i + 2).copied().unwrap_or(0) as usize;
        
        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        
        if i + 1 < bytes.len() {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        
        if i + 2 < bytes.len() {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
        
        i += 3;
    }
    
    result
}

/// Simple base64 decoding
fn base64_decode(s: &str) -> Result<Vec<u8>, &'static str> {
    fn decode_char(c: char) -> Result<u8, &'static str> {
        match c {
            'A'..='Z' => Ok(c as u8 - b'A'),
            'a'..='z' => Ok(c as u8 - b'a' + 26),
            '0'..='9' => Ok(c as u8 - b'0' + 52),
            '+' => Ok(62),
            '/' => Ok(63),
            '=' => Ok(0),
            _ => Err("Invalid base64 character"),
        }
    }
    
    let chars: Vec<char> = s.chars().collect();
    if chars.len() % 4 != 0 {
        return Err("Invalid base64 length");
    }
    
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < chars.len() {
        let b0 = decode_char(chars[i])?;
        let b1 = decode_char(chars[i + 1])?;
        let b2 = decode_char(chars[i + 2])?;
        let b3 = decode_char(chars[i + 3])?;
        
        result.push((b0 << 2) | (b1 >> 4));
        
        if chars[i + 2] != '=' {
            result.push(((b1 & 0x0f) << 4) | (b2 >> 2));
        }
        
        if chars[i + 3] != '=' {
            result.push(((b2 & 0x03) << 6) | b3);
        }
        
        i += 4;
    }
    
    Ok(result)
}
