//! Unified task abstraction for cross-platform async task management.
//!
//! This module provides a common interface for spawning and cancelling tasks
//! that works on both native (tokio) and wasm (wasm-bindgen-futures).
//!
//! # Architecture
//!
//! Both platforms use async execution with a unified interface:
//! - **Native**: Uses tokio with spawn_blocking for CPU-bound sync work
//! - **Wasm**: Uses wasm-bindgen-futures with async algorithms that yield periodically
//!
//! The key insight is that wasm runs on the main thread, so sync code will block
//! the UI. To keep the UI responsive, algorithms must be written as async code
//! that yields control via `gloo_timers::future::TimeoutFuture::new(0).await`.

use std::sync::{
    atomic::{
        AtomicBool,
        Ordering,
    },
    Arc,
};

#[cfg(not(target_arch = "wasm32"))]
use tokio_util::sync::CancellationToken;

/// A handle to cancel a running task
#[derive(Clone)]
pub struct CancellationHandle {
    #[cfg(not(target_arch = "wasm32"))]
    token: CancellationToken,
    #[cfg(target_arch = "wasm32")]
    flag: Arc<AtomicBool>,
}

impl CancellationHandle {
    /// Create a new cancellation handle
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            token: CancellationToken::new(),
            #[cfg(target_arch = "wasm32")]
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Request cancellation
    pub fn cancel(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.token.cancel();
        #[cfg(target_arch = "wasm32")]
        self.flag.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation has been requested
    pub fn is_cancelled(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return self.token.is_cancelled();
        #[cfg(target_arch = "wasm32")]
        return self.flag.load(Ordering::SeqCst);
    }

    /// Get the native cancellation token (native only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Get the wasm cancellation flag (wasm only)
    #[cfg(target_arch = "wasm32")]
    pub fn flag(&self) -> Arc<AtomicBool> {
        self.flag.clone()
    }
}

impl Default for CancellationHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a task execution
#[derive(Debug, Clone)]
pub enum TaskResult {
    /// Task completed successfully
    Success,
    /// Task was cancelled
    Cancelled,
    /// Task panicked with an error message
    Panicked(String),
}

/// A handle to track and control a spawned task
pub struct TaskHandle {
    #[cfg(not(target_arch = "wasm32"))]
    join_handle: Option<tokio::task::JoinHandle<TaskResult>>,
    #[cfg(target_arch = "wasm32")]
    running_flag: Arc<AtomicBool>,
    #[cfg(target_arch = "wasm32")]
    result: Arc<std::sync::RwLock<Option<TaskResult>>>,
    cancellation: CancellationHandle,
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
    /// Check if the task is still running
    pub fn is_running(&self) -> bool {
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

    /// Check if the task has finished
    pub fn is_finished(&self) -> bool {
        !self.is_running()
    }

    /// Request cancellation of the task
    pub fn cancel(&self) {
        self.cancellation.cancel();
    }

    /// Abort the task immediately (native only - on wasm this just cancels)
    pub fn abort(&self) {
        self.cancel();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(handle) = &self.join_handle {
            handle.abort();
        }
    }

    /// Get the cancellation handle
    pub fn cancellation(&self) -> &CancellationHandle {
        &self.cancellation
    }

    /// Try to get the result if the task has finished (wasm only)
    #[cfg(target_arch = "wasm32")]
    pub fn try_get_result(&self) -> Option<TaskResult> {
        if self.is_finished() {
            self.result.read().ok().and_then(|r| r.clone())
        } else {
            None
        }
    }

    /// Spawn an async task (unified interface for both platforms)
    ///
    /// The task receives a CancellationHandle and should check it periodically.
    /// On wasm, yield periodically using `gloo_timers::future::TimeoutFuture::new(0).await`
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn<F, Fut>(f: F) -> Self
    where
        F: FnOnce(CancellationHandle) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        Self::spawn_native(f)
    }

