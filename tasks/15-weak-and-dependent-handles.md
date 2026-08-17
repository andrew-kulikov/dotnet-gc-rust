# Iteration 15 — Weak and dependent handles

## Goal

Implement runtime weak-reference clearing and dependent-handle fixed-point semantics.

## Assignment

- Add short and long weak-handle processing in their correct collection phases.
- Add dependent-handle scanning until no new secondary target becomes reachable.
- Extend `samples/GcFeatures` with weak references and `ConditionalWeakTable` cases.
- Test chains, cycles, resurrection interactions that are already supported, and handle mutation between collections.
- Record fixed-point pass counts and weak handles cleared for diagnostics.

## Constraints

- A dependent secondary cannot keep an unreachable primary alive.
- No weak handle may expose storage after it has been reclaimed.
- Do not approximate short and long weak handles as identical.

## Acceptance criteria

- Results agree with the stock GC for deterministic feature samples.
- Long dependent chains converge and mixed reachable/unreachable chains resolve correctly.
- Weak ordering is stated as an invariant and tested around the supported finalization boundary.
- Handle-table verification succeeds after repeated creation, deletion, and collection.

## Hints

Reuse the phase model from iteration 06, but confirm every ordering assumption against the pinned runtime rather than copying the model blindly.

## Agent review focus

Ask the agent to compare collection-phase ordering with CoreCLR and construct dependent graphs designed to require multiple passes.
