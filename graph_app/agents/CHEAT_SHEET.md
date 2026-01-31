# Graph App Cheat Sheet

**Quick reference for graph_app development with egui/eframe**

---

## Essential Types

```rust
// Application
App { graph_file, inserter, read_task, cancellation_token, read_ctx, vis }
ReadCtx  // Graph reading context (async)
GraphVis // Graph visualization state

// Concurrency
Arc<RwLock<ReadCtx>>     // Async read context (async_std)
Arc<SyncRwLock<GraphVis>> // Sync visualization state (std)
CancellationToken        // Tokio cancellation

// Graph (from context-engine)
Graph, HypergraphRef, Token{index, width}
```

---

## Top 10 Patterns

| Pattern | Code |
|---------|------|
| Get read context | `self.ctx()` or `self.ctx_mut()` |
| Get visualization | `self.vis()` or `self.vis_mut()` |
| Start async read | `self.start_read()` |
| Cancel operation | `self.abort()` |
| Check task done | `self.read_task.as_ref().map(\|t\| t.is_finished())` |
| Show UI panel | `egui::TopBottomPanel::top("id").show(ctx, \|ui\| ...)` |
| Context menu | `response.context_menu(\|ui\| ...)` |
| Close window | `ctx.send_viewport_cmd(egui::ViewportCommand::Close)` |
| Text input | `ui.text_edit_singleline(&mut text)` |
| Button click | `if ui.button("Label").clicked() { ... }` |

---

## UI Structure

```rust
// Main app loop
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.top_panel(ctx, frame);    // Menu bar
        self.central_panel(ctx);       // Main content
        // Optional windows
        egui::Window::new("Title").show(ctx, |ui| { ... });
    }
}
```

---

## Async Patterns

```rust
// Start background task
let token = CancellationToken::new();
let task = tokio::spawn(async move {
    // Check cancellation
    if token.is_cancelled() { return; }
    // Do work...
});

// Cancel task
token.cancel();
task.abort();
```

---

## Critical Gotchas

1. **Lock guards** - `ctx()` returns `Option<RwLockReadGuard>`, handle `None`
2. **Async vs Sync locks** - `read_ctx` uses async_std, `vis` uses std
3. **Task cleanup** - Always clear `cancellation_token` when task finishes
4. **UI thread** - Don't block the UI thread with async operations
5. **Persistence** - Use `#[cfg(feature = "persistence")]` for save/load

---

## Testing Essentials

```bash
# Run application
cargo run

# Run with logging
RUST_LOG=debug cargo run

# Run tests
cargo test
```

---

## Debug Commands

```bash
# Check for errors
cargo check

# Full build
cargo build

# Release build
cargo build --release
```

---

## Common Imports

```rust
use eframe::egui::{self, Ui};
use async_std::sync::{Arc, RwLock};
use tokio_util::sync::CancellationToken;
```
