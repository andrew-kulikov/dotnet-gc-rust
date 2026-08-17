# Iteration 04 — Model marking

## Goal

Find all model objects reachable from a root set without recursion.

## Assignment

- Define root enumeration and reference tracing boundaries used by the model.
- Store mark state in side metadata.
- Implement iterative graph traversal with an explicit work stack.
- Produce collection statistics for roots visited, objects marked, and maximum work-stack size.
- Compare the result with a simple reference reachability implementation in tests.

## Constraints

- Do not recurse through the object graph.
- Duplicate roots and duplicate edges must not cause duplicate scanning.
- Invalid references must be reported as heap corruption, not silently ignored.

## Acceptance criteria

- Reachable cycles, self-cycles, disconnected components, duplicate roots, and very deep chains behave correctly.
- Mark bits outside allocated object starts cannot be set through safe APIs.
- Random-graph results match the reference tracer.
- A second mark operation starts from an explicitly defined clean or preserved state.

## Hints

Separate “discover object,” “set mark,” and “scan outgoing slots.” This makes later CoreCLR callback integration less tangled.

## Agent review focus

Ask the agent to verify termination, stack growth, corruption handling, and independence of the differential oracle.
