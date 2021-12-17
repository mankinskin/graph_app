# Hypergraph Model Overview

A hypergraph is used to store compressed sequences of tokens.

A **hyperedge **is conceptually a set of nodes from a graph. A **directed hyperedge** is an ordered sequence of nodes in the graph. We use this model to represent **patterns of token sequences** (link needed) as hyperedges with an **alphabet** of tokens as nodes.

**Recursive hyperedges** can contain other hyperedges as nodes. We create an **index **for every hyperedge (or "pattern") when it is found in at least two different **contexts**. These hyperedges exist in a separate graph model, a **recursive hypergraph**, where each hyperedge also exists as a node, that can be connected with a hyperedge. We impose various **rules **on these edges to maximize compression and stored information.

Larger hyperedges can contain indices of smaller hyperedges. The larger indices are called **parents **and the smaller indices are called **children**. The remaining indices in the larger hyperedge form the **context **of the child in this hyperedge. A hyperedge can have multiple parents and thus multiple different contexts, even within the same parent.

Hyperedges can store **overlapping indices** in the encoding of the pattern they are representing. This is done to have smaller indices point to all larger patterns they occur in, at all positions, and to have larger indices point to all smaller indices they contain at any position.

For simpler computations, hyperedges store multiple patterns of indices, in a sort of **brick wall**. Every row is a pattern and contains indices of different **token widths**. All stored patterns have the same total **token length** and there are no two indices sharing a boundary at the same **token position** between any of the patterns, except for the most outer boundaries, the beginning and end, which are shared by all patterns. We say the inner boundaries of all patterns are **disjoint**.

Hyperedges have **token positions** and **index positions**.

* Token positions refer to **token space**
* Index positions refer to **index space**

No sequence of indices may occur twice anywhere in all hyperedges. When an existing sequence is found in a new hyperedge, it is **split **out of its context and indexed by itself, creating an index to be used instead. Parent and children relationships are updated accordingly.

****

