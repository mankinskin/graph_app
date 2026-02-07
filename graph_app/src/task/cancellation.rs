//! Cross-platform cancellation handle.

#[cfg(target_arch = "wasm32")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[cfg(not(target_arch = "wasm32"))]
use tokio_util::sync::CancellationToken;

/// A handle to cancel a running task.
///
/// This provides a unified interface for task cancellation across platforms:
/// - **Native**: Wraps a `tokio_util::sync::CancellationToken`
/// - **Wasm**: Uses an `Arc<AtomicBool>` flag
///
/// # Example
///
/// ```rust,ignore
/// let handle = CancellationHandle::new();
///
/// // In another thread/task
/// if handle.is_cancelled() {
///     return; // Early exit
/// }
///
/// // Request cancellation
/// handle.cancel();
/// ```
#[derive(Clone)]
pub(crate) struct CancellationHandle {
    #[cfg(not(target_arch = "wasm32"))]
    token: CancellationToken,
    #[cfg(target_arch = "wasm32")]
    flag: Arc<AtomicBool>,
}

impl CancellationHandle {
    /// Create a new cancellation handle.
    pub(crate) fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            token: CancellationToken::new(),
            #[cfg(target_arch = "wasm32")]
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Request cancellation.
    ///
    /// This is a cooperative cancellation - the task must check
    /// `is_cancelled()` and exit gracefully.
    pub(crate) fn cancel(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.token.cancel();
        #[cfg(target_arch = "wasm32")]
        self.flag.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation has been requested.
    pub(crate) fn is_cancelled(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return self.token.is_cancelled();
        #[cfg(target_arch = "wasm32")]
        return self.flag.load(Ordering::SeqCst);
    }

    /// Get the native cancellation token (native only).
    ///
    /// This is useful for integrating with tokio's cancellation utilities.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Get the wasm cancellation flag (wasm only).
    ///
    /// This can be shared with Web Workers for cancellation.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn flag(&self) -> Arc<AtomicBool> {
        self.flag.clone()
    }

    /// Create a child handle that will be cancelled when this handle is cancelled.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn child(&self) -> Self {
        Self {
            token: self.token.child_token(),
        }
    }

    /// Create a child handle (wasm version - just clones the flag).
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn child(&self) -> Self {
        self.clone()
    }
}

impl Default for CancellationHandle {
    fn default() -> Self {
        Self::new()
    }
}

// Note: CancellationHandle is Send + Sync because:
// - Native: CancellationToken is Send + Sync
// - Wasm: Arc<AtomicBool> is Send + Sync
