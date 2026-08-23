# Learning missions

This directory is a living, experiment-driven path through the project. It is
not a decomposition of a finished collector design. A mission should introduce
one observable problem, permit the smallest correct solution, and explain why
the following mission may need to replace part of that solution.

## How to use the missions

1. Start from the checkpoint produced by the previous mission.
2. Reproduce the problem under **Observe first** before designing a solution.
3. Implement only enough behavior to reach the new checkpoint.
4. Use the shortcuts explicitly allowed by the mission. Temporary code is not a
   failure when its limitations are documented and tested.
5. Record surprising observations and changed assumptions in the engineering
   log or pull request.
6. Run the checkpoint and relevant safety checks, then request a review using
   [REVIEW_GUIDE.md](REVIEW_GUIDE.md).
7. Mark challenge items complete only after the checkpoint passes and required
   findings are resolved.

Do not implement later missions early merely because their likely data
structures are visible from the roadmap. In particular, do not introduce
regions, free-list buckets, handle algorithms, or policy traits until a current
mission creates a problem that they solve.

## Mission format

Each mission answers these questions:

- **Where you are:** what the previous checkpoint guarantees.
- **The problem:** the concrete limitation that is now visible.
- **Observe first:** evidence to gather before changing code.
- **Your challenge:** observable behavior to implement.
- **Checkpoint:** the command and result that prove progress.
- **Allowed shortcuts:** deliberately temporary simplifications.
- **Known debt:** what the solution is expected not to handle yet.
- **What this unlocks:** why the next mission becomes meaningful.

Hints are not requirements. Read them after making an initial attempt.

## Current sequence

### Phase 1 - reach managed code quickly

| Mission | Observable result |
| ---: | --- |
| [00](00-reproducible-red-baseline.md) | Stock GC succeeds; custom GC fails in one known way |
| [01](01-loader-boundary.md) | CoreCLR crosses C++ and Rust boundaries before deliberate failure |
| [02](02-interface-shell.md) | Initialization succeeds; the first unsupported call fails by name |
| [03](03-zero-gc-hello-world.md) | `LoaderSmoke` reaches `Main` under a deliberately poor ZeroGC |
| [04](04-zero-gc-limits.md) | Workloads expose and document why ZeroGC cannot become the collector |

### Phase 2 - learn tracing without runtime memory

| Mission | Observable result |
| ---: | --- |
| [05](05-graph-reachability.md) | A plain Rust graph reports exactly the reachable objects |
| [06](06-model-collection.md) | Unreachable graph objects are removed in a complete model cycle |
| [07](07-model-strong-and-weak-handles.md) | Strong and weak handle ordering is demonstrated in the model |

### Phase 3 - discover heap representation through refactoring

| Mission | Observable result |
| ---: | --- |
| [08](08-single-region-byte-heap.md) | A one-region byte heap can be reconstructed without an allocation list |
| [09](09-multi-region-typed-arithmetic.md) | A second region forces checked ranges and distinct byte-domain types |
| [10](10-linear-reuse.md) | Dead records become reusable through the simplest linear search |
| [11](11-fragmentation-and-free-lists.md) | A measured fragmentation case motivates coalescing and size buckets |
| [12](12-dependent-handles.md) | A failing handle chain motivates fixed-point processing |

### Phase 4 - replace the prototype with a real managed heap

These missions are more provisional than phases 1-3. Revise their details when
earlier experiments invalidate an assumption; preserve only their observable
outcomes.

| Mission | Observable result |
| ---: | --- |
| [13](13-reserved-managed-heap.md) | ZeroGC's per-object allocation is replaced by one reserved address range |
| [14](14-allocation-contexts.md) | Managed threads allocate through bounded allocation contexts |
| [15](15-managed-object-sizing.md) | The collector derives checked sizes for supported managed shapes |
| [16](16-gcdesc-reference-enumeration.md) | The collector enumerates real outgoing managed references |
| [17](17-suspension-lifecycle.md) | The runtime can always restart after a collector-requested suspension |
| [18](18-runtime-roots-and-marking.md) | Runtime roots drive a real mark phase and dead-object diagnosis |
| [19](19-real-sweep.md) | Dead managed objects become valid walkable free records |

## Work intentionally not scheduled yet

Runtime handle completeness, interior pointers, frozen segments, finalization,
real-heap reuse and stabilization, regional collection, remembered sets,
frame-safe-point integration, and benchmark releases remain project directions,
not current missions. Add their mission files only after mission 19 passes and
the implementation reveals the actual constraints they must address.
