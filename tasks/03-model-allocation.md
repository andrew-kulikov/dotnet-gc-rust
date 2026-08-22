# Iteration 03 — Model allocation

## Goal

Allocate aligned model objects and reconstruct their boundaries by walking raw arena storage.

## Assignment

- [ ] Define a compact model object header and a test-only layout descriptor for reference slots.
- [ ] Add aligned bump allocation within a region.
- [ ] Represent unused tails so the region always remains walkable.
- [ ] Implement forward walking from region start to allocation frontier.
- [ ] Add generated allocation sequences with different sizes and alignments.

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
