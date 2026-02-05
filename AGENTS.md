# Agent Rules - Graph App Workspace

> **⚠️ READ THIS FILE FIRST** before any code changes in this workspace.

This is a multi-crate Rust workspace. Each major module has its own documentation.

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

### context-engine/ ⭐ Primary Development

**Core graph-based context analysis engine** - the main focus of development.

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
# Run the GUI app
cd graph_app/ && cargo run

# Run context-engine tests
cd context-engine/ && cargo test

# Run specific crate tests
cargo test -p context-trace
cargo test -p context-search
cargo test -p context-insert
```

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