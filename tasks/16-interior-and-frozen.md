# Iteration 16 — Interior pointers and frozen objects

## Goal

Resolve interior roots efficiently and treat frozen segments as external, permanently live memory.

## Assignment

- [ ] Build a region-local brick or index table that maps an interior address to its containing object.
- [ ] Update the index whenever object boundaries change.
- [ ] Implement frozen-segment registration, unregistration if required by the pinned contract, and managed-range checks.
- [ ] Ensure frozen objects are traversed when necessary but are never marked in place or swept.
- [ ] Add samples using spans, pinned interiors, string literals, and references from frozen objects.

## Constraints

- Interior lookup must reject free space, headers, one-past-end pointers, and addresses outside collector ownership as required by root flags.
- Frozen-range checks must not become a linear scan on hot mark paths.
- Never write collector metadata into frozen memory.

## Acceptance criteria

- Interior roots preserve exactly their containing objects across forced collections.
- Brick/index queries agree with a slow heap-walk oracle on randomized layouts.
- Frozen fixtures run without writes to frozen segments and their outgoing references remain live.
- Registration overlap and invalid-range cases fail safely.

## Hints

Define pointer boundary semantics explicitly before building the index. Keep a slow resolver available in debug verification.

## Agent review focus

Ask the agent to test every address around object and region boundaries and to audit all paths for accidental writes into frozen memory.
