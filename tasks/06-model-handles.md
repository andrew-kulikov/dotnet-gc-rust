# Iteration 06 — Model handles

## Goal

Learn handle semantics in isolation before mapping CoreCLR's handle manager.

## Assignment

- [ ] **Create stable, validated handle identities.** A strong or weak handle contains one
  optional target; a dependent handle contains a primary and secondary target. Growing or
  relocating the Rust container must not change a live `HandleId`, and a destroyed ID must
  not accidentally address a newly created handle. Creation, lookup, update, and destroy
  operations should report stale or wrong-kind IDs cleanly.
- [ ] **Feed strong handles into the normal root phase.** A non-null strong target keeps
  its object alive exactly like a root supplied by the model VM. Duplicate strong handles
  should not change the result, and invalid targets should be diagnosed as corruption
  rather than silently retained or ignored.
- [ ] **Clear weak handles before their target storage is reused.** A weak handle observes
  an object but does not make it live. After ordinary marking and dependent processing,
  replace the target with `None` when it is still unmarked, and prove that no public handle
  lookup can return an object after sweep has reclaimed that object.
- [ ] **Process dependent handles to a fixed point.** If a dependent handle's primary is
  live, its secondary becomes live; the secondary may then make another dependent
  primary live, requiring another pass. Continue until a complete pass discovers nothing
  new. A secondary never keeps its own primary alive, and an unreachable primary/secondary
  pair must not survive just because the handle exists.
- [ ] **Make phase ordering an explicit part of the API and tests.** Write down and encode
  the sequence: seed ordinary/strong roots, trace, process dependent handles until stable,
  clear weak handles, then sweep. Reject invalid phase transitions and test chains, cycles,
  null targets, destroyed handles, and mutations between collections.

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
