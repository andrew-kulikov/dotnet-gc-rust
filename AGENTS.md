# Working rules

- Follow the current mission in `tasks/` and its plan in `docs/`. Complete one
  bounded plan step at a time; do not implement later missions early.
- Prefer clear, idiomatic code and the smallest correct change. Add abstractions,
  dependencies, configuration, and fallback paths only for a demonstrated need.
- When tests need to replace a few calls, prefer ordinary callbacks over custom
  traits and real/fake adapter types added solely for testing.
- Reproduce the relevant baseline first. Run checks appropriate to the change;
  avoid repeating successful checks without a new reason.
- Document each completed step briefly: what changed, the command and observed
  result, remaining limitations, and the next action. Distinguish source facts,
  proposals, and runtime evidence. Keep machine-specific logs under `out/`.
- Preserve explicit unsupported behavior. Do not replace missing runtime
  semantics with silent-success stubs. Explain `unsafe` preconditions locally
  and prevent unwinding across FFI.
- Keep command runners direct: small named functions, visible process output,
  exit-code checks, and necessary timeouts. Parse output only for checkpoint
  assertions; avoid extra telemetry, summaries, and output-redirection frameworks.
