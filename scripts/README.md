# Repository scripts

Run from the repository root. All scripts use only the Python standard library.

```console
python scripts/build.py verify
python scripts/build.py build --configuration release
python scripts/build.py smoke
python scripts/build.py matrix --report-dir out/matrix
python scripts/build.py miri
python scripts/graph_fixtures.py
python scripts/graph_fixtures.py --stock-only
python scripts/graph_fixtures.py --fixture chain
python scripts/graph_fixtures.py --fixture chain --suspend
python scripts/graph_fixtures.py --suspend-repeat
python scripts/graph_fixtures.py --fixture chain --inspect
```

`--fixture` selects one named graph from the fixture file in both the Rust
model and managed runs. `--scenario` selects the file stem (default `scenarios`).
`--suspend` adds one empty diagnostic cycle in ZeroGC; stock GC still checks the
declared graph. Omit `--fixture` to run every fixture. Suspension logs have a
`.zerogc.suspend` suffix under `out/graph-fixtures/`. See the
[first-cycle notes](../docs/08-suspension-lifecycle/02-empty-cycle.md) for scope
and individual Rust test commands.

`--suspend-repeat` checks one normal-handle handoff with one mutator, then eight
cycles with a second bounded mutator. Cycle 4 injects a post-suspend operation
error; all cycles must restart once and show worker progress afterward. The managed sample checks worker progress and exit, restart, and handle release;
it also prints the graph and continuation.
It is mutually exclusive with `--suspend` and cannot be combined with
`--stock-only`. Logs use `.zerogc.suspend-repeat`; see the
[completed Mission 08 checkpoint](../docs/08-suspension-lifecycle/04-handoff-checkpoint.md).

`--inspect` runs the Mission 09 object reader on a separate pair of fixture
nodes. Eight stopped cycles alternate `B` and null; cycle 4 intentionally uses
the wrong expectation. The managed sample checks the actual slot, restart count, worker
progress, and managed handle release, then prints its continuation. Logs use the `.zerogc.inspect` suffix. See
the [inspection checkpoint](../docs/09-real-object-inspection/README.md).

The graph runner uses plain headings and always shows process stdout/stderr.
There is no quiet/verbose mode or computed run/cycle summary. `run_process()`
prints the output and checks the exit code (stdout first, then stderr). Raw model and managed output is also saved as separate
`*.stdout.log` / `*.stderr.log` files under `out/graph-fixtures/`.

The Rust model and managed sample check their own expected reachable sets. The
managed diagnostic modes also check their returned status and observations.
The runner does not parse process output or independently compare every printed
edge, root, or GC initialization message. Managed runs retain a 30-second
timeout; the model command may build Rust code, so it has no process timeout.

Both entry points accept `--log-level error|warn|info|debug|trace` to set Rust
collector verbosity for child processes. For `build.py`, put this top-level
option before the operation:

```console
python scripts/build.py --log-level trace smoke
python scripts/build.py --log-level debug matrix --report-dir out/matrix
python scripts/graph_fixtures.py --log-level trace
```

Levels are case-insensitive. An explicit option overrides `RUST_GC_LOG_LEVEL`;
omitting it preserves the inherited value, with the Rust logger defaulting to
`info` when the variable is missing or invalid. This option controls Rust logs;
Python headings, command lines, and process output remain visible.

The runner colors its own errors and replayed stderr red, and command lines
green, when the corresponding output stream is a terminal. Redirected output
stays plain, and saved scenario reports retain the original child-process text.

`build.py` handles CLI arguments and dispatches to the build, verify, Miri,
smoke, and matrix modules in `gc_tools/`. `graph_fixtures.py` runs the graph
comparison and handles its own CLI. Validation specific to a command stays
with that command.

Shared helpers in `gc_tools/`:

- `utils.py`: process execution, reports, sample builds, and GC environments.
- `constants.py`: repository paths, toolchain pins, limits, and diagnostic strings.
- `toolchain.py`: pinned CoreCLR source checks and Windows tool discovery.
- `fixtures.py`: bounded graph parsing shared by tasks and labs.

For a new task or lab, add a command module and reuse these helpers as needed.
`build_sample()` returns a Debug assembly path; use `dotnet` to run it, or
`.with_suffix(".exe")` for the Windows app host. `run_scenario()` captures both
streams with a 30-second timeout and can save them using a report prefix.

Verify changes by running `verify`, `smoke`, `matrix`, and the graph fixtures.
