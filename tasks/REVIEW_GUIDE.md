# Mission review guide

Use this prompt after reaching a mission checkpoint:

> Review my implementation of `tasks/NN-mission.md`. Do not change files. Read
> the mission, the previous mission's checkpoint, relevant design notes, and the
> complete diff. First reproduce the problem under **Observe first**, then run
> the checkpoint. Report correctness and safety findings by severity with file
> and line references. Check that the implementation solves the current problem
> without silently implementing unsupported behavior. Do not require future
> mission architecture or reject shortcuts explicitly allowed by this mission.
> Finish with the smallest next action needed for the checkpoint to pass.

## Review rules

1. Treat the mission's observable checkpoint as the specification. Proposed
   architecture in `docs/DESIGN.md` is context, not a requirement.
2. Verify the starting failure or limitation before accepting its fix.
3. Inspect code and tests; a green command alone is insufficient.
4. Accept documented temporary implementations listed under **Allowed
   shortcuts**.
5. Flag premature abstractions when they add unsafe code or lock in a future
   design without helping the current checkpoint.
6. Reject silent-success stubs, machine-specific paths, ABI guesses, unchecked
   arithmetic at raw-memory boundaries, and unwinding across FFI.
7. For `unsafe`, require a local `SAFETY` explanation and evidence that callers
   enforce its preconditions.
8. Compare the observed result with **Known debt**. Debt is acceptable only when
   it fails predictably, is bounded where necessary, and is not claimed as
   supported.
9. Record whether the mission confirmed or invalidated its starting assumptions.

End with one verdict: `PASS`, `PASS WITH DOCUMENTED DEBT`, or `NOT YET`.
