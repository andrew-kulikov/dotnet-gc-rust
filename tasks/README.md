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

## Learning sequence

### Phase 1 - observable managed execution

| Mission | Observable result |
| ---: | --- |
| [00](00-reproducible-red-baseline.md) | Stock GC succeeds; custom GC fails in one known way |
| [01](01-loader-boundary.md) | CoreCLR crosses the C++ and Rust boundaries |
| [02](02-interface-shell.md) | Initialization succeeds; unsupported calls fail by name |
| [03](03-zero-gc-hello-world.md) | A bounded managed program reaches Main |
| [04](04-zero-gc-limits.md) | Workloads expose ZeroGC's capabilities and limits |

### Phase 2 - model semantics and real fixture tracing

| Mission | Observable result |
| ---: | --- |
| [05](05-graph-reachability.md) | A Rust graph reports exactly the reachable objects |
| [06](06-model-collection.md) | Model marking completes before unreachable entries are removed |
| [07](07-managed-graph-fixtures.md) | C# and Rust scenarios share independently expected graphs |
| [08](08-suspension-lifecycle.md) | Diagnostic suspension always has exactly one restart attempt |
| [09](09-real-object-inspection.md) | Native code reads one supported real object's links |
| [10](10-fixture-tracing.md) | Real fixture graphs match model reachability from explicit roots |

### Phase 3 - runtime diagnostics and authoritative storage

| Mission | Observable result |
| ---: | --- |
| [11](11-managed-object-shapes.md) | Sizes and reference slots match supported managed shapes |
| [12](12-runtime-root-tracing.md) | Runtime-root diagnostics report reachability and coverage |
| [13](13-reserved-managed-heap.md) | Managed allocation uses owned committed storage |
| [14](14-managed-heap-walking.md) | The stopped heap is reconstructed from bytes |

### Phase 4 - complete marking and reclamation

| Mission | Observable result |
| ---: | --- |
| [15](15-model-strong-and-weak-handles.md) | Model handles demonstrate roots/trace/weak/reclaim ordering |
| [16](16-runtime-handles.md) | Required native handles participate in runtime diagnostics |
| [17](17-complete-marking.md) | A complete phase-local mark result establishes sweep eligibility |
| [18](18-real-sweep.md) | Supported weak clearing and sweep preserve heap walkability |
| [19](19-allocation-contexts.md) | Thread contexts preserve verified collection while reducing contention |

## Diagnostic and collection boundaries

- Mission 07's declared fixture edges are an independent expectation, not evidence
  of successful native memory reading.
- Mission 10 establishes reachability relative to explicit fixture roots.
- Missions 11-12 expand runtime coverage and explicitly report incomplete passes.
- Allocation records may validate addresses and aid comparisons; they never
  establish liveness. Mission 14 makes heap bytes authoritative for inventory.
- Mission 17 requires complete coverage for the declared runtime workload.
  Unknown roots, flags, shapes, or handle kinds prevent sweep authorization.
- Mission 18 mutates weak slots and object bytes only after a complete mark in
  the same uninterrupted suspension. It does not reuse reclaimed memory.
- Suspension uses the required allocation-context contracts from mission 08;
  mission 19 adds the allocation fast path.

Later missions specify observable outcomes. Implement the smallest supported
workload first and extend it using failing fixtures. Preserve explicit failure
instead of claiming support that tests do not establish.

## Storage and handle laboratories

These focused experiments are available when a concrete implementation problem
calls for them. They are not prerequisites for tracing real fixture objects.

| Laboratory | Prerequisites | Observable result |
| --- | --- | --- |
| [01: byte heap](labs/01-single-region-byte-heap.md) | Missions 05-06 | A byte walker reconstructs model records |
| [02: region arithmetic](labs/02-multi-region-typed-arithmetic.md) | Lab 01 | Multiple regions expose and constrain coordinate mistakes |
| [03: linear reuse](labs/03-linear-reuse.md) | Lab 02 | A dead range becomes walkable and reusable |
| [04: fragmentation](labs/04-fragmentation-and-free-lists.md) | Lab 03 | Measurements justify coalescing and a rebuildable index |
| [05: dependent handles](labs/05-dependent-handles.md) | Missions 06 and 15 | Dependent chains converge to the oracle's fixed point |

Use the same observe, implement, verify, and review workflow for laboratories.
Model record formats and logical handles do not establish native compatibility.

## Work intentionally not scheduled yet

Broader runtime handle coverage, interior/frozen cases, finalization and
resurrection, native free-space reuse, dependent runtime handles, regional
collection, remembered sets, and benchmark releases require separate missions
driven by managed fixtures and measurements. Any such feature encountered by a
current collection must be supported correctly or block that collection.
