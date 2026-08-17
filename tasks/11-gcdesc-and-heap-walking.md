# Iteration 11 — GCDesc and heap walking

## Goal

Walk the real heap and enumerate every managed reference slot described by CoreCLR metadata.

## Assignment

- Implement forward walking across every committed collector segment.
- Decode both positive and negative `GCDesc` forms used by the pinned runtime.
- Expose a callback-based reference-slot enumerator that does not allocate in the hot path.
- Expand `samples/ObjectLayouts` with reference arrays, inheritance, nested structs, and arrays of structs containing references.
- Add a diagnostic comparison against runtime tooling or a deliberately independent fixture oracle.

## Constraints

- DAC may validate results but cannot be required for collection correctness.
- Metadata decoding must have strict bounds and progress checks.
- Do not retain Rust references into managed memory across callbacks.

## Acceptance criteria

- Each fixture's outgoing reference slots are found exactly once at the correct addresses.
- Empty layouts and every supported `GCDesc` form are covered.
- Heap walking detects corrupt sizes or impossible metadata before leaving committed ranges.
- The implementation documents all unsafe aliasing and lifetime assumptions.

## Hints

Treat descriptor bytes as untrusted input even though CoreCLR produced them. First test decoding against synthetic byte sequences, then real method tables.

## Agent review focus

Ask the agent to compare decoding with the pinned CoreCLR implementation and probe malformed counts, negative-series arithmetic, and segment boundaries.
