# Mission 09: reading a real managed object

[Mission 09](../../tasks/09-real-object-inspection.md) inspects one fixture `Node`
during the suspension established by [Mission 08](../08-suspension-lifecycle/README.md).
The reader obtains the object size and reference slot from runtime metadata,
reads the actual target, and compares it with an independent fixture expectation.
Only logical observations survive restart.

**Status:** the checkpoint passed on the pinned Windows x64 runtime. The live
probe observed both a reference and null with size 24 and slot offset 8; it also
checked an intentional mismatch and worker progress after restart. Synthetic
tests cover malformed input; they are separate from the runtime evidence.

Run `python scripts/graph_fixtures.py --fixture chain --inspect` for the focused
checkpoint. It builds the fixture and ZeroGC, displays their output, checks
process exits, and saves logs under `out/graph-fixtures/`. The managed sample
checks the returned inspection and restart results.

1. [Object layout](01-object-layout.md): prefix, size, alignment, and flags.
2. [Bounds and metadata provenance](02-metadata-and-bounds.md): what makes
   reads valid and what the allocation ledger does not prove.
3. [GCDesc](03-gcdesc.md): the one-series encoding and bounded decoding rules.
4. [Suspended inspection](04-suspended-inspection.md): handle ownership,
   identity comparison, outcomes, and restart.
5. [Plan and checkpoint](05-plan-and-checkpoint.md): sequential steps, required
   tests, actual results, and remaining work.

The supported target is the fixture's sealed `Node` on Windows x64 workstation
CoreCLR 10.0.11. One positive descriptor series containing one reference slot is
in scope. Graph traversal, arrays, repeating descriptors, runtime roots, heap
walking, marking, and reclamation belong to later missions.
