# Iteration 01 — Loader boundary

## Goal

Prove that the pinned CoreCLR reaches the custom collector boundary. Failure after reaching the boundary is the expected result.

## Assignment

- [x] **Record the runtime contract being compiled against.** Pin the exact
  `dotnet/runtime` commit, make the required GC headers available from that checkout, and
  fail the build if the checkout is missing or at a different revision. The installed
  CoreCLR used by the smoke test must belong to the same supported runtime release.
- [x] **Build a thin C++ shim and a Rust `cdylib`.** The C++ DLL may include CoreCLR's C++
  interface headers, while Rust exposes only a small `extern "C"` API made from
  fixed-layout values, pointers, and integer result codes. Prove the boundary in both the
  compiler/linker setup and one actual C++-to-Rust call.
- [x] **Export the two standalone-GC loader entry points.** Implement
  `GC_VersionInfo` with the pinned interface version and `GC_Initialize` with the exact
  calling convention and parameter types from the pinned header. Export inspection must
  find their undecorated names in the final shim DLL.
- [x] **Stop deliberately after crossing the boundary.** Have `GC_Initialize` call the
  Rust probe, emit one deterministic native diagnostic, set every output parameter to a
  safe empty value, and return a failure `HRESULT`. At this stage, failing initialization
  is correct; returning fake interface pointers is not.
- [x] **Add one command that exercises the real CoreCLR loader.** Build both native
  libraries, place dependent DLLs together, set the standalone-GC configuration for only
  the child process, and launch `LoaderSmoke`. The command should distinguish the expected
  project diagnostic from a missing DLL, missing export, version mismatch, or crash.
- [ ] **Write ADR-0001 for the ABI ownership decision.** Explain why CoreCLR's C++ virtual
  interfaces stay in C++, why Rust receives a narrow C ABI, which side owns collector
  state, and how panics/exceptions are contained. Record direct Rust implementation of
  MSVC vtables as the rejected alternative.

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
