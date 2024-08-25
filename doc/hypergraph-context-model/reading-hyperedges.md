# 👓 Reading Hyperedges

Reading new hyperedges means searching for known patterns in a given sequence of tokens or pattern indicies and adding new indicies for repeated patterns. We want to assert all invariants of the index while reading:

* No sequence occurs twice at different locations in the graph
* No child patterns have a border at the same token position
* Each index contains all indicies representing a subsequence of it

#### Reading new tokens

Token sequences can contain new tokens, i.e. tokens not indexed yet. The sequence can therefore be partitioned into blocks of known tokens and new (unknown) tokens. Blocks of new tokens are always creating a new pattern index to represent the given new token block.

#### Reading known tokens

Known tokens may occur in existing pattern indicies, so we start searching for largest ancestors from the beginning of a known block. Because known patterns may also occur after the start of the first pattern but before the end of it, we have to look for indicies starting in the first pattern. We call these indicies overlaps. After we have read at least two indicies in sequence, i.e. we have found two indicies representing the beginning of the known block, we can search for overlaps between these two indicies.

#### Finding Overlaps

Since no index sequence may occur twice at two different locations in the hypergraph, we only need to start searching from the last indicies of each child pattern. We search for indicies starting with the last index of a child pattern and continuing with a prefix of the next index.

We search for all overlaps starting with the largest postfix of the first index and the largest prefix of the second index. We try postfixes largest to smallest and for each we try prefixes largest to next largest to smallest. This way we find the largest overlaps first and can skip some checks.

On the second side the whole index also counts as a prefix and is considered for overlaps but on the first side the whole index is not considered for overlaps because it must be the largest first index already.

If we find an overlap, we want to remove all overlaps with both a smaller first half and a smaller second half from the search space. This way we will not search for overlaps already contained in the overlaps we found. (needs more explanation)

To improve cache efficiency, we try to find all overlaps in the children of the given indices before descending down into their smallest inner children. That means we try each first half with each second half at the current indicies before trying smaller second halves for first halves we didn't a second half for.

We store the half sizes for the overlap where all larger first halves have finished their search (found overlap or have no overlap). We know that we only need to look at smaller first sides and larger second sides respectively.

We may check the two smallest inner indicies early to detect whether two indicies can have an overlap at all, because any overlaps would always contain the smallest overlap of the two indicies.

When we add an overlap to a new index we also need to create an index for the context of the overlap, because it will be stored at two different locations in the graph. For this we need to remember the parent index and the respective pattern id on both sides.

After we found the overlaps of two adjacient indices, we search the next adjacient index and search for overlaps between the previous index and its respective overlap patterns. That means we use the previous index as the parent for its child patterns, but we need to store the parent locations of the previous overlap contexts.

We use previous overlap patterns instead of the respective patterns in the overlapped index where they were taken from to find further overlaps. This is correct because the pattern with the overlap contains a larger index than the child pattern in the respective parent.

Any overlap patterns which don't find an overlap create a shared boundary in two patterns (the pattern with the overlap and the pattern of the patterns being overlapped) and thus a new index needs to be created for them.

Any overlap patterns finding new overlaps are extended into the next pattern and remain an overlap pattern.

If there were any other overlaps on the previous two indicies, new overlap patterns found will use a different set of patterns to store reading patterns, because they start at a later position.

Once no overlaps are found for a set of patterns and the next adjacient index, all previous pattern sets are indexed into indices and the whole process repeats until no more adjacient indices are found.
