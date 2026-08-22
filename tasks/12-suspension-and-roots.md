# Iteration 12 — Suspension and roots

## Goal

Stop managed execution, enumerate real roots, and always restart execution safely.

## Assignment

- [ ] Wrap execution-engine suspension in an RAII state guard.
- [ ] Fix or retire active allocation contexts before heap traversal.
- [ ] Adapt `GcScanRoots` callbacks into the Rust tracing boundary.
- [ ] Record root kind and flags needed by later pinning/interior-pointer work.
- [ ] Add a multithreaded sample that keeps a known object graph alive through stack and static roots.
- [ ] Inject failures after suspension to exercise restart paths.

## Constraints

- The guard must never permit allocation or collection re-entry in an invalid phase.
- Callback pointers are valid only under the runtime's documented suspension contract.
- Diagnostics during suspension must avoid managed allocation.

## Acceptance criteria

- Known roots are observed across multiple threads, statics, and exception paths.
- Execution resumes after every successful test and every injected error after suspension.
- Allocation-context tails remain walkable after retirement.
- Double suspend, double restart, and invalid phase transitions are rejected.

## Hints

Model collection phases explicitly. Make “restart attempted” observable in tests instead of trusting `Drop` by inspection.

## Agent review focus

Ask the agent to enumerate every exit after suspension and prove which guard owns restart responsibility on that path.
