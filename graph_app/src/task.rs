//! Unified task abstraction for cross-platform async task management.
//!
//! This module provides a common interface for spawning, cancelling, and polling
//! tasks that works on both native (tokio) and wasm (wasm-bindgen-futures).

use std::sync::{
    atomic::{
        AtomicBool,
        Ordering,
    },
    Arc,
};

#[cfg(not(target_arch = "wasm32"))]
use tokio_util::sync::CancellationToken;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::{
    self,
    prelude::*,
    JsCast,
    JsValue,
};

#[cfg(target_arch = "wasm32")]
use js_sys;

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

/// Status of a spawned task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task was cancelled
    Cancelled,
    /// Task panicked with an error
    Panicked,
    /// No task is active
    Idle,
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
    #[allow(dead_code)]
    status: Arc<AtomicTaskStatus>,
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

/// Atomic wrapper for TaskStatus
struct AtomicTaskStatus(AtomicBool, AtomicBool); // (is_running, has_error)

impl AtomicTaskStatus {
    fn new() -> Self {
        Self(AtomicBool::new(true), AtomicBool::new(false))
    }

    fn set_completed(&self) {
        self.0.store(false, Ordering::SeqCst);
    }

    fn set_error(&self) {
        self.1.store(true, Ordering::SeqCst);
        self.0.store(false, Ordering::SeqCst);
    }

    fn is_running(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    fn has_error(&self) -> bool {
        self.1.load(Ordering::SeqCst)
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

    /// Try to get the result if the task has finished
    #[cfg(target_arch = "wasm32")]
    pub fn try_get_result(&self) -> Option<TaskResult> {
        if self.is_finished() {
            self.result.read().ok().and_then(|r| r.clone())
        } else {
            None
        }
    }
}

/// Spawn a task that runs the given closure
///
/// On native: Uses tokio::spawn with spawn_blocking for sync work
/// On wasm: Uses wasm_bindgen_futures::spawn_local with catch_unwind
pub fn spawn<F>(f: F) -> TaskHandle
where
    F: FnOnce(&CancellationHandle) + Send + 'static,
{
    let cancellation = CancellationHandle::new();
    let status = Arc::new(AtomicTaskStatus::new());

    #[cfg(not(target_arch = "wasm32"))]
    {
        let cancel_clone = cancellation.clone();
        let cancel_for_check = cancellation.clone();
        let status_clone = status.clone();

        let join_handle = tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    f(&cancel_clone);
                }))
            })
            .await;

            match result {
                Ok(Ok(())) => {
                    status_clone.set_completed();
                    if cancel_for_check.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Success
                    }
                },
                Ok(Err(panic_info)) => {
                    status_clone.set_error();
                    let msg = extract_panic_message(panic_info);
                    TaskResult::Panicked(msg)
                },
                Err(join_error) => {
                    status_clone.set_error();
                    if join_error.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Panicked(format!(
                            "Task join error: {:?}",
                            join_error
                        ))
                    }
                },
            }
        });

        TaskHandle {
            join_handle: Some(join_handle),
            cancellation,
            status,
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let cancel_clone = cancellation.clone();
        let cancel_for_check = cancellation.clone();
        let running_flag = Arc::new(AtomicBool::new(true));
        let running_flag_clone = running_flag.clone();
        let result = Arc::new(std::sync::RwLock::new(None));
        let result_clone = result.clone();
        let status_clone = status.clone();

        wasm_bindgen_futures::spawn_local(async move {
            // Yield to allow the UI to update before starting heavy work
            yield_now().await;

            // Run the task with JavaScript-level error catching
            let task_result = run_task_with_js_catch(move || {
                f(&cancel_clone);
            });

            let final_result = match task_result {
                Ok(()) => {
                    status_clone.set_completed();
                    if cancel_for_check.is_cancelled() {
                        TaskResult::Cancelled
                    } else {
                        TaskResult::Success
                    }
                },
                Err(msg) => {
                    status_clone.set_error();
                    web_sys::console::error_1(
                        &format!("Task panicked: {}", msg).into(),
                    );
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
            status,
        }
    }
}

/// Extract a message from a panic payload
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

/// Yield control back to the JavaScript event loop
/// This allows the UI to remain responsive during long-running tasks
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

/// Sleep for a given number of milliseconds using JavaScript's setTimeout
#[cfg(target_arch = "wasm32")]
pub async fn sleep_ms(ms: u32) {
    use wasm_bindgen_futures::JsFuture;
    
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("no window");
        window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .expect("setTimeout failed");
    });
    
    JsFuture::from(promise).await.ok();
}

