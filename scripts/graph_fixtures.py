#!/usr/bin/env python3
"""Check shared graph fixtures with the Rust model, stock GC, and Windows ZeroGC."""

import argparse
from pathlib import Path
import subprocess

from gc_tools.build import build
from gc_tools.constants import INTERFACE_SHELL_DIAGNOSTIC, LOADER_DIAGNOSTIC, REPOSITORY_ROOT
from gc_tools.fixtures import GraphFixture, load_fixtures
from gc_tools.utils import (
    build_sample, custom_gc_environment, log, run, run_scenario, workstation_environment,
)


def check_model(
    path: Path, fixtures: list[GraphFixture], environment: dict[str, str]
) -> None:
    model = run(
        [
            "cargo", "run", "--quiet", "--locked", "-p", "gc-rust",
            "--example", "managed_graph_fixtures", "--", str(path),
        ],
        env=environment,
        capture_output=True,
    )
    expected = [f"model {fixture.name} {fixture.live}" for fixture in fixtures]
    if model.stdout.splitlines() != expected:
        raise RuntimeError(
            f"Rust model did not match fixtures:\n{model.stdout}\n{model.stderr}"
        )
    log(f"Rust model: {len(fixtures)} fixture live sets verified")


def check_managed_result(
    fixture: GraphFixture, collector: str, result: subprocess.CompletedProcess[str]
) -> None:
    markers = [
        line for line in result.stdout.splitlines()
        if line.startswith(("fixture ", "graph "))
    ]
    expected = [
        f"fixture {fixture.name} constructed",
        f"fixture {fixture.name} observation declared-only raw-handoff=unsupported",
        f"graph {fixture.name} {fixture.edges} {fixture.roots} {fixture.live}",
        f"fixture {fixture.name} continuation",
    ]
    output = result.stdout + result.stderr
    if (
        result.returncode != 0
        or markers != expected
        or INTERFACE_SHELL_DIAGNOSTIC in output
        or (collector == "zerogc" and LOADER_DIAGNOSTIC not in output)
        or (collector == "stock" and (LOADER_DIAGNOSTIC in output or result.stderr))
    ):
        raise RuntimeError(
            f"{fixture.name}/{collector} failed ({result.returncode}):\n{output}"
        )


def check(stock_only: bool = False) -> None:
    path = REPOSITORY_ROOT / "samples/GraphFixtures/scenarios.txt"
    fixtures = load_fixtures(path)
    environment = workstation_environment()
    check_model(path, fixtures, environment)
    sample = build_sample("GraphFixtures", env=environment, warnings_as_errors=True)

    collectors = [("stock", environment)]
    if not stock_only:
        shim = build("debug")
        collectors.append(("zerogc", custom_gc_environment(shim, environment)))

    reports = REPOSITORY_ROOT / "out/graph-fixtures"
    for collector, env in collectors:
        for fixture in fixtures:
            result = run_scenario(
                ["dotnet", str(sample), fixture.name, fixture.edges, fixture.roots, fixture.live],
                env=env,
                report_prefix=reports / f"{fixture.name}.{collector}",
            )
            check_managed_result(fixture, collector, result)
            log(
                f"{fixture.name}/{collector}: graph verified, continuation reached; "
                "raw handoff unsupported"
            )
    if stock_only:
        log("ZeroGC: not run (--stock-only); full checkpoint requires Windows x64")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--stock-only", action="store_true", help="explicitly skip Windows ZeroGC"
    )
    arguments = parser.parse_args()
    try:
        check(arguments.stock_only)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        log(f"error: {error}", is_error=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
