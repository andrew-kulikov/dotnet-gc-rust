# Iteration 14 — Strong and pinned handles

## Goal

Preserve objects referenced by strong and pinned runtime handles with correct non-moving semantics.

## Assignment

- [ ] Implement creation, update, enumeration, and destruction for strong and pinned handles.
- [ ] Add these targets to the appropriate root-processing phase.
- [ ] Preserve stable handle identity as handle storage grows.
- [ ] Extend `samples/GcFeatures` with normal handles and pinned buffers used across forced collections.
- [ ] Verify that pinned flags and addresses are reported consistently even though the baseline collector never moves objects.

## Constraints

- A destroyed handle must not remain a root.
- Reusing a handle slot must not let a stale identifier mutate the new handle.
- Pinning behavior must match the runtime contract rather than becoming a no-op hidden behind the non-moving design.

## Acceptance criteria

- Strong and pinned targets survive while their handles exist and become collectible afterwards.
- Pinned buffer addresses remain stable across collections.
- Concurrent handle-table operations follow a documented synchronization rule.
- Growth, deletion, slot reuse, and forced-GC stress cases pass.

## Hints

Separate stable external identity from internal storage position. Document whether generations, epochs, or non-reused IDs prevent stale access.

## Agent review focus

Ask the agent to audit handle lifetime races, stale IDs, storage relocation, and the precise phase in which handle targets become roots.