/// Spawn an async task for wasm
/// This is different from `spawn` - it takes an async closure that can yield to the event loop
#[cfg(target_arch = "wasm32")]
pub fn spawn_async<F, Fut>(f: F) -> TaskHandle
where
    F: FnOnce(CancellationHandle) -> Fut + 'static,
    Fut: std::future::Future<Output = ()> + 'static,
{
    let cancellation = CancellationHandle::new();
    let status = Arc::new(AtomicTaskStatus::new());
    
    let cancel_clone = cancellation.clone();
    let cancel_for_check = cancellation.clone();
    let running_flag = Arc::new(AtomicBool::new(true));
    let running_flag_clone = running_flag.clone();
    let result = Arc::new(std::sync::RwLock::new(None));
    let result_clone = result.clone();
    let status_clone = status.clone();

    wasm_bindgen_futures::spawn_local(async move {
        // Yield once to let the UI update
        yield_now().await;
        
        // Run the async task
        let task_future = f(cancel_clone);
        task_future.await;
        
        // Mark as completed
        status_clone.set_completed();
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
        status,
    }
}

/// Thread-local storage for panic messages before they abort
#[cfg(target_arch = "wasm32")]
thread_local! {
    static PANIC_MESSAGE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Run a task with JavaScript-level error catching.
///
/// This works by:
/// 1. Setting a custom panic hook that stores the panic message and throws a JS exception
/// 2. Calling the task via JavaScript which can catch the exception
/// 3. Returning the error message if a panic occurred
#[cfg(target_arch = "wasm32")]
fn run_task_with_js_catch<F>(f: F) -> Result<(), String>
where
    F: FnOnce() + 'static,
{
    use std::panic;

    // Clear any previous panic message
    PANIC_MESSAGE.with(|pm| *pm.borrow_mut() = None);

    // Save the original panic hook
    let original_hook = panic::take_hook();

    // Set a custom panic hook that stores the message and throws a JS exception
    panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = info
            .location()
            .map(|loc| {
                format!(" at {}:{}:{}", loc.file(), loc.line(), loc.column())
            })
            .unwrap_or_default();

        let full_msg = format!("{}{}", msg, location);

        // Store the message for later retrieval
        PANIC_MESSAGE.with(|pm| *pm.borrow_mut() = Some(full_msg.clone()));

        // Log to console
        web_sys::console::error_1(&format!("Panic: {}", full_msg).into());

        // Throw a JavaScript exception to prevent the Rust abort
        // This will unwind the JS call stack instead of aborting
        wasm_bindgen::throw_str(&full_msg);
    }));

    // Call the task through JavaScript so we can catch the exception
    let result = call_with_js_catch(Box::new(f));

    // Restore the original panic hook
    panic::set_hook(original_hook);

    result
}

/// Helper to call a closure through JavaScript with exception catching
#[cfg(target_arch = "wasm32")]
fn call_with_js_catch(f: Box<dyn FnOnce()>) -> Result<(), String> {
    use std::cell::RefCell;

    // We'll use a simpler approach - just run the closure and if the panic hook
    // throws a JS exception, the wasm execution will stop but won't abort the runtime
    let f = RefCell::new(Some(f));
    let closure = Closure::once(move || {
        if let Some(func) = f.borrow_mut().take() {
            func();
        }
    });

    // Call the closure and catch any JS exception
    let result = catch_js_exception(&closure);

    match result {
        Ok(()) => Ok(()),
        Err(js_val) => {
            let msg = js_val
                .as_string()
                .unwrap_or_else(|| format!("{:?}", js_val));
            Err(msg)
        },
    }
}

/// Call a closure and catch any JavaScript exception
#[cfg(target_arch = "wasm32")]
fn catch_js_exception(closure: &Closure<dyn FnMut()>) -> Result<(), JsValue> {
    let func: &js_sys::Function = closure.as_ref().unchecked_ref();

    // Use Reflect.apply with try/catch semantics via js_sys
    // We'll call the function and if it throws, it becomes a Result::Err
    func.call0(&JsValue::NULL).map(|_| ())
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
