# Iteration 06 — Model handles

## Goal

Learn handle semantics in isolation before mapping CoreCLR's handle manager.

## Assignment

- [ ] Add stable handle IDs and storage for strong, weak, and dependent handles.
- [ ] Treat strong targets as roots.
- [ ] Clear weak targets when their objects are not otherwise reachable.
- [ ] Implement dependent-handle reachability to a fixed point.
- [ ] Define and test the ordering of marking, dependent processing, and weak clearing.

## Constraints

- Handle identity remains stable when storage grows.
- A dependent secondary does not keep its primary alive.
- Do not add pinning behavior to the non-moving model merely for naming symmetry.

## Acceptance criteria

- Chains and cycles of dependent handles reach the correct fixed point.
- Weak handles clear at the specified phase and never expose a reclaimed object.
- Removing or reusing handles cannot mutate a different live handle.
- Differential tests use an independently implemented fixed-point oracle.

## Hints

Write the phase ordering in prose before coding it. Fixed-point processing should report whether a pass discovered new objects.

## Agent review focus

Ask the agent to construct adversarial dependent chains, mixed strong/weak graphs, stale IDs, and handle-storage growth cases.
