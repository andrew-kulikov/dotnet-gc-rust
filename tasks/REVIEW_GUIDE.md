# Agent review guide

Use this prompt after completing an iteration:

> Review my implementation of `tasks/NN-task-name.md`. Do not change files. Read the task, relevant design documentation, and the complete diff for this iteration. Run the checks needed to verify every acceptance criterion. Look especially for memory-safety assumptions, arithmetic overflow, ABI mismatches, missing negative tests, and undocumented scope changes. Report findings by severity with file and line references, then give a criterion-by-criterion PASS/FAIL table. If verification is blocked, say exactly what evidence is missing. Finish with the smallest next action required for the iteration to pass.

## Review rules

The reviewing agent should:

1. Treat the iteration file as the specification and `docs/DESIGN.md` as supporting context.
2. Inspect the implementation and tests; a green test command alone is insufficient.
3. Run only commands relevant to the iteration and report the exact commands used.
4. Distinguish correctness blockers, important improvements, and optional polish.
5. Reject accidental reliance on machine-specific paths, untracked SDK binaries, undefined ABI assumptions, or panic unwinding across FFI.
6. For `unsafe` code, require a local `SAFETY` explanation and verify that its preconditions are enforced by the safe API.
7. End with one verdict: `PASS`, `PASS WITH FOLLOW-UP`, or `NOT YET`.

The agent must not broaden the task or require stretch work for a pass. A learner can explicitly ask for implementation help after receiving the review.
