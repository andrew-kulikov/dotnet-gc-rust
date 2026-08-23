# Mission 02 - Let initialization succeed

## Where you are

CoreCLR reaches `GC_Initialize`, crosses into Rust, and receives an intentional
failure. The shim has no `IGCHeap` or handle-manager objects.

## The problem

Returning `S_OK` without valid, stable interface objects would let CoreCLR call
garbage pointers. Implementing every interface method as if the final collector
already existed would be equally misleading. The useful next checkpoint is to
accept initialization, continue just far enough to observe the next runtime
request, and fail there by name.

## Observe first

From the pinned headers, inventory the pure virtual methods of `IGCHeap`,
`IGCHandleManager`, and `IGCHandleStore`. Group them as:

- required to construct and return the interface;
- observed during the current startup trace;
- not yet observed;
- explicitly outside the supported Windows x64 workstation configuration.

Do not infer that an unobserved method is optional forever.

## Your challenge

- [ ] Create C++ interface objects with process-long stable addresses and exact
  signatures from the pinned headers.
- [ ] Store the supplied `IGCToCLR` pointer in the adapter without exposing its
  C++ layout to Rust.
- [ ] Move any mutable collector state behind an opaque Rust-owned C ABI handle;
  do not duplicate heap policy or object state in C++.
- [ ] Make each unsupported method terminate through one diagnostic path that
  includes the method name. A stub must not return a plausible success value.
- [ ] Return the stable `IGCHeap` and `IGCHandleManager` pointers from
  `GC_Initialize` and return `S_OK`.
- [ ] Add ABI-focused tests for representative calls from C++ to Rust and Rust
  callbacks routed through C++ to the supplied `IGCToCLR` object.
- [ ] Reject unsupported architecture and Server GC configuration with a useful
  startup message.

## Checkpoint

Launch `LoaderSmoke` with the custom GC. The old message
`GC initialization failed` must disappear. The process should then stop at a
deterministic diagnostic naming the first operation that the shell does not yet
support.

Record that method and the call sequence; they are input to mission 03.

## Allowed shortcuts

- Most interface methods may fail fast.
- No managed allocation needs to succeed.
- Interface objects may live until process exit.

## Known debt

The shell proves ABI shape and lifetime only. It is not a collector, and its
method inventory is not yet a supported-feature matrix.

## What this unlocks

Mission 03 can implement only the operations demonstrated by the startup trace
and aim for the first managed `Main` execution.

## Hints

Use compile-time version assertions and let the C++ compiler verify overrides.
Keep failure reporting native and safe on allocation-sensitive paths.
