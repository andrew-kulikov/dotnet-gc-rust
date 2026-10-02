# Mission 17 - Complete marking before authorizing sweep

## Where you are

The stopped heap is walkable and supported runtime roots and handles feed
tracing. The model supplies independent reachability and ordering expectations.

## The problem

A collection may mutate memory only after every required liveness source and
object layout has been processed. A partial diagnostic result is insufficient.

## Observe first

Run a graph with known retained objects and deliberately detached objects.
Remove observation-only strong references before tracing; allow for JIT local
lifetimes when designing independent expectations.

## Your challenge

- [ ] Verify the full heap and all current object boundaries while suspended.
- [ ] Enumerate every root and handle source required by the supported runtime
  configuration, recording flags and applying their supported semantics.
- [ ] Trace validated slots directly with an explicit work stack and side marks;
  preserve differential tests against diagnostic snapshots and the model.
- [ ] Use bounded or preallocated traversal storage with a defined failure path.
- [ ] Produce a completed-mark result only if heap, metadata, roots, handles,
  flags, and traversal all succeed. Keep partial results unusable by sweep.
- [ ] Compute proposed weak clearing after marking and report candidate dead
  ranges only for a complete cycle; do not mutate slots or object memory yet.
- [ ] Clear or version marks for repeated cycles and report phase counters.
- [ ] Inject unsupported cases, corruption, and budget failures; suppress dead-set
  claims and preserve the suspension/restart guarantee.

## Checkpoint

Repeated managed runs retain every known-live fixture object. Complete cycles
identify independently expected dead fixtures; every incomplete cycle produces
no sweep authorization. The heap remains unchanged and execution resumes.

## Allowed shortcuts

- Expensive full validation and a fixed bounded managed workload are allowed.
- Unsupported runtime capabilities may prevent completion until implemented;
  narrowing a fixture must never silently exclude runtime-created objects.

## Known debt

This is a read-only marking checkpoint. It establishes eligibility for the
supported workload, not general runtime compatibility.

## What this unlocks

Mission 18 applies weak clearing and sweep only to a completed mark result
within the same uninterrupted suspended phase.

## Hints

Root slots and completion evidence cannot be cached across a restart or graph
mutation. The next suspension requires a fresh mark.
