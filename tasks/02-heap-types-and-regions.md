# Iteration 02 — Heap types and regions

## Goal

Model a heap as typed offsets and fixed-size regions without using OS memory or CoreCLR.

## Assignment

- [ ] **Give different meanings different Rust types.** Introduce small newtypes such as
  `ObjectId`, `RegionId`, `ByteSize`, `ByteOffset`, and `Alignment` instead of passing
  every value as `usize`. An object or region ID names a logical entity; an offset is a
  position relative to the arena start; a size is a byte count; an alignment is a
  validated nonzero power of two. Safe APIs should make mistakes such as adding two
  offsets or using an object ID as a byte position impossible or conspicuous.
- [ ] **Centralize checked byte arithmetic.** Provide operations such as “align this
  offset upward,” “compute the end of `start + size`,” and “construct a range contained in
  this arena.” Use checked addition/subtraction and reject zero or non-power-of-two
  alignments. For example, aligning offset 13 to 8 should produce 16, while aligning a
  value near `usize::MAX` must return an overflow error rather than wrap to zero.
- [ ] **Build a byte-backed arena split into fixed-size regions.** Construct an arena from
  a total usable size and region size, assign each region a stable `RegionId`, and derive
  its half-open range `[start, end)`. Decide explicitly whether a partial final region is
  rejected or excluded; do not leave remainder bytes with ambiguous ownership.
- [ ] **Store region metadata by IDs and offsets, not borrowed buffer addresses.** Track
  bounds and an initial state such as unused/active without keeping `&[u8]`, `&mut [u8]`,
  or pointers into a `Vec<u8>` that may move when it grows. A lookup by `RegionId` should
  validate the ID and create only a short-lived view of the backing storage when needed.
- [ ] **Write the model invariants before allocation is added.** Crate documentation
  should state what owns the arena, whether its size may change, how region ranges cover
  it, which values are valid IDs/offsets, and which constructors establish those facts.
  These statements become the preconditions used by the allocator in iteration 03.

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
