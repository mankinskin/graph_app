# 🏗 Crate Architecture

The context-engine project is organized into layered crates, each building upon the previous to provide increasingly advanced operations on the hypergraph data structure.

## Crate Hierarchy

```
┌─────────────────────────────────────────────────────────┐
│                    context-read                         │
│         (Builds largest token decompositions)           │
├─────────────────────────────────────────────────────────┤
│                   context-insert                        │
│      (Inserts new nodes maintaining invariants)         │
├─────────────────────────────────────────────────────────┤
│                   context-search                        │
│         (Traverses hierarchy to find matches)           │
├─────────────────────────────────────────────────────────┤
│                   context-trace                         │
│    (Foundational types, graph structure, tracing)       │
└─────────────────────────────────────────────────────────┘
```

Each crate depends only on those below it, creating a clean dependency chain:

- **context-trace** → (no internal dependencies)
- **context-search** → context-trace
- **context-insert** → context-search, context-trace
- **context-read** → context-insert, context-search, context-trace

## Crate Responsibilities

### context-trace

The **foundational layer** providing:

- Core hypergraph data structures (`Hypergraph`, `Token`, `Vertex`)
- Thread-safe graph references (`HypergraphRef` via `Arc<RwLock<...>>`)
- Path operations with accessors and mutators
- Bidirectional tracing infrastructure (bottom-up and top-down)
- Pattern and PatternId types
- Cache management primitives

### context-search

The **traversal layer** providing:

- Search algorithms that traverse the node hierarchy
- Policy-driven search operations
- Configurable traversal strategies
- Pattern matching with partial match handling
- Parent exploration (moving from children to containing parents)

The search algorithm:
1. Starts at the first token of a query
2. Traverses **upward** through the parent hierarchy
3. Compares successive query tokens against pattern children
4. Continues until finding the largest match or exhausting options

### context-insert

The **modification layer** providing:

- Insertion of new nodes into the hierarchy
- Split-join architecture for safe modifications
- Maintenance of the **reachability invariant** (see below)
- Multi-phase processing (pre/in/post visit modes)
- Uses context-search internally to find insertion points

### context-read

The **high-level operation layer** providing:

- Building largest known token decompositions for new sequences
- Ordered recursive hypergraph operations
- Uses context-insert to add new patterns
- Uses context-search to find existing patterns
- Expansion chain management

## Data Flow Example

When processing a new sequence like `"ababab"`:

1. **context-read** receives the sequence
2. **context-search** finds the largest matching prefix (e.g., `abab`)
3. **context-insert** adds new patterns if needed (e.g., `ababab = [abab, ab]`)
4. **context-trace** structures store the results

---

## The Reachability Invariant

A **crucial property** that all algorithms depend on:

> Two nodes have a path between them **if and only if** one is a substring of the other.

### Implications

1. **Containment hierarchy**: If `a` contains `b` as a substring, then `a.width() > b.width()`

2. **Transitive reduction**: We only store edges to **closest neighbors** (direct containment), not all containment relationships

3. **Bidirectional traversal**: From any node, we can:
   - Go **down** to all substrings (via children)
   - Go **up** to all containing patterns (via parents)

### Example

For the pattern `"ababab"`:

```
Substrings: "ab", "abab", "bab", "aba", etc.
```

The graph stores direct relationships:
```
ababab ──contains──▶ abab ──contains──▶ ab
       ──contains──▶ ab
```

But NOT redundant edges like `ababab ──▶ ab` directly (that's reachable transitively through `abab`).

### Why This Matters

The search algorithm relies on reachability to find the **largest matching pattern**:

1. Search starts at a small token (e.g., `ab`)
2. Explores parents to find larger containing patterns
3. If reachability is broken, search may miss valid larger patterns

**Broken example**: If we incorrectly stored `ababab = [ab, ab, ab]`:
- `abab` is a substring of `ababab`
- But there's no edge `abab → ababab`
- Search starting from `ab` reaches `abab` but can't reach `ababab`
- Result: Suboptimal decomposition

**Correct structure**: `ababab = [abab, ab]`:
- Edge `abab → ababab` exists
- Search can traverse: `ab → abab → ababab`
- Result: Optimal decomposition found

### Maintenance

The **context-insert** crate is responsible for maintaining reachability when adding new nodes. It must ensure that all substring relationships create appropriate parent-child edges in the graph.
