# Iteration 21 — Frame-safe-point integration

## Goal

Run regional collections at an explicit point between frames and measure where the pause time is spent.

## Assignment

- [ ] Add an experimental managed control API that requests a collection with a microsecond work budget.
- [ ] Extend `samples/FrameWorkload` with deterministic 60 Hz and 120 Hz loops and seeded allocation patterns.
- [ ] Integrate regional selection, remembered-set scanning, victim tracing, and sweeping.
- [ ] Measure suspension/root scanning separately from regional work.
- [ ] Add emergency full-collection watermarks and observable fallback reasons.
- [ ] Ensure ordinary allocation pressure can still request a safe full collection.

## Constraints

- The API must describe the budget as a target, never a deadline guarantee.
- A safe-point request cannot bypass runtime suspension and root correctness.
- Optimization must not disable the complete baseline path.

## Acceptance criteria

- Smaller budgets generally select less regional work while fixed suspension cost remains visible.
- Partial collections preserve all objects in cross-region and runtime-feature stress samples.
- Emergency pressure reliably triggers a full collection instead of risking corruption or premature failure.
- The sample reports frame misses and phase timings in machine-readable form.

## Hints

Keep the controller open-loop at first: requested budget in, measured result out. Adaptive tuning can wait until the measurements are trustworthy.

## Agent review focus

Ask the agent to verify fallback safety, API claims, phase accounting, deterministic workload replay, and equivalence with full collection on the same seeded trace.
