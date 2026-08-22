# Iteration 00 — Public skeleton

## Goal

Create a clean, reproducible repository skeleton before implementing collector behavior.

## Assignment

- [ ] Pin stable Rust and one .NET 10 SDK/runtime version.
- [x] Create the Cargo workspace and placeholder crates described in `README.md`.
- [x] Create `samples/LoaderSmoke` as a minimal console application.
- [ ] Add formatting, lint, test, and managed-build commands to Windows CI.
- [ ] Add a license, contribution guide, security policy, and a short local setup guide.
- [x] Ensure build outputs and local diagnostics are ignored.

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
