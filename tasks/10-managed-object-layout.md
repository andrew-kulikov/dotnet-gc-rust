# Iteration 10 — Managed object layout

## Goal

Calculate the size and boundaries of real managed objects for the pinned x64 runtime.

## Assignment

- [ ] Document the relevant object header, method-table pointer, alignment, string, array, and free-object layouts.
- [ ] Implement checked size calculation for ordinary objects, strings, boxed values, and arrays.
- [ ] Add `samples/ObjectLayouts` with inheritance, nested value types, multiple array forms, and boundary lengths.
- [ ] Fill abandoned allocation-context tails with valid free objects.
- [ ] Keep runtime-layout knowledge isolated from the generic collector model.

## Constraints

- Derive behavior from the pinned runtime source, not current-memory observations alone.
- Every size calculation must detect overflow before pointer arithmetic.
- Do not enumerate object references yet.

## Acceptance criteria

- Native calculations agree with independently observed fixture sizes for every supported shape.
- Walking mixed allocations stops at the exact frontier.
- Impossible method tables, lengths, sizes, and alignment states fail deterministically.
- The layout note cites the exact runtime source files and commit.

## Hints

Keep “base size,” “component size,” and final aligned size separate. Arrays near arithmetic limits are valuable tests even if they cannot be allocated normally.

## Agent review focus

Ask the agent to trace representative class, string, reference-array, and value-type-array calculations and audit signed/unsigned conversions.
