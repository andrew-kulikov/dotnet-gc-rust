# Mission 15 - Establish strong and weak handle ordering

## Where you are

Runtime diagnostics can trace and walk supported real objects. The ordinary Rust
model from mission 06 provides a small place to specify handle ordering.

## The problem

Strong handles contribute roots. Weak handles must not keep objects alive and
must be cleared before their targets are reclaimed.

## Observe first

Point a strong handle and then a weak handle at an otherwise unreachable model
object. Predict the live set and the weak value after model collection.

## Your challenge

- [ ] Provide stable model handle IDs with create, read, update, and destroy
  operations; a destroyed ID must not accidentally address a later handle.
- [ ] Add strong targets to the model root input before tracing.
- [ ] Clear weak targets absent from the completed live set before reclamation.
- [ ] Keep handle identities stable when their containers grow.
- [ ] Test nulls, duplicate strong targets, destroyed handles, mutation between
  cycles, and weak targets also reachable through ordinary fields.
- [ ] Express roots/strong handles, trace, weak clearing, reclaim as an executable
  phase test and retain the graph oracle.

## Checkpoint

Generated model sequences match an independent reference. Strong targets
survive, weak-only targets clear before removal, and successful handle lookups
never expose reclaimed model objects.

## Allowed shortcuts

- Ordinary Rust containers and simple generation counters are allowed.
- Pinning, dependent handles, and finalization are separate capabilities.

## Known debt

Logical handles do not establish native slot stability or the runtime's precise
weak-handle categories.

## What this unlocks

Mission 16 connects the tested semantics to actual runtime handle operations.

## Hints

Handle coverage must be explicit before a real sweep can be enabled.
