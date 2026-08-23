# Mission 08 - Reconstruct a one-region byte heap

## Where you are

Reachability and basic handle semantics work with ordinary Rust collections.
The model still gets object boundaries for free from `HashMap` entries.

## The problem

A real collector sees memory, not a Rust collection of object values. It must
advance from one object to the next even after allocator bookkeeping is gone.
Build the smallest byte representation that makes that possible before adding
multiple regions or reusable holes.

## Observe first

Write down the minimum information needed to start at byte zero, validate one
record, and find the following record. Keep a test-only expected allocation list,
then prove a walker can reproduce it without reading that list.

## Your challenge

- [ ] Add one fixed-size `Vec<u8>` arena and one bump cursor. Plain `usize`
  positions are acceptable in this mission.
- [ ] Define a teaching object header with an encoded total size and a model
  layout identifier. Keep it unrelated to CoreCLR's real layout.
- [ ] Allocate an aligned record only when its checked end fits; a failed request
  must not advance the cursor or partially initialize storage.
- [ ] Represent alignment padding in a form the walker can skip safely, or adopt
  an allocation rule that proves no ambiguous gap can occur.
- [ ] Walk from the arena start to the cursor by decoding bytes. Reject truncated
  headers, zero progress, impossible sizes, and a record crossing the frontier.
- [ ] Use layout descriptors to store and enumerate logical reference slots so
  the existing graph marker can operate on walked objects.

## Checkpoint

Generated allocation sequences are reconstructed exactly by the byte walker,
then participate in the existing marking tests. Corrupt byte sequences fail in
bounded time.

## Allowed shortcuts

- There is one arena and one allocation frontier.
- Tests may use an expected side list as an oracle only.
- No free records, regions, or native pointers are required.

## Known debt

Every byte-domain value is still a `usize`; this is tolerable while there is
only one coordinate system. Space cannot be reused.

## What this unlocks

Mission 09 adds a second coordinate system. The resulting ambiguity supplies the
reason to introduce region IDs, offsets, sizes, alignments, and checked ranges.

## Hints

Make encoding and decoding separate functions. Every successful decode must
prove that the next cursor is strictly greater than the current cursor.
