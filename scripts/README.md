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
```

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
