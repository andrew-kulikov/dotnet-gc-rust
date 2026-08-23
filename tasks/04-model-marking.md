# Iteration 04 — Model marking

## Goal

Find all model objects reachable from a root set without recursion.

## Assignment

- [ ] **Define how the marker receives roots and outgoing references.** A root source
  enumerates starting `ObjectId` values without exposing the marker's storage. A tracer
  uses the model layout descriptor from iteration 03 to enumerate reference slots of one
  validated object. Keep these boundaries simple enough that CoreCLR callbacks can later
  be adapted to the same “visit reference” shape.
- [ ] **Keep liveness bits outside object payloads.** Add side metadata indexed only by
  valid object starts or stable object IDs. Marking must not overwrite the teaching
  object's header or fields, and safe APIs must reject attempts to mark padding, free
  records, or arbitrary byte offsets.
- [ ] **Trace the graph with an explicit work stack.** For each root, validate and mark a
  previously unseen object, push it, then repeatedly pop objects and discover their
  outgoing references. Mark before pushing so duplicate roots, duplicate edges,
  self-references, and cycles terminate without duplicate scans. Do not use recursive Rust
  calls for graph depth.
- [ ] **Return observable work statistics.** Report roots visited, unique objects marked,
  reference slots examined, and maximum work-stack length. Define whether invalid roots
  or edges produce partial statistics or no result, and keep that behavior deterministic.
- [ ] **Build an independent reachability oracle.** In tests, represent the same graph
  using ordinary Rust maps/sets and compute reachability with a deliberately simple BFS or
  DFS that does not reuse the heap walker or mark bitmap. Random graphs must produce the
  same live-object set in both implementations.

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
