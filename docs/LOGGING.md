# Rust logging migration

This bounded maintenance change replaces the custom logger without advancing
Mission 09 or changing collector semantics.

## Changes and source facts

- `tracing` replaces all ordinary `log_*` call sites; `util.rs` is removed.
  `logging.rs` installs one `tracing-subscriber` formatter during the loader
  probe. The cached installation result makes repeated probes idempotent.
- `RUST_GC_LOG_LEVEL` still selects verbosity, case-insensitively, with `info`
  for missing/invalid values. Errors go to stderr; other levels go to stdout.
  Prefixes are now uppercase (`INFO`, `ERROR`) without brackets or ANSI
  colors. UTC timestamps and selected function timings were enabled in the
  2026-10-28 follow-up below. The environment is read at loader initialization rather than
  on the first log call. Cargo.lock pins the new dependencies.
- Enabled spans emit `new` and `close` events. Function instrumentation remains
  explicit with `#[tracing::instrument(...)]`; existing functions were not
  instrumented wholesale. No logging was added inside the stopped operation.
- Subscriber installation failure returns `E_FAIL` to the C++ loader. The shim
  now propagates that failure before publishing the heap and handle manager.
- The terminal allocation report deliberately writes directly to stderr.
  In tracing-subscriber 0.3.23, `fmt_layer.rs::on_event` uses `BUF.with(...)`
  on a destructible thread-local `String`. CoreCLR can call `Shutdown` after
  that buffer has been destroyed. The direct report preserves shutdown,
  unsupported-operation, and allocation-exhaustion diagnostics at that point.
- The fixture runner now rejects Rust panic text even when a process exits 0.

## Observed verification

Baseline: `cargo test --workspace` passed 42 unit tests and six doctests; the
intentional UB demonstration remained ignored. The existing staged native shim
also passed the chain handoff/repeated-suspension fixture before replacement;
its raw output is under `out/tracing/baseline.{stdout,stderr}.log`.

An initial native run with the formatted terminal report produced
`cannot access a Thread Local Storage value during or after destruction`.
The process could still exit 0 after all managed continuation markers. This
runtime observation motivated the direct terminal report and panic assertion;
it was not detected by the Rust tests. The failing trace is retained under
`out/tracing/teardown-failure.{stdout,stderr}.log`.

Final checks on 2026-10-27:

| Command / checkpoint | Observed result |
| --- | --- |
| `python scripts/build.py verify` | Exit 0: formatting, Clippy with warnings denied, 42 unit tests, six doctests, managed build, stock-GC smoke |
| `cargo build --workspace --locked --offline` | Exit 0; cached dependencies suffice |
| `cmake --build out/build/native-shim/debug --config Debug` | Exit 0 with the existing Windows x64 CMake configuration |
| `python out/tracing/check_runtime.py` | Exit 0: all six ZeroGC handoff/repeated-suspension fixtures, then chain again with `TrAcE`; final reports present, no Rust panics |
| `python out/tracing/check_ffi.py` | Exit 0: actual DLL exports preserve filtering and stream routing for missing/invalid settings and all named levels; repeated loader probes succeed |
| Temporary Rust probe using production `logging.rs` | All five levels, mixed case, missing/empty/invalid values, lazy argument evaluation, function parameters/result, span lifecycle, repeated initialization, and subscriber conflict passed |
| Fixture runner with an injected exit-0 panic result | Panic rejected with `RuntimeError` |
| `git diff --check --ignore-submodules` | Passed |

Temporary probe sources, binaries, and machine-specific output stay under
`out/tracing/`; managed fixture output stays under `out/graph-fixtures/`.
The local Rust compiler reported incremental-cache hard-link warnings and copied
files instead. The C++ build reported existing unused-parameter warnings.

## Limits and next action

The usual `python scripts/graph_fixtures.py --fixture chain --suspend-repeat`
entry point stopped at its Git submodule identity check: access to the
submodule's Git metadata is denied in this environment. Runtime verification
therefore used the already configured CMake build and existing `run_fixture`
checkpoint assertions directly. This does not establish a fresh submodule
identity check or a fresh end-to-end build through the normal runner.

The shell's Python alias was also inaccessible; the installed interpreter was
invoked directly. These are local execution limits, not repository changes.

Logging is not safe inside the suspended operation or after thread-local
teardown; only the explicit terminal report supports that late stage. No
per-module filter syntax or asynchronous output was added.
Next action: run the normal fixture entry point where submodule metadata is
accessible; the learning sequence still proceeds to Mission 09.

## 2026-10-28: shutdown details and selected function timings

The user enabled timestamps by removing `without_time()`; that change is
preserved. This first added `Info` spans to `rust_gc_initialize`,
`rust_gc_handle_manager_initialize`, `rust_gc_suspend_ee`, and
`rust_gc_suspend_with_handle`. They replace the corresponding manual entry
messages and include return values. The handoff span records the opaque handle
address and failure-injection flag. `DiagnosticResult` derives `Debug` to expose
its copied status/restart count; its C ABI layout remains unchanged.

