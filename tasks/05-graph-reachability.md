# Mission 05 - Find live objects in an ordinary Rust graph

## Where you are

ZeroGC runs managed code but cannot decide which allocations are live. Its
native allocation registry is deliberately not a heap model.

## The problem

Root discovery, object layout, raw memory, and reclamation are separate sources
of bugs. Learn the semantic core of tracing first: given roots and directed
references, which objects are reachable?

## Observe first

Represent a tiny graph with ordinary Rust collections, draw its expected live
set by hand, and implement the most obvious traversal. Then add a cycle and a
very deep chain to expose duplicate work or recursive stack growth.

## Your challenge

- [ ] Model objects as stable logical IDs mapped to lists of outgoing logical
  IDs. Do not introduce byte offsets or regions.
- [ ] Accept a root list and return the set of reachable objects.
- [ ] Replace recursive traversal if the deep-chain test can overflow the Rust
  stack; use an explicit work collection and avoid scanning an object twice.
- [ ] Reject a root or edge naming a nonexistent object as model corruption.
- [ ] Report roots visited, unique objects discovered, edges examined, and
  maximum pending work.
- [ ] Test cycles, self-cycles, disconnected components, duplicate roots,
  duplicate edges, an empty graph, and a deep chain.

## Checkpoint

Randomly generated valid graphs produce the same reachable set as a deliberately
simple independent BFS/DFS oracle. Corrupt graphs return an error rather than a
partial success.

## Allowed shortcuts

- `HashMap`, `HashSet`, and `Vec` are encouraged.
- Object IDs may be a small wrapper or test-oriented integer; no memory-layout
  meaning is needed.
- There is no mark bitmap or collection phase yet.

## Known debt

Reachability is returned as a set and nothing is reclaimed. The graph storage
does not resemble a managed heap.

## What this unlocks

Mission 06 can turn the reachability result into a complete model collection and
make phase ordering observable.

## Hints

Mark an object as discovered before enqueueing it. Keep the independent oracle
structurally different from the implementation under test.
