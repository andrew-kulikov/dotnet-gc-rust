# Mission 07 - Discover strong and weak handle ordering

## Where you are

The graph model has separate tracing and reclamation phases.

## The problem

References stored outside ordinary object fields do not all have the same
liveness semantics. A strong handle must behave like a root. A weak handle must
not keep its target alive, but it also must not expose an ID after the target has
been reclaimed.

## Observe first

Add one otherwise unreachable object and point both a strong and a weak handle
at it in separate tests. Try clearing the weak handle after reclamation and
observe why that ordering is too late.

## Your challenge

- [ ] Provide stable handle identities with create, read, update, and destroy
  operations. A stale destroyed ID must not address a later handle accidentally.
- [ ] Add strong-handle targets to the root input before ordinary tracing.
- [ ] Inspect weak targets after marking but before reclamation; clear a target
  that is not in the completed live set.
- [ ] Ensure handle-container growth cannot change the identity of a live handle.
- [ ] Test null targets, duplicate strong targets, destroyed handles, handle
  mutation between collections, and a weak target reachable through fields.
- [ ] Write the discovered order as an executable phase test:
  roots/strong handles, trace, weak clearing, reclaim.

## Checkpoint

Strong targets survive, weak-only targets are cleared and reclaimed, and no
successful handle lookup returns an object ID absent from the model graph.

## Allowed shortcuts

- Strong and weak handles may use separate ordinary Rust containers.
- Pinning and dependent handles are explicitly out of scope.

## Known debt

Stable logical IDs do not yet model stable native handle addresses. Weak-handle
ordering will become more complex around finalization later.

## What this unlocks

The semantic model is strong enough to survive a change in storage. Mission 08
can replace map entries with self-describing byte records without changing the
meaning of liveness.

## Hints

A generation counter in a logical handle ID is one way, but not the only way, to
detect stale slot reuse. Choose only what the tests require.