The outer suspension spans enter before suspension and exit after restart.
No subscriber callbacks were added to the stopped operation or to the restart
guard. At that checkpoint, allocations, handle updates, count queries, loader
setup, and late shutdown/reporting had not received spans. `close` now contains `time.busy` and
`time.idle`, measured through `Instant`. For a synchronous function, busy time
includes blocking while the span is entered. It is not CPU time or an exact
stop-the-world duration, and logging itself affects short measurements.

The pinned CoreCLR source establishes the late call path in
[`ceemain.cpp`](../external/dotnet-runtime/src/coreclr/vm/ceemain.cpp):
`EEDllMain(DLL_PROCESS_DETACH)` calls `EEShutDown(TRUE)`, whose helper calls
`IGCHeap::Shutdown` in phase 2 under the loader lock. The shim forwards this to
`rust_gc_shutdown` and the final report. Rust 1.97.1's
`sys/thread_local/guard/windows.rs` separately registers TLS destructor callbacks
for thread/process detach. `native/eager.rs` sets destructible TLS storage to
`Destroyed` before dropping its value. Subsequent `LocalKey::with` access
panics; `try_with` returns `AccessError`. This concerns native Rust thread-local
objects, not managed `ThreadStatic` data, and does not imply that the OS has
already unmapped the whole DLL or heap.

The formatter's `BUF.with(...)` is shared by enabled `Info` and `Error` events.
Changing severity, timestamps, the output writer, or only `FormatEvent` does
not remove that access. A nonblocking writer also receives data after formatting,
so it does not fix this TLS dependency. A library change using `try_with` with
a temporary buffer, or a different subscriber without this cache, could address
the immediate panic; neither makes arbitrary loader-lock work safe.

The practical alternative is to emit richer reports explicitly before process
teardown and keep the late callback minimal. The present direct stderr report
avoids the formatter TLS, but still takes ledger/stderr locks and sorts ranges.
It is not a proof against locks orphaned by abrupt thread termination, blocked
I/O, allocator shutdown, or calls into already torn-down libraries. Worker joins,
queue drains, managed callbacks, and DLL loading must not be introduced here.

Verification on 2026-10-28:

- `python scripts/build.py verify` passed formatting, Clippy, 42 unit tests,
  six doctests, and the stock-GC smoke.
- `cargo build --workspace --locked --offline` and the existing CMake build
  passed. `python out/tracing/check_runtime.py` passed all six repeated-cycle
  fixtures at the default level plus chain at `TrAcE`, with final reports and
  no Rust panics. The empty `--suspend` chain fixture also passed.
- Output assertions verified one initialization span per entry, nine handoff
  spans, the injected `OperationFailed` result, and `time.busy`/`time.idle` on
  every close. Machine output is under `out/tls-shutdown/` and
  `out/graph-fixtures/`.
- A small standalone Rust destructor-order probe initialized a logging buffer
  after another TLS object, then logged from that object's later destructor.
  Enabled `info!` and `error!` both reproduced the destroyed-TLS panic and
  aborted. `info!` filtered out at maximum `error` completed normally because
  it never reached the formatter. This isolates the mechanism without relying
  on CoreCLR or changing the production shutdown path.

The normal build runner's submodule-metadata access limit remains unchanged.

## Remaining function-entry messages

All remaining manual `tracing::info!` entry messages now use spans. Count
queries and allocation-context validation use `#[tracing::instrument]` at the
existing `Info` level, recording scalar/pointer arguments and count results.
The collection-count parameter names were corrected for readable fields;
the C ABI is unchanged.

The loader probe creates an explicit `info_span!` after subscriber installation.
A function attribute would run before installation and miss the first call.
Its duration therefore covers the probe after setup, not logger initialization.
No manual `tracing::info!` calls remain in the crate.

`python scripts/build.py verify` passed formatting, Clippy, 42 tests, six
doctests, and the stock-GC smoke. The Rust library and configured CMake shim
rebuilt successfully. `python out/tracing/check_ffi.py` verified two loader
span pairs on repeated probes and existing level/stream behavior.
`python out/tracing/check_runtime.py` passed all six repeated-cycle fixtures
plus chain at `TrAcE`. Output assertions confirmed matching `new`/timed `close`
pairs for all converted entries, collection-count fields, and the context
address. No shutdown panic occurred. The existing submodule-metadata limitation
for the normal native build runner still applies.

## 2026-10-28: nested console scopes

This bounded logging step adds `tracing-tree` 0.4.1 without advancing the
learning missions or changing the stopped operation.

Changes and source facts:

- stdout uses `HierarchicalLayer` with two spaces per level, tree lines,
  named exits, and span retracing when concurrent output switches context.
  Existing `instrument` attributes and event calls are unchanged. ANSI remains
  disabled, and `RUST_GC_LOG_LEVEL` keeps its previous defaults and parsing.