    /// Spawn an async task (unified interface for both platforms)
    ///
    /// The task receives a CancellationHandle and should check it periodically.
    /// On wasm, yield periodically using `gloo_timers::future::TimeoutFuture::new(0).await`
    #[cfg(target_arch = "wasm32")]
    pub fn spawn<F, Fut>(f: F) -> Self
    where
        F: FnOnce(CancellationHandle) -> Fut + 'static,
        Fut: std::future::Future<Output = ()> + 'static,
    {
        Self::spawn_wasm(f)
    }
}

// ============================================================================
// Native implementation
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
impl TaskHandle {
    fn spawn_native<F, Fut>(f: F) -> Self
    where
        F: FnOnce(CancellationHandle) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        use futures::FutureExt;

        let cancellation = CancellationHandle::new();
        let cancel_clone = cancellation.clone();
        let cancel_for_check = cancellation.clone();

        let join_handle = tokio::spawn(async move {
            // Catch panics
            let result = std::panic::AssertUnwindSafe(f(cancel_clone))
                .catch_unwind()
                .await;

            match result {
                Ok(()) =>
                    if cancel_for_check.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Success
                    },
                Err(panic_info) => {
                    let msg = extract_panic_message(panic_info);
                    TaskResult::Panicked(msg)
                },
            }
        });

        TaskHandle {
            join_handle: Some(join_handle),
            cancellation,
        }
    }

    /// Spawn a blocking task (native only)
    /// Use this for CPU-bound sync work that would block the async runtime
    pub fn spawn_blocking<F>(f: F) -> Self
    where
        F: FnOnce(&CancellationHandle) + Send + 'static,
    {
        let cancellation = CancellationHandle::new();
        let cancel_clone = cancellation.clone();
        let cancel_for_check = cancellation.clone();

        let join_handle = tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    f(&cancel_clone);
                }))
            })
            .await;

            match result {
                Ok(Ok(())) =>
                    if cancel_for_check.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Success
                    },
                Ok(Err(panic_info)) => {
                    let msg = extract_panic_message(panic_info);
                    TaskResult::Panicked(msg)
                },
                Err(join_error) =>
                    if join_error.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Panicked(format!(
                            "Task join error: {:?}",
                            join_error
                        ))
                    },
            }
        });

        TaskHandle {
            join_handle: Some(join_handle),
            cancellation,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn extract_panic_message(panic_info: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    }
}

// ============================================================================
// Wasm implementation
// ============================================================================

#[cfg(target_arch = "wasm32")]
impl TaskHandle {
    fn spawn_wasm<F, Fut>(f: F) -> Self
    where
        F: FnOnce(CancellationHandle) -> Fut + 'static,
        Fut: std::future::Future<Output = ()> + 'static,
    {
        let cancellation = CancellationHandle::new();
        let cancel_clone = cancellation.clone();
        let cancel_for_check = cancellation.clone();
        let running_flag = Arc::new(AtomicBool::new(true));
        let running_flag_clone = running_flag.clone();
        let result = Arc::new(std::sync::RwLock::new(None));
        let result_clone = result.clone();

        wasm_bindgen_futures::spawn_local(async move {
            // Yield once to let the UI update before starting
            yield_now().await;

            // Run the async task
            f(cancel_clone).await;

            // Determine result
            let final_result = if cancel_for_check.is_cancelled() {
                TaskResult::Cancelled
            } else {
                TaskResult::Success
            };

            if let Ok(mut r) = result_clone.write() {
                *r = Some(final_result);
            }
            running_flag_clone.store(false, Ordering::SeqCst);
        });

        TaskHandle {
            running_flag,
            result,
            cancellation,
        }
    }
}

/// Yield control back to the JavaScript event loop.
/// This allows the UI to remain responsive during long-running async tasks.
#[cfg(target_arch = "wasm32")]
async fn yield_now() {
    use std::{
        future::Future,
        pin::Pin,
        task::{
            Context,
            Poll,
        },
    };

    struct YieldNow(bool);

    impl Future for YieldNow {
        type Output = ();

        fn poll(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<()> {
            if self.0 {
                Poll::Ready(())
            } else {
                self.0 = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    YieldNow(false).await
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
