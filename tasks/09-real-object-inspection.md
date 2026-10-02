# Mission 09 - Read one real managed object

## Where you are

Fixture objects can be lent to native diagnostics during a verified suspension.
The allocation registry describes owned ranges but does not determine liveness.

## The problem

Runtime objects have method tables and reference metadata, unlike model map
entries. One small supported shape is enough to test the boundary.

## Observe first

Use the fixture `Node` with one reference field, then with that field null.
Locate its size and reference-series rules in the exact pinned runtime source.

## Your challenge

- [ ] Document the supported Windows x64 object prefix, method-table metadata,
  size, alignment, and required `GCDesc` form with pinned source locations.
- [ ] Validate a supplied object start and its storage range before reading it.
  The ZeroGC registry may provide bounds, never reachability.
- [ ] Establish the runtime provenance and lifetime of metadata pointers before
  reading them; numeric plausibility alone does not validate a method table.
- [ ] Decode the supported size and enumerate the single reference slot using
  checked offsets and bounded descriptor traversal.
- [ ] Distinguish null references, unsupported layouts, invalid targets, and
  corrupt descriptors with explicit diagnostic outcomes.
- [ ] Compare actual slots and target labels with fixture expectations while
  suspended; copy only logical observations out of the window.
- [ ] Test malformed metadata through bounded synthetic buffers or an injected
  reader, without corrupting the running managed heap.

## Checkpoint

Native diagnostics read the expected reference or null from supported `Node`
instances. Unsupported or malformed input fails in bounded time, and every
post-suspend outcome follows the restart guarantee.

## Allowed shortcuts

- Support only the fixture class and the minimum object forms its inspection
  needs. Full heap walking and arrays are not required.
- The reader may be slow and use existing allocation-range diagnostics.

## Known debt

Only selected objects can be inspected. No complete runtime graph or dead-object
classification is available.

## What this unlocks

Mission 10 applies the existing reachability algorithm to real fixture links.

## Hints

Keep object validation and reference enumeration distinct from graph traversal.