- The tree calls `MakeWriter::make_writer()` rather than `make_writer_for()`.
  Therefore the old level-aware writer cannot route its errors. A separate
  standard fmt layer handles only error events in stderr, with span context;
  the tree handles other events and all enabled spans. Both filters accept
  non-event metadata, including `enabled!` hints, to preserve nesting.
- A small timer adapter reuses tracing-subscriber's UTC `SystemTime` formatter
  through its public `Writer`. It prints monotonic elapsed wall time since span
  creation beside events, including `return`, without a date/time dependency.
  Unlike the previous fmt output, boundary lines contain names/arguments only;
  there are no `time.busy`/`time.idle` fields. Return-event elapsed time includes
  time outside the entered span and formatting overhead, and is not a GC pause
  or CPU-time measurement. A span without events has no printed duration.
- The terminal report still uses direct stderr output. The error formatter still
  has the destructible TLS buffer; changing the stdout layer does not establish
  late-shutdown safety. No new logging was added inside suspension.
- The first managed probe exposed Python's Windows locale decoding of the tree's
  UTF-8 bytes (`UnicodeDecodeError`). The shared runner now explicitly decodes
  captured output as UTF-8 and reprints captured streams as UTF-8, including when
  redirected. This keeps the ordinary fixture assertions and saved logs usable
  without requiring `PYTHONUTF8` or a console code-page change.

Verification:

- Baseline `cargo test --workspace`: 42 unit tests and six doctests passed;
  the intentional Miri UB demonstration remained ignored.
- `python scripts/build.py verify`: formatting, Clippy with warnings denied,
  42 unit tests, six doctests, managed build, and stock-GC smoke passed.
- `cargo build --workspace --locked --offline` and
  `cmake --build out/build/native-shim/debug --config Debug` passed using the
  existing CMake configuration.
- `python out/tracing-tree/check_logging.py`: separate processes passed for
  unset, empty, invalid, error, warn, info, debug, and mixed-case trace settings.
  Assertions cover nested openings/exits, parameters/results, UTC timestamps,
  elapsed durations, stderr context, stream routing, disabled-argument evaluation,
  repeated initialization, cached subscriber conflict, and actual DLL exports.
- `python out/tracing/check_runtime.py`: all six repeated-cycle fixtures plus
  chain at trace passed with final reports and no Rust panics. This also exercises
  the UTF-8 runner without enabling Python's UTF-8 mode.

Machine-specific probes and logs are under `out/tracing-tree/` and
`out/graph-fixtures/`. The network registry index was unavailable; the offline
build used cached dependencies. The native build still cannot check the runtime
submodule identity: `git -C external/dotnet-runtime rev-parse HEAD` reports
permission denied for its Git metadata. Native verification therefore used the
existing configured build, not a fresh normal-runner checkout/build.

Next action: review the tree during diagnostic runs; resume Mission 09 separately.

## 2026-10-28: simplify scope output

The tree format left `rust_gc_fix_alloc_context` with untimed boundary lines
and put collection-count timings on a separate return line. This follow-up
removes `tracing-tree`, its timer adapter, and the two filtered output layers.

One standard fmt subscriber now handles timestamps, span fields, `new`/`close`,
return events, duration measurement, filtering, and stdout/stderr routing.
`Indented::format_event` only adds two leading spaces per nested span before
delegating to the standard formatter. It adds no lifecycle state or timers.
Existing `ret` attributes are preserved; they still produce a separate result
line, now using the same format as the boundaries. Spans without `ret` get a
timestamp and `time.busy`/`time.idle` on `close` too.

The dependency and its otherwise-unused transitive packages are removed from
Cargo.toml/Cargo.lock. The terminal report and the no-logging suspension boundary
are unchanged. The UTF-8 runner correction remains useful for duration units and
other Rust text. The earlier tree section records the superseded experiment.

Verification:

- Baseline `cargo test --workspace --locked`: 42 tests and six doctests passed.
- `python scripts/build.py verify`: formatting, Clippy with warnings denied,
  42 tests, six doctests, managed build, and stock-GC smoke passed.
- `cargo build --workspace --locked --offline` and the configured
  `cmake --build out/build/native-shim/debug --config Debug` passed.
- `python out/logging-simple/check_logging.py` passed separate-process checks
  for all levels and missing/empty/invalid settings, nested indentation, UTC
  timestamps on every line, timed closure of a span without events/ret, return
  values when requested, stream routing, DLL exports, disabled arguments,
  repeated initialization, and cached subscriber conflict.
- `python out/tracing/check_runtime.py` passed all six repeated-suspension
  fixtures plus chain at trace, with final reports and no Rust panic. Runtime
  `rust_gc_fix_alloc_context` emits both timestamped entry and timed closure.

Temporary probes and machine output are under `out/logging-simple/` and
`out/graph-fixtures/`. Native verification still uses the existing CMake
configuration because the runtime submodule's Git metadata is inaccessible.
No mission semantics changed; next action remains Mission 09 separately.
