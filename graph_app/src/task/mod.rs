//! Unified task abstraction for cross-platform async task management.
//!
//! This module provides a common interface for spawning and cancelling tasks
//! that works on both native (tokio) and wasm (web workers).
//!
//! # Architecture
//!
//! The task system uses a trait-based abstraction to unify task execution across platforms:
//!
//! - **[`TaskExecutor`]**: Core trait for spawning async and blocking tasks
//! - **[`CancellationHandle`]**: Cross-platform cancellation token
//! - **[`TaskHandle`]**: Handle to track and control spawned tasks
//! - **[`TaskResult`]**: Result of task execution
//!
//! ## Platform Implementations
//!
//! - **Native**: Uses tokio runtime with `spawn` for async and `spawn_blocking` for CPU-bound work
//! - **Wasm**: Uses wasm-bindgen-futures for async tasks and Web Workers for blocking tasks
//!
//! # Example
//!
//! ```rust,ignore
//! use graph_app::task::{TaskExecutor, TaskHandle, CancellationHandle};
//!
//! // Spawn an async task
//! let handle = TaskHandle::spawn(|cancel| async move {
//!     while !cancel.is_cancelled() {
//!         // Do work...
//!         task::yield_now().await;
//!     }
//! });
//!
//! // Check if running
//! if handle.is_running() {
//!     handle.cancel();
//! }
//! ```

mod cancellation;
mod handle;
mod result;
mod traits;
mod utils;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

// Re-export core types
pub(crate) use cancellation::CancellationHandle;
pub(crate) use handle::TaskHandle;
pub(crate) use result::TaskResult;
pub(crate) use traits::{
    BlockingTask,
    TaskExecutor,
};
pub(crate) use utils::{
    sleep,
    sleep_ms,
};

#[cfg(target_arch = "wasm32")]
pub(crate) use utils::yield_now;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::NativeExecutor;
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm::WasmExecutor;

/// Get the default executor for the current platform
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn default_executor() -> NativeExecutor {
    NativeExecutor::new()
}

/// Get the default executor for the current platform
#[cfg(target_arch = "wasm32")]
pub(crate) fn default_executor() -> WasmExecutor {
    WasmExecutor::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_handle() {
        let handle = CancellationHandle::new();
        assert!(!handle.is_cancelled());
        handle.cancel();
        assert!(handle.is_cancelled());
    }
}
