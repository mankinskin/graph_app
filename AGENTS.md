# Agent Rules - Graph App Workspace

> **⚠️ READ THIS FILE FIRST** before any code changes in this workspace.

> **📖 IMPORTANT:** Always read [`context-engine/AGENTS.md`](context-engine/AGENTS.md) for detailed development rules, documentation structure, and debugging workflows specific to the core engine.

This is a multi-crate Rust workspace. Each major module has its own documentation.

## Environment Guidelines

- **Prefer bash commands** over PowerShell or cmd when running terminal commands
- **Always use Unix-style paths** (forward slashes `/`) in commands, documentation, and code comments

## Workspace Structure

```
graph_app/                 # Root workspace
├── context-engine/        # Core graph analysis engine ⭐ (has its own AGENTS.md)
├── graph_app/             # GUI application using egui
├── ngrams/                # N-gram utilities
├── tracing-egui/          # Tracing integration for egui (submodule)
├── egui/                  # egui framework (submodule)
├── rerun/                 # Rerun visualization (submodule)
├── doc/                   # Documentation and thesis
└── test/                  # Test assets and corpus
```

## Module Overview

> **⚠️ IMPORTANT:** When working on any `context-*` crate (context-trace, context-search, context-insert, context-read), you **MUST read [`context-engine/AGENTS.md`](context-engine/AGENTS.md) first**. It contains essential development rules, documentation structure, and debugging workflows.

📖 **See [`context-engine/AGENTS.md`](context-engine/AGENTS.md) for detailed development rules, documentation, and workflows.**

Contains crates for graph analysis:
- `context-trace` - Foundation: graph structures, paths, bidirectional tracing
- `context-search` - Pattern matching and search with unified Response API
- `context-insert` - Insertion via split-join architecture
- `context-read` - Context reading and expansion
- `context-trace-macros` - Procedural macros

Architecture: trace → search → insert → read (each layer builds on previous)

### graph_app/

**GUI application** built with egui for visualizing and interacting with the context graph.

### ngrams/

**N-gram utilities** for text processing and analysis.

### tracing-egui/ (submodule)

**Tracing integration** - A tracing Layer and egui Widget to capture and display tracing events in-app with filtering and search.

### External Submodules

- `egui/` - GUI framework
- `rerun/` - Visualization toolkit

## Quick Commands

```bash
# Run context-engine tests
cd context-engine/ && cargo test

# Run specific crate tests
cargo test -p context-trace
cargo test -p context-search
cargo test -p context-insert
```

## Test Tracing Guide

The context-engine crates use a custom test tracing system that writes logs to files and optionally to stdout. This is essential for debugging test failures.

### Basic Usage

```rust
use context_trace::init_test_tracing;

#[test]
fn my_test() {
    let _tracing = init_test_tracing!();  // Basic - logs go to target/test-logs/<test_name>.log
    // test code...
}
```

### With Graph (RECOMMENDED for readable output)

Pass the graph to get human-readable token labels instead of `T0w1`:

```rust
use context_trace::{init_test_tracing, HypergraphRef, BaseGraphKind};

#[test]
fn my_test() {
    let graph = HypergraphRef::<BaseGraphKind>::default();
    // ... build graph first ...
    let _tracing = init_test_tracing!(&graph);  // Tokens show as "abc"(T3) instead of T3w3
    // test code...
}
```

### Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `LOG_STDOUT` | Enable console output | `LOG_STDOUT=1` |
| `LOG_FILTER` | Set log level/filter | `LOG_FILTER=debug` or `LOG_FILTER=context_search=trace` |
| `RUST_LOG` | Fallback log filter | `RUST_LOG=trace` |

### Debug Commands

```bash
# Run single test with full trace output
LOG_STDOUT=1 LOG_FILTER=trace cargo test -p context-read my_test -- --nocapture

# Run test and check log file (logs preserved on failure)
cargo test -p context-read my_test
cat target/test-logs/my_test.log

# Filter logs for specific modules
LOG_STDOUT=1 LOG_FILTER=context_search::search=trace cargo test -p context-search -- --nocapture
```

### Log File Behavior

- **Location:** `<workspace>/context-engine/target/test-logs/<test_name>.log`
- **On success:** Log file is automatically deleted
- **On failure:** Log file is preserved with message: `❌ Test failed! Log file preserved at: ...`

### Common Patterns

```rust
// 1. Initialize tracing AFTER building the graph
let graph = HypergraphRef::<BaseGraphKind>::default();
let _result = ReadRequest::from_text("hello").execute(&mut graph);
let _tracing = init_test_tracing!(&graph);  // Now tokens are labeled

// 2. With custom config
use context_trace::logging::tracing_utils::TracingConfig;
let config = TracingConfig::default().with_stdout_level("debug");
let _tracing = init_test_tracing!(&graph, config);
```

📖 **See [`context-engine/agents/guides/20251203_TOKEN_TEST_LABELING_GUIDE.md`](context-engine/agents/guides/20251203_TOKEN_TEST_LABELING_GUIDE.md) for detailed token labeling troubleshooting.**

## Development Guidelines

| Working on... | Follow... |
|---------------|-----------|
| context-engine | [`context-engine/AGENTS.md`](context-engine/AGENTS.md) (detailed rules, workflows, docs) |
| GUI app | `graph_app/` source code |
| Other modules | Module-specific READMEs |

## Key Documentation

| Module | Documentation |
|--------|---------------|
| **context-engine** | [`context-engine/AGENTS.md`](context-engine/AGENTS.md) - Complete development guide |
| graph_app | [`graph_app/README.md`](graph_app/README.md) |
| Root | [`README.md`](README.md), [`doc/`](doc/) |