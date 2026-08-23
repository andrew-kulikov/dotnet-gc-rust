# Iteration 03 — Model allocation

## Goal

Allocate aligned model objects and reconstruct their boundaries by walking raw arena storage.

## Assignment

- [ ] **Define a self-describing object format for the model heap.** Give every allocated
  object a compact encoded header containing enough information for a walker to find the
  next object, such as total object size and a model layout identifier. Keep a separate
  test-VM layout descriptor that says which payload slots contain `ObjectId` references;
  this is a teaching format and must not pretend to be CoreCLR's real object header.
- [ ] **Allocate by advancing a per-region frontier.** Given payload size and alignment,
  calculate the aligned object start and checked end, ensure the whole encoded object fits
  in one region, initialize it, and only then commit the new frontier. A failed request
  must leave both storage and frontier observably unchanged.
- [ ] **Make padding and abandoned tails representable.** Alignment gaps and space left
  when a region can no longer satisfy a request cannot be mysterious bytes that stop a
  future heap walk. Encode them as a valid padding/free record, or define another explicit
  representation that lets the walker advance over every byte up to the frontier.
- [ ] **Walk objects from storage rather than allocator bookkeeping.** Starting at a
  region boundary, decode the current header, validate size/alignment/range, yield the
  object, and advance by its encoded size until the frontier. Reject zero progress,
  truncated headers, impossible sizes, and records crossing the region boundary.
- [ ] **Generate varied allocation histories.** Property tests should mix small and
  boundary-sized objects, different valid alignments, nearly full regions, and failed
  requests. Keep an expected list only in the test oracle and compare it with the objects
  reconstructed by the real walker.

## Constraints

- The walker may not rely on a side list of allocated objects.
- Allocation must be failure-atomic: an unsuccessful request does not advance the frontier.
- Do not copy CoreCLR object layout into this model.

## Acceptance criteria

- Allocated ranges are aligned, non-overlapping, and contained in one region.
- Walking reconstructs every allocated object in order and stops exactly at the frontier.
- Corrupt sizes and impossible headers are rejected without an unbounded loop.
- Property tests cover random valid allocation sequences and malformed storage.

## Hints

Start with explicit encoding and decoding functions. Keep an independent expected-object list inside tests only; it is an oracle, not implementation state.

## Agent review focus

Ask the agent to inspect parser progress guarantees, allocation rollback, zero-size behavior, and whether generated tests use a genuinely independent oracle.
