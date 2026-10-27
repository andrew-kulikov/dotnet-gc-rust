# 5. Dependent handles for ordinary formatting

[Mission overview](README.md) · [Previous: the checkpoint](04-handoff-checkpoint.md)

Ordinary numeric interpolation now works under ZeroGC. The fixture no longer
converts cycle numbers to strings before interpolation. This follow-up was
explicitly requested to remove the formatting workarounds encountered in Mission 08.

## Reproduced problem

After changing the fixture to `$"Worker made progress before cycle {cycle}."`,
`python scripts/graph_fixtures.py --fixture chain --suspend-repeat` exited 1.
Stock GC passed; ZeroGC aborted at the named `CreateDependentHandle` callback.
The runtime's interpolated-string handler uses ArrayPool, whose thread-local
tracking uses ConditionalWeakTable. The initial failing streams are preserved
under `out/dependent-handles/baseline.{stdout,stderr}.log`.

## Pinned contract and implementation

[`comdependenthandle.cpp`](../../external/dotnet-runtime/src/coreclr/vm/comdependenthandle.cpp)
defines the required EE calls. [`gcinterface.h`](../../external/dotnet-runtime/src/coreclr/gc/gcinterface.h)
assigns dependent handles kind 6; the bridge asserts that value. The existing
stable native record now stores a secondary pointer as well as its first-field
primary slot. All handle records still use the same allocator and layout.

- `CreateDependentHandle` initializes both targets and returns null on native
  allocation failure, allowing the EE's existing slow path/OOM handling.
- `GetDependentHandleSecondary` reads the secondary atomically with acquire
  ordering; `SetDependentHandleSecondary` publishes replacements with release
  ordering. These operations borrow only the atomic field, so concurrent primary
  clearing does not overlap a Rust reference to the whole record.
- The EE reads the primary first and hides the secondary when that primary is
  null. The GC-side getter must return the stored secondary without rechecking
  the primary: a concurrent clear must not invalidate an EE reader's earlier
  non-null observation. The source implements `TargetAndDependent` this way.
- `DependentHandle.Dispose` uses `DestroyHandleOfType(..., HNDTYPE_DEPENDENT)`.
  That path now frees the native record after all readers have stopped. Disposal
  must not race with other operations, as required by the managed API. Other
  typed-destruction kinds retain their named unsupported failure.

There is no collection-time barrier or dependent marking pass: ZeroGC does not
collect or move objects. This change implements handle storage and API operations
for the current allocator; tracing and weak clearing still belong to the later
collection missions. Normal-handle destruction keeps its existing behavior.

## Checks and results

The new bounded `LoaderSmoke dependent` scenario checks `Target`, `Dependent`,
and `TargetAndDependent`; secondary replacement and nulling; explicit primary
clearing; initial null targets and aliased targets; disposal and repeated disposal.
It runs under both stock GC and ZeroGC through the existing scenario matrix.
Native tests separately check the first-field primary slot, stored secondary
updates, nulls, kind 6, and destruction. Miri checks the allocation/access/free path.

| Command | Observed result |
| --- | --- |
| `python scripts/build.py verify` | Exit 0: formatting, Clippy, 42 Rust unit tests, six doctests, stock smoke |
| `cargo +nightly-2026-08-17 miri test --locked -p gc-rust --lib handle_store::tests` | Four tests passed |
| `python scripts/build.py matrix --report-dir out/dependent-handles/matrix` | Exit 0, including the new dependent-handle case under both collectors; existing unsupported/exhaustion outcomes preserved |
| `python scripts/graph_fixtures.py` | Exit 0: all six baseline fixtures under both collectors |
| `python scripts/graph_fixtures.py --suspend-repeat` | Exit 0 for all six fixtures, using ordinary numeric interpolation; single restart, worker progress, injected failure cleanup, and continuation checks still pass |

The Windows workflow also runs the scenario matrix and saves its logs, so the
explicit dependent-handle API checks run alongside the interpolation regression.

Next action: Mission 09 can use the verified observation window. No object
decoding or collection behavior was added by this formatting fix.
