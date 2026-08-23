# Mission 04 - Make ZeroGC explain why it must be replaced

## Where you are

A bounded managed process reaches `Main` and exits normally using individually
allocated, never-collected objects.

## The problem

`Hello, World!` proves only startup. Before designing a real heap, turn ZeroGC's
limitations into reproducible evidence. Some scenarios may work accidentally;
others may fail because an interface contract or runtime feature is missing.
The project needs to know which is which.

## Observe first

Add command-line-selected, bounded scenarios to the sample instead of one large
mixed test. Start with small-object churn, one large array, two allocating
threads, a pinned buffer, and a finalizable object. Run each under stock GC, then
under ZeroGC with native diagnostics enabled.

For every failure, record the last collector method called and whether the
failure was an explicit unsupported diagnostic, deterministic exhaustion, or an
unexpected crash.

## Your challenge

- [ ] Add allocation and native-memory counters that reconcile at process exit:
  requests, successful allocations, requested bytes, owned bytes, and configured
  limit.
- [ ] Make exhaustion reproducible with a small command-line or environment
  limit. It must fail without wraparound, use-after-free, or an access violation.
- [ ] Add separate bounded scenarios for small allocations, a large object,
  multiple allocating threads, pinning, and finalization registration.
- [ ] Produce a checked-in capability note or machine-readable table recording
  `works`, `unsupported by named method`, or `unexpected defect` for each
  scenario. Do not turn an unsupported feature into a success stub merely to
  improve the table.
- [ ] Add one diagnostic showing that allocations occupy disjoint native ranges
  and that no byte-wise forward heap walk is currently possible.
- [ ] Keep the original minimal smoke green while the exploratory scenarios run
  independently.

## Checkpoint

One command runs the scenario matrix under stock GC and ZeroGC. The minimal
custom-GC scenario still exits normally, the configured exhaustion case always
fails the same way, counters reconcile, and every non-working feature has a
named reason rather than an unexplained crash.

## Allowed shortcuts

- Scenarios other than the minimal smoke may remain unsupported.
- Diagnostics may use a slow, locked metadata registry.
- No collection, byte heap, region design, or production performance is needed.

## Known debt

The allocation registry may describe objects, but it is not a walkable heap and
must not become the collector's liveness oracle. The prototype still cannot
reclaim memory.

## What this unlocks

The inability to reclaim safely motivates a collector model. Mission 05 starts
with ordinary Rust graph data so reachability can be learned without CoreCLR,
raw memory, or the temporary ZeroGC registry.

## Hints

Treat an explicit unsupported result as useful evidence. Keep workloads small
enough that a leak is bounded and every run is suitable for CI.
