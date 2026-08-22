# Iteration 18 — Reclamation and v0.1

## Goal

Reuse real managed heap space safely and publish a reproducible baseline collector.

## Assignment

- [ ] Connect swept free objects to the verified segregated free lists.
- [ ] Split and coalesce free objects while maintaining the interior-pointer index.
- [ ] Decommit wholly free regions after a delay and retain a bounded reuse cache.
- [ ] Add low-memory, allocation-failure, fragmentation, and long soak workloads.
- [ ] Inventory unsafe code, supported features, limitations, and debugging procedures.
- [ ] Reproduce build and tests from a clean checkout before tagging `v0.1.0`.

## Constraints

- Prefer bounded memory and correctness over aggressive decommit behavior.
- A failed allocation may trigger collection or return failure; it may not corrupt allocator state.
- Do not claim production, real-time, or broad runtime compatibility.

## Acceptance criteria

- A steady-state allocate/drop workload reaches a bounded memory plateau.
- Heap, free-list, handle, brick-table, and byte-accounting verifiers survive the soak run.
- Reuse measurably occurs and wholly free regions can be decommitted and recommitted.
- Public documentation and CI reproduce the exact supported matrix.

## Hints

Gate decommit behind a simple conservative policy. The most valuable result is a boring, repeatable plateau, not minimal committed bytes.

## Agent review focus

Ask the agent for a release audit: clean-build reproduction, claimed feature coverage, memory plateau evidence, unsafe inventory, and repository privacy check.
