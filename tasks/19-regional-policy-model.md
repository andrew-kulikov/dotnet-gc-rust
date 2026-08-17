# Iteration 19 — Regional policy model

## Goal

Select a bounded set of victim regions in the pure-Rust model and prove partial-collection liveness.

## Assignment

- Add per-region live-byte estimates, age, free-space quality, and dirty-reference cost inputs.
- Define `Throughput`, `Interactive`, and `FrameBudget` policy modes.
- Select victim regions under an estimated work budget.
- Mark and sweep only victims while preserving objects reachable from roots or non-victim regions.
- Fall back to a full collection under memory pressure or unproductive partial work.
- Compare estimates with measured model work on generated graphs.

## Constraints

- A budget is a selection target, not a real-time guarantee.
- Non-victim objects cannot be reclaimed or mutated by partial sweep.
- The correctness oracle must conservatively include cross-region edges.

## Acceptance criteria

- Partial and full collectors agree on liveness inside selected victims.
- Tighter budgets select no more estimated work than looser budgets for the same state.
- Pathological cross-region graphs cause conservative work or explicit fallback, never unsound reclamation.
- Policy decisions and fallback reasons are observable in metrics.

## Hints

Keep selection separate from tracing. Begin with exact model costs, then introduce estimates so estimation error can be measured.

## Agent review focus

Ask the agent to generate adversarial cross-region graphs and prove why every edge that can enter a victim is represented in the liveness calculation.
