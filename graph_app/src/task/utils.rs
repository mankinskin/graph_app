//! Cross-platform utility functions for async task management.

use std::time::Duration;

/// Cross-platform async sleep.
///
/// - **Native**: Uses `tokio::time::sleep`
/// - **Wasm**: Uses `gloo_timers::future::TimeoutFuture`
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// Cross-platform async sleep.
///
/// - **Native**: Uses `tokio::time::sleep`
/// - **Wasm**: Uses `gloo_timers::future::TimeoutFuture`
#[cfg(target_arch = "wasm32")]
pub(crate) async fn sleep(duration: Duration) {
    gloo_timers::future::TimeoutFuture::new(duration.as_millis() as u32).await;
}

/// Convenience function for sleeping a number of milliseconds.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep_ms(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await;
}

/// Convenience function for sleeping a number of milliseconds.
#[cfg(target_arch = "wasm32")]
pub(crate) async fn sleep_ms(millis: u32) {
    gloo_timers::future::TimeoutFuture::new(millis).await;
}

/// Yield control back to the JavaScript event loop (wasm only).
///
/// This allows the UI to remain responsive during long-running async tasks.
/// Call this periodically in long-running async tasks on wasm.
///
/// # Example
///
/// ```rust,ignore
/// for item in large_collection {
///     process(item);
///     yield_now().await; // Let UI update
/// }
/// ```
#[cfg(target_arch = "wasm32")]
pub(crate) async fn yield_now() {
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
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

/// Extract a message from a panic payload.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn extract_panic_message(panic_info: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    }
}
