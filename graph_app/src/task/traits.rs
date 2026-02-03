//! Core traits for task execution abstraction.
//!
//! These traits define the interface for spawning tasks across different platforms.

use std::future::Future;

use super::{
    CancellationHandle,
    TaskHandle,
};

/// A blocking task that can be executed in a worker context.
///
/// On native, this runs in a tokio blocking thread pool.
/// On wasm, this runs in a Web Worker.
///
/// The task receives a `CancellationHandle` to check for cancellation requests.
pub trait BlockingTask: Send + 'static {
    /// Execute the blocking task
    fn run(
        self,
        cancellation: CancellationHandle,
    );
}

/// Implement BlockingTask for closures that take an owned CancellationHandle.
impl<F> BlockingTask for F
where
    F: FnOnce(CancellationHandle) + Send + 'static,
{
    fn run(
        self,
        cancellation: CancellationHandle,
    ) {
        self(cancellation)
    }
}

/// Trait for executing tasks across different platforms.
///
/// This trait provides a unified interface for spawning both async and blocking tasks.
/// Each platform provides its own implementation:
///
/// - **Native**: Uses tokio runtime
/// - **Wasm**: Uses wasm-bindgen-futures and Web Workers
pub trait TaskExecutor {
    /// Spawn an async task.
    ///
    /// The task receives a `CancellationHandle` that it should check periodically.
    /// On wasm, the task should yield periodically to keep the UI responsive.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// executor.spawn_async(|cancel| async move {
    ///     while !cancel.is_cancelled() {
    ///         // Do async work
    ///     }
    /// });
    /// ```
    fn spawn_async<F, Fut>(
        &self,
        f: F,
    ) -> TaskHandle
    where
        F: FnOnce(CancellationHandle) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static;

    /// Spawn a blocking task.
    ///
    /// On native, this uses `tokio::task::spawn_blocking`.
    /// On wasm, this uses a Web Worker to avoid blocking the main thread.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// executor.spawn_blocking(|cancel| {
    ///     // CPU-intensive work
    ///     while !cancel.is_cancelled() {
    ///         // Process data...
    ///     }
    /// });
    /// ```
    fn spawn_blocking<T: BlockingTask>(
        &self,
        task: T,
    ) -> TaskHandle;
}

/// Wasm-specific executor trait with relaxed bounds (no Send required).
#[cfg(target_arch = "wasm32")]
pub trait WasmTaskExecutor {
    /// Spawn an async task without Send requirement.
    fn spawn_local<F, Fut>(
        &self,
        f: F,
    ) -> TaskHandle
    where
        F: FnOnce(CancellationHandle) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static;
}
