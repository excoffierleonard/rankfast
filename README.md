# rankfast

Goal: build a barebones proof-of-concept for **comparison-optimized ranking** of items with minimal comparisons, assuming a strict total order (transitivity).

The current implementation uses the Ford-Johnson (merge-insertion) sorting algorithm to reduce the number of comparisons. It asks comparisons interactively (A vs B), and it relies on a consistent comparator to produce a deterministic, fully ordered ranking.

In short: minimize comparisons via Ford-Johnson under transitivity.
