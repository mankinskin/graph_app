# 🪜 Searching Hyperedges

**Searching**

Searching an index containing a given pattern. We return the index, in which sub-pattern the pattern was found, at which index position, what the context in the found index is (if it didn't match completely) and the remainder of the query pattern, if any.

The algorithm looks for largest parents starting from the leftmost or rightmost index in the pattern. It searches the ancestral graph of the index for parents matching the given context in the query pattern using breadth first search. It matches each parent with the given context. If no parent matches it searches in the parents of parents where the index is at the back end of the parent relative to the search direction (for example, when we are searching left-to-right, the rightmost position matters). If a parent matches the context until its end, but there are still indices in the search pattern, we continue searching in the parents of the matching direct parent. In this case, if the&#x20;
