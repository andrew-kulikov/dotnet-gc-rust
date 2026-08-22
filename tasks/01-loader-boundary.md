# Iteration 01 — Loader boundary

## Goal

Prove that the pinned CoreCLR reaches the custom collector boundary. Failure after reaching the boundary is the expected result.

## Assignment

- [x] Record the exact runtime source commit whose GC headers are used.
- [x] Build the thin C++ interface shim and Rust `cdylib` with a narrow C ABI between them.
- [x] Implement the standalone-GC version and initialization entry points.
- [x] Make initialization emit a deterministic diagnostic and return a deliberate failure.
- [x] Add a command that launches `samples/LoaderSmoke` with the custom GC selected.
- [ ] Write ADR-0001 describing why the project uses a C++ ABI shim and what remains owned by Rust.

## Constraints

- Do not implement an allocator or return fake interface objects.
- No Rust panic or C++ exception may cross an ABI boundary.
- Do not depend on globally installed source files without checking their version.

## Acceptance criteria

- Export inspection shows the required standalone-GC entry points.
- Running the sample proves that both the loader and initialization boundary were reached.
- The process fails predictably with the project diagnostic, not an access violation or missing-symbol error.
- CI builds the boundary against the pinned headers.

## Hints

Keep callbacks boring and log without allocating on a path that may later become allocation-sensitive. A loader trace is stronger evidence than “the DLL was found.”

## Agent review focus

Ask the agent to verify symbol names, calling conventions, version negotiation, error handling, and agreement between the pinned headers and built runtime.
