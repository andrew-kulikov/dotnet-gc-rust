# Iteration 22 — Benchmarks and v0.2

## Goal

Decide honestly whether the frame-aware policy improves tail pauses enough to justify its memory and throughput costs.

## Assignment

- Implement the workloads and comparisons listed in roadmap milestone 9.
- Record pause percentiles, maxima, frame misses, throughput, total GC CPU, committed memory, reclamation rate, and fragmentation.
- Automate warmup, seeded trials, raw result capture, and report generation.
- Record hardware class, OS, power plan, runtime commit, trial count, and uncertainty without private machine identifiers.
- Explain wins, losses, outliers, and fallback frequency.
- Reproduce the report from a clean checkout and tag `v0.2.0` even if the hypothesis fails.

## Constraints

- Do not compare averages alone or omit failed/fallback runs.
- Stock and custom configurations must run equivalent workloads.
- Separate exploratory tuning data from final held-out measurements.

## Acceptance criteria

- Raw machine-readable results and exact reproduction commands are public.
- Reported tables and charts can be regenerated from committed data.
- Conclusions distinguish measured facts, hypotheses, and known confounders.
- README claims match the measured result and documented supported matrix.

## Hints

Automate environment capture and randomization before collecting final data. A negative but reproducible result is a successful research outcome.

## Agent review focus

Ask the agent for a reproducibility and methodology audit, including recalculation of headline metrics from raw data and a final public-repository privacy scan.
