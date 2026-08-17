# Iteration 17 — Finalization

## Goal

Implement ordinary and critical finalization with correct weak-reference and resurrection ordering.

## Assignment

- Implement registration, suppression, re-registration, and f-reachable queues.
- Preserve unreachable finalizable objects long enough for the runtime finalizer thread.
- Support critical finalizers required by the pinned runtime.
- Add `samples/GcFeatures` cases for ordering, exceptions, suppression, resurrection, and repeated collections.
- Add sync-block weak-pointer scanning and eager-finalization behavior required by the supported matrix.

## Constraints

- Finalizers never execute inside the collector implementation.
- Queue mutation and object resurrection follow documented collection phases.
- Unsupported assembly-unload behavior must fail safely or be excluded explicitly.

## Acceptance criteria

- Deterministic samples agree with stock-GC observable ordering for supported cases.
- Each object is queued no more often than its registration state permits.
- Resurrected objects survive the expected collection and become collectible later.
- Weak-reference behavior before and after finalization is covered.

## Hints

Draw the phase sequence before coding: mark, dependent handles, weak processing, finalization promotion, re-marking, and later weak processing are easy to confuse.

## Agent review focus

Ask the agent to review the phase diagram against CoreCLR and find paths that can lose, double-queue, or prematurely reclaim a finalizable object.
