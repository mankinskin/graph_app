# Context Graph Engine

<img width="1920" height="1080" alt="Context Graph Engine" src="https://github.com/user-attachments/assets/a6cf3d91-f220-4082-ac37-7e33a4462dd5" />

> **Status: experimental research prototype.** This project explores a
> symbolic language-model representation based on hierarchical n-grams and
> their context relationships. It is not presented as a production LLM service.

The workspace combines an interactive graph application with Rust crates for
parsing text into n-gram and context graphs. The research design described in the
[thesis documentation](doc/thesis/README.md) aims for a representation that is
reconstructable, human-readable, and updatable, with frequency counts
supporting local predictions.

## What is in the repository

- [`graph_app/`](graph_app/) is a Rust desktop application built with
  `eframe`/`egui`. It displays and edits graphs, loads example graph presets,
  accepts text input, and can run the `N-grams Parse Corpus`, `Context Read`,
  and `Context Insert` algorithms.
- [`ngrams/`](ngrams/) contains corpus parsing and n-gram graph construction,
  including frequency labeling and substring-containment relationships.
- [`context-engine/`](context-engine/) provides the local `context-trace`,
  `context-read`, and `context-insert` crates used by the application.

The main technology stack is Rust 2021, `eframe`/`egui`, the `ngrams` crate, and
the local context-engine crates.

## Setup and run

The workspace uses nested Git submodules. Clone them recursively:

```bash
git clone --recurse-submodules https://github.com/mankinskin/graph_app.git
cd graph_app
cargo run -p graph_app
```

For an existing checkout that was cloned without submodules, run this command
from the repository root before building:

```bash
git submodule update --init --recursive
```

## Current build blocker

The app's documented run command cannot currently complete at the pinned
dependency revisions. `graph_app/Cargo.toml` points to
`context-engine/crates/context-*`, while the pinned `context-engine` workspace
lists those crates under `context-stack/` and has no root `crates/` tree. As a
result, `cargo run -p graph_app` and workspace commands such as
`cargo test -p ngrams` fail while resolving the missing
`context-engine/crates/context-insert/Cargo.toml`. The dependency paths must be
aligned before those commands can be verified as working.

## Tests

These are the intended test commands after the recursive submodules are
initialized. The `ngrams` workspace command is currently blocked by the path
mismatch described above; the nested context-engine command also requires its
own `context-stack` and `memory-api` submodules.

```bash
cargo test -p ngrams
cd context-engine
cargo test
```
