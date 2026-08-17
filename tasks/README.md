# Learning iterations

This directory turns the project roadmap into a CodeCrafters-style sequence. Each iteration introduces one observable capability and is intended to fit into one or two focused sessions. The files describe behavior and constraints without prescribing the complete implementation.

## Workflow

1. Work through the iterations in order. If an iteration exposes a design flaw, fix it before moving on.
2. Create one branch or small pull request per iteration.
3. Read the task and the matching row in the [roadmap reading sequence](../docs/RESOURCES.md#suggested-reading-sequence-by-roadmap), then try the assignment before opening the hints.
4. Update tests, invariants, and the engineering log as part of the iteration.
5. Run the self-check and commit only files relevant to the capability.
6. Ask an agent to review the result using [REVIEW_GUIDE.md](REVIEW_GUIDE.md). The agent verifies; it does not implement missing work unless explicitly asked.
7. Mark the task complete only after all required findings are resolved or recorded as an explicit limitation.

## Sequence

| Iteration | Capability | Roadmap milestone |
|---:|---|---:|
| [00](00-public-skeleton.md) | Public repository skeleton | 0 |
| [01](01-loader-boundary.md) | CoreCLR reaches the native GC boundary | 0 |
| [02](02-heap-types-and-regions.md) | Typed heap model and regions | 1 |
| [03](03-model-allocation.md) | Aligned model allocation and heap walking | 1 |
| [04](04-model-marking.md) | Graph tracing and marking | 2 |
| [05](05-model-sweeping.md) | Sweep, coalescing, and reuse | 2 |
| [06](06-model-handles.md) | Weak and dependent handles | 2 |
| [07](07-windows-virtual-memory.md) | Reserved and committed native memory | 3 |
| [08](08-standalone-gc-shell.md) | Standalone GC interface shell | 3 |
| [09](09-leak-only-managed-allocation.md) | Managed allocation without collection | 3 |
| [10](10-managed-object-layout.md) | Managed object sizing | 4 |
| [11](11-gcdesc-and-heap-walking.md) | Reference enumeration and real heap walking | 4 |
| [12](12-suspension-and-roots.md) | Suspension and root discovery | 5 |
| [13](13-real-mark-and-sweep.md) | First real collection | 5 |
| [14](14-strong-and-pinned-handles.md) | Strong and pinned runtime handles | 6 |
| [15](15-weak-and-dependent-handles.md) | Weak and dependent runtime handles | 6 |
| [16](16-interior-and-frozen.md) | Interior pointers and frozen objects | 6 |
| [17](17-finalization.md) | Finalization and resurrection | 6 |
| [18](18-reclamation-and-v01.md) | Reuse, stabilization, and v0.1 | 7 |
| [19](19-regional-policy-model.md) | Regional policy in the Rust model | 8 |
| [20](20-remembered-sets.md) | Write barrier and remembered sets | 8 |
| [21](21-frame-safe-point-integration.md) | Frame-safe-point integration | 8 |
| [22](22-benchmarks-and-v02.md) | Reproducible evaluation and v0.2 | 9 |

The calendar in `ROADMAP.md` remains the schedule. These iterations are checkpoints, not additional scope.

## Progress convention

Do not edit task files merely to record personal progress. Track completion in GitHub issues, pull requests, or `docs/weekly/`. This keeps the exercises reusable for other learners.
