# Mission 15 - Derive real managed object boundaries

## Where you are

CoreCLR allocates into collector-owned thread contexts. Diagnostics know the
allocated ranges, but the collector cannot reconstruct object boundaries after
that temporary bookkeeping is gone.

## The problem

Real managed objects do not use the teaching header from the Rust model. Their
size depends on pinned CoreCLR method-table metadata and, for strings and arrays,
length and component size. Guessing from observed memory would make a corrupted
length turn into an unbounded walk.

## Observe first

Create one managed fixture per shape and independently inspect its expected size:
ordinary class, inherited class, boxed value, string, reference array, and
value-type array. Locate the exact pinned runtime source used to derive each
field and rule.

## Your challenge

- [ ] Document the supported x64 object prefix, method-table pointer, base size,
  component size, string/array length, final alignment, and free-object format
  with pinned source locations.
- [ ] Decode and validate one ordinary object first, then add supported shapes
  one failing fixture at a time.
- [ ] Check every signed/unsigned conversion, multiplication, addition, and
  alignment before pointer arithmetic or memory access.
- [ ] Retire allocation-context tails as valid managed free objects so a stopped
  context no longer leaves ambiguous bytes.
- [ ] Walk each committed managed range to its exact known frontier and stop on
  corrupt metadata with a bounded diagnostic.
- [ ] Keep runtime layout knowledge outside the generic model algorithms.

## Checkpoint

Native calculations match independent fixtures for every claimed shape, mixed
allocations walk exactly to the frontier, and impossible method tables, lengths,
sizes, alignments, and truncated tails fail deterministically.

## Allowed shortcuts

- Support one pinned Windows x64 runtime only.
- Unsupported object shapes may fail before collection.
- Reference fields do not need to be enumerated yet.

## Known debt

The walker knows where objects end but not which words inside them are managed
references.

## What this unlocks

Mission 16 can decode reference metadata and turn real managed objects into the
same “visit outgoing reference” boundary used by the model tracer.

## Hints

Keep base size, variable component bytes, and final aligned size as separate
checked steps. Test arithmetic limits even when normal managed code cannot
allocate such an object.
