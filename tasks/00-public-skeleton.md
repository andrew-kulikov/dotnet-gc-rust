# Iteration 00 — Public skeleton

## Goal

Create a clean, reproducible repository skeleton before implementing collector behavior.

## Assignment

- [ ] **Pin the toolchain used by both halves of the project.** Keep the Rust compiler in
  `rust-toolchain.toml` and the workspace `rust-version` in agreement. Add a
  `global.json` that selects one .NET 10 SDK, and separately record the exact .NET 10
  runtime/CoreCLR build used by the standalone-GC smoke test. Document commands that
  print all three versions so a clean checkout can detect a mismatch before compiling.
- [x] **Create the Cargo workspace and its initial crate boundaries.** The placeholder
  crates from `README.md` should compile together, but should not contain speculative GC
  APIs. Each crate must already have a clear responsibility: model algorithms, platform
  memory, CoreCLR-specific behavior, or the exported FFI boundary.
- [x] **Create `samples/LoaderSmoke` as the smallest managed executable.** It should have
  no package dependencies or GC-specific behavior yet. Its only job in this iteration is
  to prove that the selected SDK can restore and build a `net10.0` application.
- [ ] **Put the repeatable checks in Windows CI.** Run Rust formatting, workspace tests,
  Clippy with warnings denied, and the managed sample build from a fresh checkout. Use
  the repository-pinned tool versions rather than whichever SDK happens to be newest on
  the runner.
- [ ] **Write a short local setup section.** List the required Rust toolchain, .NET SDK and
  runtime, Python, Visual Studio C++ workload, and Windows SDK, together with the exact
  commands used to verify them. Keep the section focused on building and running the
  project locally.
- [x] **Keep generated and machine-local files out of version control.** Ignore Cargo,
  CMake, MSBuild, and managed build outputs plus local dumps and traces. After running the
  documented build, `git status` should show no generated files.

## Constraints

- Do not vendor .NET binaries or CoreCLR source.
- Do not introduce GC logic, virtual-memory code, or unsafe placeholders.
- Repository scripts must not contain usernames or absolute machine paths.

## Acceptance criteria

- A clean checkout can build and test the Rust workspace and managed sample using documented commands.
- Formatting and lint checks run in CI.
- `git status` remains clean after the documented build, apart from ignored outputs.
- The README clearly states the platform, runtime version, educational scope, and non-goals.

## Hints

Prefer empty, compiling libraries over speculative APIs. Pin tools through repository files rather than prose alone. Check ignored files from the point of view of a future public fork.

## Agent review focus

Use `REVIEW_GUIDE.md`. Also ask the agent to search tracked files for absolute paths, generated binaries, credentials, and machine identifiers.
