# Mission 07 - Build managed graph fixtures

## Where you are

Mission 05 computes reachability, and mission 06 separates model tracing from
reclamation. ZeroGC runs bounded managed programs.

## The problem

The model needs shared scenarios with a managed program before native readers
can be checked against real object relationships.

## Observe first

Draw a chain, a cycle, and two disconnected components. For each scenario,
write the expected reachable labels for an explicit list of fixture roots.

## Your challenge

- [ ] Add bounded C# scenarios using a simple non-finalizable `Node` class with
  one reference field; build larger shapes from several nodes.
- [ ] Define stable scenario labels and expected edges independently of addresses.
- [ ] Run the same expected graphs through the Rust model and compare live sets.
- [ ] Add a fixture control point and specify the pinned-runtime contract for
  handing object references to native diagnostics and keeping them valid.
- [ ] Keep fixture construction, native observation, and managed continuation
  separately observable. Do not manufacture addresses by casting managed values.
- [ ] Run bounded scenarios under stock GC and ZeroGC, recording supported and
  explicitly unsupported diagnostic paths separately.

## Checkpoint

One reproducible command checks fixture construction and Rust-model results
for chains, cycles, disconnected components, duplicate roots, and null fields.
The managed program reaches a continuation marker after each supported run.

## Allowed shortcuts

- Fixture descriptions may be small checked-in data files.
- Managed code may export declared labels and relationships for comparison.
- Reading native object memory is not required.

## Known debt

Exported relationships describe the fixture; they do not prove that a native
reader can discover those relationships. Fixture roots are not runtime roots.

## What this unlocks

Mission 08 establishes the safe observation window for these same fixtures.

## Hints

Keep the expected graph independent of the native reader. Any native handoff
must account for rooting, GC transitions, and the lifetime of borrowed references.
