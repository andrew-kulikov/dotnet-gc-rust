#!/usr/bin/env python3
"""Check shared graph fixtures with the Rust model, stock GC, and Windows ZeroGC."""

import argparse
import os
import subprocess

from build import (
    INTERFACE_SHELL_DIAGNOSTIC,
    LOADER_DIAGNOSTIC,
    REPOSITORY_ROOT,
    build,
    clean_stock_gc_environment,
    log,
    run,
)


def check(stock_only: bool) -> None:
    fixture = REPOSITORY_ROOT / "samples/GraphFixtures/scenarios.txt"
    scenarios = [
        line.split()
        for line in fixture.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    ]
    # Bound this teaching workload before constructing any managed objects.
    if not scenarios or len(scenarios) > 16:
        raise RuntimeError("expected 1..16 scenarios")
    names = set()
    for fields in scenarios:
        if len(fields) != 4:
            raise RuntimeError("expected name, edges, roots, and live labels")
        name, edges, roots, live = fields
        links = [link.split(":") for link in edges.split(",")]
        if (name in names or not 1 <= len(links) <= 16
                or any(len(link) != 2 for link in links)):
            raise RuntimeError("duplicate scenario or invalid bounded links")
        names.add(name)
        labels = [source for source, _ in links]
        if (len(set(labels)) != len(labels)
                or any(not label.isascii() or not label.isalnum() for label in labels)):
            raise RuntimeError("expected distinct ASCII alphanumeric node labels")
        if any(target != "-" and target not in labels for _, target in links):
            raise RuntimeError("unknown edge target")
        for entries in (roots, live):
            if entries != "-" and (
                len(entries.split(",")) > 16
                or any(label not in labels for label in entries.split(","))
            ):
                raise RuntimeError("invalid root or live labels")
        if live != "-" and live.split(",") != sorted(set(live.split(","))):
            raise RuntimeError("expected sorted, distinct live labels")

    environment = clean_stock_gc_environment()
    environment.pop("ZERO_GC_LIMIT_BYTES", None)
    environment["DOTNET_GCServer"] = "0"
    model = run(
        ["cargo", "run", "--quiet", "--locked", "-p", "gc-rust",
         "--example", "managed_graph_fixtures", "--", str(fixture)],
        env=environment,
        capture_output=True,
    )
    expected_model = [f"model {name} {live}" for name, _, _, live in scenarios]
    if model.stdout.splitlines() != expected_model:
        raise RuntimeError(
            f"Rust model did not match fixtures:\n{model.stdout}\n{model.stderr}"
        )
    log(f"Rust model: {len(scenarios)} fixture live sets verified")

    project = REPOSITORY_ROOT / "samples/GraphFixtures/GraphFixtures.csproj"
    run(["dotnet", "build", str(project), "-warnaserror"], env=environment)
    sample = project.parent / "bin/Debug/net10.0/GraphFixtures.dll"
    collectors = [("stock", environment)]
    if not stock_only:
        shim = build("debug")
        custom = environment.copy()
        custom["DOTNET_GCPath"] = str(shim)
        custom["PATH"] = f"{shim.parent}{os.pathsep}{custom.get('PATH', '')}"
        collectors.append(("zerogc", custom))

    reports = REPOSITORY_ROOT / "out/graph-fixtures"
    reports.mkdir(parents=True, exist_ok=True)
    for collector, env in collectors:
        for name, edges, roots, live in scenarios:
            result = subprocess.run(
                ["dotnet", str(sample), str(fixture), name],
                cwd=REPOSITORY_ROOT, env=env, capture_output=True, text=True, timeout=30,
            )
            for stream in ("stdout", "stderr"):
                (reports / f"{name}.{collector}.{stream}.log").write_text(
                    getattr(result, stream), encoding="utf-8"
                )
            markers = [
                line for line in result.stdout.splitlines()
                if line.startswith(("fixture ", "graph "))
            ]
            expected = [
                f"fixture {name} constructed",
                f"fixture {name} observation declared-only raw-handoff=unsupported",
                f"graph {name} {edges} {roots} {live}",
                f"fixture {name} continuation",
            ]
            output = result.stdout + result.stderr
            if (result.returncode != 0 or markers != expected
                    or INTERFACE_SHELL_DIAGNOSTIC in output
                    or (collector == "zerogc" and LOADER_DIAGNOSTIC not in output)
                    or (collector == "stock"
                        and (LOADER_DIAGNOSTIC in output or result.stderr))):
                raise RuntimeError(f"{name}/{collector} failed ({result.returncode}):\n{output}")
            log(f"{name}/{collector}: graph verified, continuation reached; "
                "raw handoff unsupported")
    if stock_only:
        log("ZeroGC: not run (--stock-only); full checkpoint requires Windows x64")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stock-only", action="store_true", help="explicitly skip Windows ZeroGC")
    try:
        check(parser.parse_args().stock_only)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        log(f"error: {error}", is_error=True)
        raise SystemExit(1)
