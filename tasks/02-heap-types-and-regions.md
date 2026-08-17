# Iteration 02 — Heap types and regions

## Goal

Model a heap as typed offsets and fixed-size regions without using OS memory or CoreCLR.

## Assignment

- Introduce distinct types for object IDs, region IDs, byte sizes, alignments, and offsets.
- Implement checked alignment and address-range calculations.
- Model a contiguous arena divided into equal regions.
- Track each region's bounds and state without storing references into a growable byte buffer.
- Define the minimum invariants in crate documentation.

## Constraints

- Safe APIs must not accept interchangeable bare `usize` values where unit confusion is possible.
- Invalid alignment, overflow, and out-of-range access must return errors rather than panic.
- Keep this crate independent of CoreCLR and Windows.

## Acceptance criteria

- Unit tests cover zero, boundary, misalignment, overflow, final-region, and oversized-range cases.
- No two regions overlap and their union matches the arena's usable range.
- The FFI-free crate passes its normal tests and Miri-compatible tests.
- Public safe APIs cannot construct an invalid region range.

## Hints

Make illegal states difficult to express, but do not create a trait for every integer wrapper. Checked arithmetic is part of the domain model, not defensive decoration.

## Agent review focus

Ask the agent to try to construct invalid values through public APIs and to audit every addition, multiplication, and alignment operation for overflow.
