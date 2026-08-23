# Mission 01 - Cross the loader boundary

## Where you are

The stock sample and repository toolchain have a reproducible checkpoint. The
custom collector does not yet participate in startup.

## The problem

CoreCLR loads a standalone collector through exported C functions, then passes
C++ virtual interfaces across the GC/EE boundary. Rust cannot safely guess the
MSVC C++ vtable ABI. Before implementing a collector, prove that the pinned
runtime can load a thin native shim and that the shim can call Rust.

## Observe first

Read the pinned declarations of `GC_VersionInfo` and `GC_Initialize`. Run the
sample once with an invalid `DOTNET_GCPath` and record how a missing DLL differs
from a library that loads and deliberately refuses initialization.

## Your challenge

- [x] Pin the exact `dotnet/runtime` source commit that supplies the GC headers
  and make bootstrap reject a missing or locally modified checkout.
- [x] Build a C++ DLL against those headers and a Rust `cdylib` exposing one
  narrow `extern "C"` probe.
- [x] Export `GC_VersionInfo` and `GC_Initialize` with the exact pinned calling
  convention and interface version.
- [x] Have `GC_Initialize` call Rust, initialize every output to a safe empty
  value, emit one deterministic native diagnostic, and return a deliberate
  failure `HRESULT`.
- [x] Add one smoke command that builds both DLLs, verifies exports, launches the
  real managed executable, and recognizes the expected failure.
- [x] Record the boundary decision in a short engineering note: C++ owns only the
  virtual-interface adapter; Rust will own collector state and policy; neither
  panic nor exception may cross the boundary.

## Checkpoint

```powershell
python scripts/build.py smoke
```

The process reports the project diagnostic proving `CoreCLR -> C++ -> Rust`,
then exits with the expected initialization failure. Missing-library,
missing-export, access-violation, and version-mismatch failures do not count.

## Allowed shortcuts

- Initialization must fail.
- No `IGCHeap` or handle implementation is needed.
- Native logging may be minimal and allocation-free.

## Known debt

No interface pointer can be returned to CoreCLR, so managed startup cannot
continue.

## What this unlocks

Mission 02 can return real interface objects and discover the next required
operation from an actual runtime call rather than implementing an imagined
startup sequence.

## Hints

Inspect the final DLL exports instead of trusting source spelling. Keep the
runtime header commit and installed runtime release visibly paired.
