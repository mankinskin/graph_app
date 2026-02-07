//! Wasm task executor implementation using wasm-bindgen-futures and Web Workers.
//!
//! This module provides:
//! - `spawn_local`: Spawns async tasks on the main thread
//! - `spawn_blocking`: Spawns blocking tasks in Web Workers
//!
//! # Web Worker Architecture
//!
//! Web Workers allow running JavaScript/Wasm code in background threads.
//! This is essential for CPU-intensive tasks that would otherwise block the UI.
//!
//! The implementation uses a message-passing architecture:
//! 1. Main thread creates a Worker and sends it the task data
//! 2. Worker executes the task and posts results back
//! 3. Main thread receives results and updates the TaskHandle

mod worker;

use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use super::{
    traits::{BlockingTask, TaskExecutor, WasmTaskExecutor},
    CancellationHandle, TaskHandle, TaskResult,
};

/// Wasm task executor using wasm-bindgen-futures and Web Workers.
///
/// This executor provides:
/// - `spawn_local`: Spawns async tasks on the main thread (no Send required)
/// - `spawn_blocking`: Spawns blocking tasks in Web Workers
///
/// # Example
///
/// ```rust,ignore
/// use graph_app::task::{WasmExecutor, WasmTaskExecutor};
///
/// let executor = WasmExecutor::new();
///
/// // Spawn an async task (main thread)
/// let handle = executor.spawn_local(|cancel| async move {
///     while !cancel.is_cancelled() {
///         // Do async work, yielding periodically
///         task::yield_now().await;
///     }
/// });
/// ```
#[derive(Debug, Clone, Default)]
pub(crate) struct WasmExecutor;

impl WasmExecutor {
    /// Create a new wasm executor.
    pub(crate) fn new() -> Self {
        Self
    }

    /// Spawn an async task on the main thread.
    ///
    /// This is the primary way to spawn async tasks on wasm.
    /// The task runs on the main thread, so it should yield periodically
    /// to keep the UI responsive.
    fn spawn_local_impl<F, Fut>(&self, f: F) -> TaskHandle
    where
        F: FnOnce(CancellationHandle) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let cancellation = CancellationHandle::new();
        let cancel_clone = cancellation.clone();
        let cancel_for_check = cancellation.clone();
        let running_flag = Arc::new(AtomicBool::new(true));
        let running_flag_clone = running_flag.clone();
        let result = Arc::new(RwLock::new(None));
        let result_clone = result.clone();

        wasm_bindgen_futures::spawn_local(async move {
            // Yield once to let the UI update before starting
            super::yield_now().await;

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

impl WasmTaskExecutor for WasmExecutor {
    fn spawn_local<F, Fut>(&self, f: F) -> TaskHandle
    where
        F: FnOnce(CancellationHandle) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        self.spawn_local_impl(f)
    }
}

impl TaskExecutor for WasmExecutor {
    fn spawn_async<F, Fut>(&self, f: F) -> TaskHandle
    where
        F: FnOnce(CancellationHandle) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // On wasm, we just use spawn_local since there's no multi-threading
        // for async tasks (only Web Workers for blocking)
        self.spawn_local_impl(f)
    }

    fn spawn_blocking<T: BlockingTask>(&self, task: T) -> TaskHandle {
        worker::spawn_in_worker(task)
    }
}
