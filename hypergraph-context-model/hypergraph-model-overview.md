# 🏞 Hypergraph Model Overview

A hypergraph is used to store compressed sequences of tokens.

A **hyperedge** is conceptually a set of nodes from a graph. A **directed hyperedge** is an ordered sequence of nodes in the graph. We use this model to represent **patterns of token sequences** (link needed) as hyperedges with an **alphabet** of tokens as nodes.

**Recursive hyperedges** can contain other hyperedges as nodes. We create an **index** for every hyperedge (or "pattern") when it is found in at least two different **contexts**. These hyperedges exist in a separate graph model, a **recursive hypergraph**, where each hyperedge also exists as a node, that can be connected with a hyperedge. We impose various **rules** on these edges to maximize compression and stored information.

Larger hyperedges can contain indices of smaller hyperedges. The larger indices are called **parents** and the smaller indices are called **children**. The remaining indices in the larger hyperedge form the **context** of the child in this hyperedge. A hyperedge can have multiple parents and thus multiple different contexts, even within the same parent.

Hyperedges can store **overlapping indices** in the encoding of the pattern they are representing. This is done to have smaller indices point to all larger patterns they occur in, at all positions, and to have larger indices point to all smaller indices they contain at any position.

For simpler computations, hyperedges store multiple **patterns** of indices, in a sort of **brick wall**. We call each row a pattern. They contain indices of different **token widths**. All stored patterns have the same total **token length** and there are no two indices sharing a boundary at the same **token position** between any of the patterns, except for the most outer boundaries, the beginning and end, which are shared by all patterns. We say the inner boundaries of all patterns are **disjoint**.

Hyperedges have **token positions** and **index positions**.

* Token positions refer to the position of a token in **token space**
* Index positions refer to the position of an index in **index space**

A special type of index position is the pattern index (better naming) of a child, which refers to both the pattern in the parent and the index position in that pattern, where the child occurs.

No sequence of indices may occur twice anywhere in all hyperedges. When a known sequence is found in a new pattern, it is **split** out of the existing hyperedge and replaced with an index representing that sequence. Parent and children relationships are updated accordingly.

We use invariants like this to simplify algorithms for **comparing**, **searching, modifying** and **inserting** hyperedges.

****

