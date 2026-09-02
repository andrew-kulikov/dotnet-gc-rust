# Mission 00 - Reproduce the red baseline

## Where you are

The repository contains a Rust workspace and a minimal `net10.0` console
application. No collector behavior is required yet.

## The problem

Later failures will be impossible to interpret if different machines silently
select different Rust compilers, .NET SDKs, runtimes, or native build tools. You
also need a known-good stock-GC run and a known failing custom-GC run so future
missions can prove exactly what changed.

## Observe first

From a fresh shell, record the versions selected by the repository and run the
managed sample with the stock GC. Then inspect `git status` after the build to
see which outputs need to remain untracked.

Do not add GC code while doing this mission.

## Your challenge

- [x] Pin the Rust compiler through `rust-toolchain.toml` and the workspace
  `rust-version`.
- [x] Select one .NET 10 SDK through `global.json`. Separately record the exact
  .NET 10 runtime/CoreCLR build used by native smoke tests; `global.json` pins an
  SDK, not the runtime loaded into the sample process.
- [x] Keep the initial Cargo crates compiling as empty responsibility boundaries
  rather than speculative collector APIs.
- [x] Build and run `samples/LoaderSmoke` with the stock GC and record its stable
  success output.
- [x] Provide repository commands for formatting, Clippy, Rust tests, Miri on
  `gc-rust`, and the managed build. Run the repeatable subset in Windows CI.
- [x] Document only the local prerequisites and verification commands needed to
  build the project.
- [x] Ignore Cargo, CMake, MSBuild, managed, dump, and trace outputs so the
  documented checks leave the worktree clean.

## Checkpoint

From a clean checkout:

```powershell
rustc --version
dotnet --version
cargo test --workspace --locked
dotnet run --project samples/LoaderSmoke/LoaderSmoke.csproj
git status --short
```

The selected versions match the repository pins, the sample prints its expected
line under the stock GC, tests pass, and generated outputs do not appear in
`git status`.

## Allowed shortcuts

- Crates may remain nearly empty.
- CI may target only the supported Windows x64 configuration.
- The sample may contain a single `Console.WriteLine`.

## Known debt

There is no custom GC library yet. This mission proves the environment, not the
collector.

## What this unlocks

Mission 01 can attribute a startup failure to the custom loader boundary rather
than to an unpinned or broken toolchain.

## Hints

Print SDK and runtime inventories separately when diagnosing version selection.
Keep setup scripts free of usernames and absolute local paths.
