//! Native (tokio) task executor implementation.

use std::future::Future;

use futures::FutureExt;

use super::{
    traits::{BlockingTask, TaskExecutor},
    utils::extract_panic_message,
    CancellationHandle, TaskHandle, TaskResult,
};

/// Native task executor using tokio runtime.
///
/// This executor provides:
/// - `spawn_async`: Spawns async tasks on the tokio runtime
/// - `spawn_blocking`: Spawns blocking tasks on tokio's blocking thread pool
///
/// # Example
///
/// ```rust,ignore
/// use graph_app::task::{NativeExecutor, TaskExecutor};
///
/// let executor = NativeExecutor::new();
///
/// // Spawn an async task
/// let handle = executor.spawn_async(|cancel| async move {
///     while !cancel.is_cancelled() {
///         // Do async work
///     }
/// });
///
/// // Spawn a blocking task
/// let handle = executor.spawn_blocking(|cancel| {
///     // CPU-intensive work
/// });
/// ```
#[derive(Debug, Clone, Default)]
pub struct NativeExecutor;

impl NativeExecutor {
    /// Create a new native executor.
    pub fn new() -> Self {
        Self
    }
}

impl TaskExecutor for NativeExecutor {
    fn spawn_async<F, Fut>(&self, f: F) -> TaskHandle
    where
        F: FnOnce(CancellationHandle) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let cancellation = CancellationHandle::new();
        let cancel_clone = cancellation.clone();
        let cancel_for_check = cancellation.clone();

        let join_handle = tokio::spawn(async move {
            // Catch panics
            let result = std::panic::AssertUnwindSafe(f(cancel_clone))
                .catch_unwind()
                .await;

            match result {
                Ok(()) => {
                    if cancel_for_check.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Success
                    }
                }
                Err(panic_info) => {
                    let msg = extract_panic_message(panic_info);
                    TaskResult::Panicked(msg)
                }
            }
        });

        TaskHandle {
            join_handle: Some(join_handle),
            cancellation,
        }
    }

    fn spawn_blocking<T: BlockingTask>(&self, task: T) -> TaskHandle {
        let cancellation = CancellationHandle::new();
        let cancel_clone = cancellation.clone();
        let cancel_for_check = cancellation.clone();

        let join_handle = tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    task.run(cancel_clone);
                }))
            })
            .await;

            match result {
                Ok(Ok(())) => {
                    if cancel_for_check.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Success
                    }
                }
                Ok(Err(panic_info)) => {
                    let msg = extract_panic_message(panic_info);
                    TaskResult::Panicked(msg)
                }
                Err(join_error) => {
                    if join_error.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Panicked(format!("Task join error: {:?}", join_error))
                    }
                }
            }
        });

        TaskHandle {
            join_handle: Some(join_handle),
            cancellation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_spawn_async() {
        let executor = NativeExecutor::new();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        let handle = executor.spawn_async(move |_cancel| async move {
            completed_clone.store(true, Ordering::SeqCst);
        });

        // Wait for completion
        while handle.is_running() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        assert!(completed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_spawn_blocking() {
        let executor = NativeExecutor::new();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        let handle = executor.spawn_blocking(move |_cancel| {
            // Simulate some blocking work
            std::thread::sleep(std::time::Duration::from_millis(50));
            completed_clone.store(true, Ordering::SeqCst);
        });

        // Wait for completion
        while handle.is_running() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        assert!(completed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_cancellation() {
        let executor = NativeExecutor::new();
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = cancelled.clone();

        let handle = executor.spawn_async(move |cancel| async move {
            loop {
                if cancel.is_cancelled() {
                    cancelled_clone.store(true, Ordering::SeqCst);
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });

        // Cancel after a short delay
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        handle.cancel();

        // Wait for the task to respond to cancellation
        while handle.is_running() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        assert!(cancelled.load(Ordering::SeqCst));
    }
}
