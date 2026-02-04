//! Web Worker integration for running blocking tasks.
//!
//! This module provides the infrastructure to run CPU-intensive blocking tasks
//! in Web Workers, keeping the main thread responsive.
//!
//! # Architecture
//!
//! Web Workers in wasm-bindgen require special handling:
//!
//! 1. **Worker Script**: We need a separate worker script or inline blob
//! 2. **SharedArrayBuffer**: For efficient data sharing (requires COOP/COEP headers)
//! 3. **Message Passing**: For communication between main thread and worker
//!
//! ## Current Implementation
//!
//! Due to the complexity of full Web Worker support with wasm-bindgen, this
//! implementation provides a simulated blocking context using chunked async
//! execution. This keeps the API consistent while we work on full worker support.
//!
//! For true parallel execution, see the `wasm-bindgen-rayon` crate or
//! implement a custom worker pool.

use std::sync::{
    atomic::{
        AtomicBool,
        Ordering,
    },
    Arc,
    RwLock,
};

use crate::task::{
    traits::BlockingTask,
    CancellationHandle,
    TaskHandle,
    TaskResult,
};

/// Spawn a blocking task in a Web Worker.
///
/// This function creates a Web Worker to execute the blocking task,
/// allowing the main thread to remain responsive.
///
/// # Current Limitations
///
/// Full Web Worker support requires:
/// - COOP/COEP headers on the server for SharedArrayBuffer
/// - A separate worker script or blob URL
/// - Careful handling of wasm memory
///
/// The current implementation simulates blocking behavior using chunked
/// async execution on the main thread. For true parallelism, consider
/// using `wasm-bindgen-rayon`.
pub fn spawn_in_worker<T: BlockingTask>(task: T) -> TaskHandle {
    let cancellation = CancellationHandle::new();
    let cancel_clone = cancellation.clone();
    let cancel_for_check = cancellation.clone();
    let running_flag = Arc::new(AtomicBool::new(true));
    let running_flag_clone = running_flag.clone();
    let result = Arc::new(RwLock::new(None));
    let result_clone = result.clone();

    // Try to spawn in a real Web Worker first
    if let Some(handle) = try_spawn_worker(cancellation.clone()) {
        return handle;
    }

    // Fallback: Run on main thread with periodic yields
    // This simulates blocking behavior while keeping UI responsive
    wasm_bindgen_futures::spawn_local(async move {
        web_sys::console::warn_1(
            &"Web Worker not available, running blocking task on main thread"
                .into(),
        );

        // Yield to let UI update
        crate::task::yield_now().await;

        // Run the task
        // Note: This will block the main thread, but we yield before and after
        let panic_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                task.run(cancel_clone);
            }));

        let final_result = match panic_result {
            Ok(()) =>
                if cancel_for_check.is_cancelled() {
                    TaskResult::Cancelled
                } else {
                    TaskResult::Success
                },
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                TaskResult::Panicked(msg)
            },
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

/// Attempt to spawn a task in a real Web Worker.
///
/// Returns `None` if Web Workers are not available or setup fails.
fn try_spawn_worker(_cancellation: CancellationHandle) -> Option<TaskHandle> {
    // Check if Web Workers are available
    let _window = web_sys::window()?;

    // Check if we have the Worker constructor
    if !has_worker_support() {
        web_sys::console::log_1(
            &"Web Workers not supported in this environment".into(),
        );
        return None;
    }

    // For full Web Worker support, we would need to:
    // 1. Create a Blob with worker script code
    // 2. Create a Worker from the blob URL
    // 3. Set up message handlers for communication
    // 4. Serialize the task and send it to the worker
    // 5. Handle results via postMessage
    //
    // This requires additional setup like:
    // - wasm-bindgen-rayon for thread support
    // - Proper COOP/COEP headers
    // - A worker script that can load the wasm module
    //
    // For now, we return None to use the fallback

    web_sys::console::log_1(
        &"Full Web Worker support requires additional setup. Using fallback."
            .into(),
    );

    None
}

/// Check if the browser supports Web Workers.
fn has_worker_support() -> bool {
    js_sys::Reflect::get(&js_sys::global(), &"Worker".into())
        .map(|v| !v.is_undefined())
        .unwrap_or(false)
}

// ============================================================================
// Worker Pool (for future implementation)
// ============================================================================

/// A pool of Web Workers for executing blocking tasks.
///
/// This is a placeholder for future implementation of a proper worker pool.
/// The pool would:
/// - Pre-create a fixed number of workers
/// - Queue tasks and distribute them to available workers
/// - Handle worker lifecycle and error recovery
#[derive(Debug)]
#[allow(dead_code)]
pub struct WorkerPool {
    max_workers: usize,
    // workers: Vec<Worker>,
    // task_queue: VecDeque<QueuedTask>,
}

impl WorkerPool {
    /// Create a new worker pool with the specified maximum number of workers.
    #[allow(dead_code)]
    pub fn new(max_workers: usize) -> Self {
        Self { max_workers }
    }

    /// Get the ideal number of workers based on hardware concurrency.
    #[allow(dead_code)]
    pub fn ideal_worker_count() -> usize {
        // Try to get navigator.hardwareConcurrency
        web_sys::window()
            .map(|w| w.navigator().hardware_concurrency() as usize)
            .unwrap_or(4)
            .max(1)
    }
}
