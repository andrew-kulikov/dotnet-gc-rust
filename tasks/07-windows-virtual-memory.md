# Iteration 07 — Windows virtual memory

## Goal

Reserve a stable address range and control commitment through a small RAII abstraction.

## Assignment

- [ ] **Wrap Windows virtual-memory operations behind one owning type.** Reserving chooses
  a stable virtual address range without making its pages readable; committing makes a
  page-aligned subrange accessible; decommitting removes physical backing while retaining
  the reservation; releasing gives the entire reservation back to the OS. Translate
  `VirtualAlloc`/`VirtualFree` failures into errors that retain the Windows error code.
- [ ] **Separate the interface from the Windows implementation.** Define the smallest
  injectable abstraction needed by the collector so `gc-core` tests can use a deterministic
  fake and `gc-platform` can provide the real Windows backend. Do not make generic heap
  algorithms depend on `HANDLE`, Windows constants, or system calls.
- [ ] **Validate and account for page ranges.** Query or record page/allocation granularity,
  round only according to a documented rule, and reject overflow, empty ranges where
  unsupported, overlaps with an invalid state, or subranges outside the reservation.
  Track which pages are committed so committed-byte totals can be recomputed and checked.
- [ ] **Add a guard-page test configuration.** Leave at least one known page inaccessible
  next to a committed test range and prove that an intentional overrun is caught by the
  operating system in an isolated child process. The normal test runner must survive and
  distinguish the expected access violation from an unrelated crash.
- [ ] **Document the unsafe ownership contract.** State which object exclusively releases
  the reservation, when raw addresses may be dereferenced, what decommit invalidates,
  whether the owner is `Send` or `Sync`, and how concurrent commit/decommit is prevented
  or synchronized. Every unsafe system call or pointer operation needs a local `SAFETY`
  explanation tied to checked preconditions.

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
