# Iteration 07 — Windows virtual memory

## Goal

Reserve a stable address range and control commitment through a small RAII abstraction.

## Assignment

- [ ] Add a Windows implementation for reserve, commit, decommit, and release.
- [ ] Keep the API injectable so most collector-core tests remain platform independent.
- [ ] Track reserved and committed ranges with checked page alignment.
- [ ] Add a guard-page test mode for detecting overruns.
- [ ] Document ownership, thread-safety, and lifetime invariants for the raw address range.

## Constraints

- Only committed pages may be dereferenced.
- Drop must release exactly the owned reservation and must not panic.
- OS error codes must be preserved in diagnostic errors.

## Acceptance criteria

- Tests can reserve a range larger than initially committed, commit selected pages, decommit them, and recommit them.
- Committed-byte accounting matches queried state.
- Invalid, overlapping, and out-of-reservation requests fail safely.
- Every unsafe operation has a checked precondition and a local `SAFETY` explanation.

## Hints

Keep addresses as raw pointers or non-null wrappers; avoid creating long-lived Rust slices over partially committed memory.

## Agent review focus

Ask the agent to review page rounding, partial failure, double release, Send/Sync decisions, and whether safe methods can create aliased mutable access.
