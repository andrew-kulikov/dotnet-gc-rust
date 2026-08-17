# Managed samples

This directory will contain small C# programs that demonstrate one behavior at a time. They are executable examples, not a second test framework.

Planned samples:

- `LoaderSmoke` — starts with the custom GC and performs a small allocation.
- `ObjectLayouts` — classes, strings, arrays, inheritance, and value types containing references.
- `GcFeatures` — handles, weak references, pinning, finalizers, and resurrection.
- `FrameWorkload` — deterministic 60 Hz and 120 Hz allocation workloads.

Keep samples deterministic and command-line driven. Each program should print a concise success/failure result, accept a random seed where randomness is useful, and run under both the stock GC and the custom GC. Do not commit runtime binaries, dumps, traces, hostnames, usernames, or machine-specific paths.
