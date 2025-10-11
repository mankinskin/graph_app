
# Context-Search

Graph search and traversal capabilities for the context framework. 
Provides policy-driven search operations over graph structures with 
pattern matching, folding operations, and configurable traversal strategies.

## Features
- Search operations (sequences, parents, ancestors)
- Configurable traversal strategies (BFT, DFT)
- Early-terminating operations with state management
- Pattern matching with partial match handling
- Resumable search operations

## Structure
- **`search/`**: Searchable trait, find operations
  - `bft.rs`: Breadth-first traversal implementation
  - `context.rs`: Search context and ancestor policies
- **`fold/`**: Foldable trait, early termination
  - `foldable.rs`: Foldable trait and error states
  - `result.rs`: CompleteState, FinishedState, IncompleteState
  - `state.rs`: Folding state management
- **`traversal/`**: TraversalKind, policies, containers
  - `policy.rs`: Traversal policies and strategies
  - `container/`: State containers (BFT, DFT, ordering, extension)
  - `state/`: Traversal state (start, cursor, end conditions)
- **`match/`**: Pattern matching, cursors
  - `iterator.rs`: Match iteration logic
  - `root_cursor.rs`: Root cursor for search operations
- **`compare/`**: State comparison, relationships
  - `iterator.rs`: Compare iteration functionality
  - `parent.rs`: Parent comparison states
  - `state.rs`: General comparison state management

## Usage
```rust
use context_search::Searchable;

// Search for token sequences
let result = graph.find_sequence(vec!["hello", "world"])?;

// Find direct parent matches
let parent_result = graph.find_parent(pattern)?;

// Find ancestor matches
let ancestor_result = graph.find_ancestor(pattern)?;
```

## Key Concepts
- **Foldable Operations**: Early termination for efficient results
- **Policy-Based Design**: Configurable traversal strategies
- **State Continuation**: Pausable and resumable operations

## Dependencies
- **context-trace**: Core graph and tracing functionality
- **petgraph**: Graph data structures and algorithms
- **itertools**: Iterator utilities
- **tracing**: Logging and debugging support

## Development
```bash
cargo test          # Run tests
cargo doc --open    # Generate documentation
```

**Features**: `test-api` (testing utilities), default (logging)

## Architecture
Layered design with search operations, foldable operations, 
configurable traversal policies, pattern matching, and state 
comparison utilities.
