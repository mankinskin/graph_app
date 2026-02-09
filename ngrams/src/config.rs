//! Configuration options for the ngrams crate

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag for cache file writing (disabled by default)
static CACHE_ENABLED: AtomicBool = AtomicBool::new(false);

/// Cache configuration for controlling file-based caching behavior
pub struct CacheConfig;

impl CacheConfig {
    /// Check if cache file writing is enabled
    pub fn is_enabled() -> bool {
        CACHE_ENABLED.load(Ordering::Relaxed)
    }

    /// Enable cache file writing
    pub fn enable() {
        CACHE_ENABLED.store(true, Ordering::Relaxed);
    }

    /// Disable cache file writing
    pub fn disable() {
        CACHE_ENABLED.store(false, Ordering::Relaxed);
    }

    /// Set cache file writing enabled/disabled
    pub fn set_enabled(enabled: bool) {
        CACHE_ENABLED.store(enabled, Ordering::Relaxed);
    }
}
