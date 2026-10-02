# Mission 16 - Integrate the required runtime handle kinds

## Where you are

The model specifies strong and weak ordering. Runtime-root diagnostics already
report which root and handle sources are covered or missing.

## The problem

CoreCLR handle storage and enumeration must participate in tracing. A successful
root scan alone cannot certify process liveness.

## Observe first

Run small managed handle fixtures and record which pinned runtime callbacks and
handle kinds startup and each scenario actually use.

## Your challenge

- [ ] Implement native stable handle slots and required create, read, update,
  destroy, and enumeration contracts for the declared workload.
- [ ] Inventory runtime-owned and collector-owned handle sources; avoid omissions
  and double counting when combining them with `GcScanRoots`.
- [ ] Seed strong-handle targets into the diagnostic trace.
- [ ] Classify required weak kinds using pinned contracts. Compute proposed weak
  clearing after complete tracing while diagnostics leave slots unchanged.
- [ ] Support pinned semantics required by the workload or reject the cycle;
  non-moving storage does not by itself prove complete pinned-root handling.
- [ ] Reject unsupported dependent, finalization-related, or other handle cases
  before any reclamation decision. Unsupported creation must not fake success.
- [ ] Test native slot stability, destruction, mutation, nulls, and model/runtime
  agreement on fixture reachability and proposed weak clearing.

## Checkpoint

Every handle kind claimed by the declared workload has passing native and
managed tests. Strong handles keep fixture graphs marked, proposed weak clearing
matches the model, and missing coverage produces an incomplete trace.

## Allowed shortcuts

- A locked container and slow scans are sufficient.
- Runtime feature completeness is not required; the supported workload must be
  explicit and executable without encountering unsupported cases.

## Known debt

Diagnostics compute weak actions but do not apply them. Reclamation still needs
one gate covering the whole heap, roots, layouts, and handle kinds together.

## What this unlocks

Mission 17 consolidates those invariants into a complete marking checkpoint.

## Hints

Addresses of native handle slots and logical graph IDs have different lifetimes
and must remain distinct.
