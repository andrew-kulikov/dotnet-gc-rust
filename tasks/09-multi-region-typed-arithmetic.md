# Mission 09 - Let a second region break ambiguous arithmetic

## Where you are

A single byte arena supports bump allocation, walking, and model tracing. Its
cursor, sizes, alignments, and positions are all represented by `usize`.

## The problem

With multiple regions, a number may mean a region index, an arena-relative
offset, a region-relative offset, or a byte count. Swapped arguments still
compile, and `start + size` can wrap. Introduce stronger types now because the
working one-region implementation demonstrates where ambiguity actually occurs.

## Observe first

Sketch a naive two-region API using only `usize`. Write tests or small examples
showing that a region number can be passed where a size is expected and that a
region-relative offset can be mistaken for an arena-relative offset.

## Your challenge

- [ ] Split the arena into equal non-overlapping regions and define what happens
  to a remainder smaller than one region.
- [ ] Introduce distinct types only for meanings now proven ambiguous, such as
  `RegionId`, `ByteOffset`, `ByteSize`, and validated `Alignment`.
- [ ] Centralize checked `start + size`, containment, intersection, and
  align-up operations. Reject zero/non-power-of-two alignment and overflow.
- [ ] Derive half-open region ranges `[start, end)` and prevent an allocation or
  walked record from crossing its owning region.
- [ ] Store long-lived metadata as IDs and offsets, not slices or pointers into a
  growable byte buffer.
- [ ] Keep all mission 08 allocation, walking, corruption, and marking tests
  working across region boundaries.

## Checkpoint

Tests cover alignment of 13 to 8, invalid alignments 0 and 6, arithmetic near
`usize::MAX`, first/last region boundaries, invalid region IDs, oversized
records, and complete non-overlapping arena coverage. Miri-compatible model tests
pass.

## Allowed shortcuts

- The arena may have a fixed size for its lifetime.
- One region may be active for allocation at a time.
- Do not create a trait or newtype for every integer; protect only demonstrated
  unit or coordinate mistakes.

## Known debt

Model offsets are not native pointers. The future Windows heap may reuse range
logic, but it must not pretend Rust offsets and CoreCLR addresses are identical.

## What this unlocks

Mission 10 can rewrite dead byte records without risking accidental overlap with
a neighboring region.

## Hints

An offset names a position; a size names a quantity. Adding two positions is
usually meaningless, while adding a checked size to a position may be valid.
