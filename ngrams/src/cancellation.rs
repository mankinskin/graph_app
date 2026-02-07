//! Cancellation abstraction for cross-platform async cancellation
//!
//! This module provides a unified interface for checking cancellation
//! that works both on native platforms (using tokio_util::sync::CancellationToken)
//! and in WebAssembly (using Arc<AtomicBool>).

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[cfg(not(target_arch = "wasm32"))]
use tokio_util::sync::CancellationToken;

/// A cancellation handle that can be checked for cancellation
pub(crate) trait Cancellable {
    /// Check if cancellation has been requested
    fn is_cancelled(&self) -> bool;
}

/// Native cancellation using tokio's CancellationToken
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub(crate) struct NativeCancellation {
    token: CancellationToken,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeCancellation {
    pub(crate) fn new(token: CancellationToken) -> Self {
        Self { token }
    }
    
    pub(crate) fn token(&self) -> &CancellationToken {
        &self.token
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Cancellable for NativeCancellation {
    fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<CancellationToken> for NativeCancellation {
    fn from(token: CancellationToken) -> Self {
        Self::new(token)
    }
}

/// Wasm cancellation using Arc<AtomicBool>
#[derive(Debug, Clone)]
pub(crate) struct WasmCancellation {
    cancelled: Arc<AtomicBool>,
}

impl WasmCancellation {
    pub(crate) fn new(cancelled: Arc<AtomicBool>) -> Self {
        Self { cancelled }
    }
    
    pub(crate) fn flag(&self) -> &Arc<AtomicBool> {
        &self.cancelled
    }
}

impl Cancellable for WasmCancellation {
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl From<Arc<AtomicBool>> for WasmCancellation {
    fn from(cancelled: Arc<AtomicBool>) -> Self {
        Self::new(cancelled)
    }
}

/// Platform-appropriate cancellation type
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type PlatformCancellation = NativeCancellation;

#[cfg(target_arch = "wasm32")]
pub(crate) type PlatformCancellation = WasmCancellation;

/// Unified cancellation enum that can hold either type
#[derive(Debug, Clone)]
pub enum Cancellation {
    #[cfg(not(target_arch = "wasm32"))]
    Native(NativeCancellation),
    Wasm(WasmCancellation),
    /// No cancellation support
    None,
}

impl Cancellation {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn native(token: CancellationToken) -> Self {
        Self::Native(NativeCancellation::new(token))
    }
    
    pub(crate) fn wasm(cancelled: Arc<AtomicBool>) -> Self {
        Self::Wasm(WasmCancellation::new(cancelled))
    }
    
    pub(crate) fn none() -> Self {
        Self::None
    }
}

impl Cancellable for Cancellation {
    fn is_cancelled(&self) -> bool {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Cancellation::Native(c) => c.is_cancelled(),
            Cancellation::Wasm(c) => c.is_cancelled(),
            Cancellation::None => false,
        }
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::None
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<CancellationToken> for Cancellation {
    fn from(token: CancellationToken) -> Self {
        Self::native(token)
    }
}

impl From<Arc<AtomicBool>> for Cancellation {
    fn from(cancelled: Arc<AtomicBool>) -> Self {
        Self::wasm(cancelled)
    }
}
