//! Task handle for tracking and controlling spawned tasks.

#[cfg(target_arch = "wasm32")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::{CancellationHandle, TaskResult};

/// A handle to track and control a spawned task.
///
/// This provides a unified interface for task management across platforms.
/// The handle allows you to:
/// - Check if a task is running or finished
/// - Request cancellation
/// - Abort the task (native only - on wasm this just cancels)
pub(crate) struct TaskHandle {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) join_handle: Option<tokio::task::JoinHandle<TaskResult>>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) running_flag: Arc<AtomicBool>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) result: Arc<std::sync::RwLock<Option<TaskResult>>>,
    pub(crate) cancellation: CancellationHandle,
}

impl std::fmt::Debug for TaskHandle {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("TaskHandle")
            .field("is_running", &self.is_running())
            .finish()
    }
}

impl TaskHandle {
    /// Check if the task is still running.
    pub(crate) fn is_running(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.join_handle
                .as_ref()
                .map(|h| !h.is_finished())
                .unwrap_or(false)
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.running_flag.load(Ordering::SeqCst)
        }
    }

    /// Check if the task has finished.
    pub(crate) fn is_finished(&self) -> bool {
        !self.is_running()
    }

    /// Request cancellation of the task.
    ///
    /// This is a cooperative cancellation - the task must check
    /// `CancellationHandle::is_cancelled()` and exit gracefully.
    pub(crate) fn cancel(&self) {
        self.cancellation.cancel();
    }

    /// Abort the task immediately.
    ///
    /// - **Native**: Aborts the tokio task
    /// - **Wasm**: Just sets the cancellation flag (cooperative only)
    pub(crate) fn abort(&self) {
        self.cancel();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(handle) = &self.join_handle {
            handle.abort();
        }
    }

    /// Get the cancellation handle.
    pub(crate) fn cancellation(&self) -> &CancellationHandle {
        &self.cancellation
    }

    /// Try to get the result if the task has finished.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn try_get_result(&self) -> Option<TaskResult> {
        if self.is_finished() {
            self.result.read().ok().and_then(|r| r.clone())
        } else {
            None
        }
    }

    // ========================================================================
    // Static spawn methods (convenience wrappers)
    // ========================================================================

    /// Spawn an async task using the default executor.
    ///
    /// The task receives a `CancellationHandle` and should check it periodically.
    /// On wasm, yield periodically using `task::yield_now().await`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn spawn<F, Fut>(f: F) -> Self
    where
        F: FnOnce(CancellationHandle) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        use super::traits::TaskExecutor;
        super::NativeExecutor::new().spawn_async(f)
    }

    /// Spawn an async task using the default executor.
    ///
    /// The task receives a `CancellationHandle` and should check it periodically.
    /// On wasm, yield periodically using `task::yield_now().await`.
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn spawn<F, Fut>(f: F) -> Self
    where
        F: FnOnce(CancellationHandle) -> Fut + 'static,
        Fut: std::future::Future<Output = ()> + 'static,
    {
        use super::traits::WasmTaskExecutor;
        super::WasmExecutor::new().spawn_local(f)
    }

    /// Spawn a blocking task using the default executor.
    ///
    /// - **Native**: Uses `tokio::task::spawn_blocking`
    /// - **Wasm**: Uses a Web Worker
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn spawn_blocking<F>(f: F) -> Self
    where
        F: FnOnce(CancellationHandle) + Send + 'static,
    {
        use super::traits::TaskExecutor;
        super::NativeExecutor::new().spawn_blocking(f)
    }

    /// Spawn a blocking task using the default executor.
    ///
    /// - **Native**: Uses `tokio::task::spawn_blocking`
    /// - **Wasm**: Uses a Web Worker
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn spawn_blocking<F>(f: F) -> Self
    where
        F: FnOnce(CancellationHandle) + Send + 'static,
    {
        use super::traits::TaskExecutor;
        super::WasmExecutor::new().spawn_blocking(f)
    }
}
