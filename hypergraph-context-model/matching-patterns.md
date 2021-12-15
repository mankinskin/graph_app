# 🧦 Matching Patterns

#### Matching

Comparing two patterns by their indicies whether they produce the same token sequences. Matching can be done from **left to right** or **right to left**. We return how the patterns match:

* Exactly: The patterns produce the same literal token sequence
* Remainder: They match at the beginning (respective to matching direction) but either or both patterns have a remainder that did not match exactly with the other pattern. If one pattern is shorter than the other, the remainder of the longer one is returned.
* Don't match: the patterns don't match at all

The algorithm uses the invariant that only smaller/shorter indicies can be children of larger/longer indicies. It iterates through both patterns and compares each pair of indicies at the same index position. When one pattern ends before the other, the remainder is returned. When different indicies are encountered, they are compared for their size, and the larger one is **searched** as an ancestor of the smaller one in the given context.
