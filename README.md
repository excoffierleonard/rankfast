# rankfast

Goal: build a barebones proof-of-concept for **perfect, optimized ranking** of items with minimal comparisons, assuming a strict total order (transitivity).

The algorithm inserts each item into a growing ranked list using binary search. Each comparison is asked interactively (A vs B), cached, and never repeated. This keeps the number of matchups low (about log2 comparisons per insert) while still producing a deterministic, fully ordered ranking.

In short: ask the fewest possible questions to get a complete ordering, leveraging transitivity and caching.
