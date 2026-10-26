#!/usr/bin/env python3
"""Check shared graph fixtures with the Rust model, stock GC, and Windows ZeroGC."""

import argparse
import os
from pathlib import Path
import subprocess

from gc_tools.build import build
from gc_tools.constants import LOADER_DIAGNOSTIC, REPOSITORY_ROOT, SCENARIO_TIMEOUT_SECONDS
from gc_tools.fixtures import GraphFixture, load_fixtures
from gc_tools.utils import (
    add_log_level_argument, build_sample, custom_gc_environment, log, print_output, run_scenario,
    workstation_environment,
)


def run_process(
    command: list[str], environment: dict[str, str], report_name: str,
    timeout: float | None = None,
) -> list[str]:
    """Print both streams, fail on nonzero exit, and return stdout then stderr lines."""
    result = run_scenario(
        command, env=environment,
        report_prefix=REPOSITORY_ROOT / "out/graph-fixtures" / report_name,
        timeout=timeout,
    )
    print_output(result)
    result.check_returncode()
    return result.stdout.splitlines() + result.stderr.splitlines()


def select_fixtures(path: Path, name: str | None) -> list[GraphFixture]:
    fixtures = load_fixtures(path)
    if name is None:
        return fixtures
    selected = [fixture for fixture in fixtures if fixture.name == name]
    if not selected:
        raise RuntimeError(f"unknown fixture: {name}")
    return selected


def check_model(
    path: Path, fixtures: list[GraphFixture], environment: dict[str, str],
    fixture_name: str | None = None,
) -> None:
    command = [
        "cargo", "run", "--quiet", "--locked", "-p", "gc-rust",
        "--example", "managed_graph_fixtures", "--", str(path),
    ]
    if fixture_name:
        command.append(fixture_name)
    lines = run_process(command, environment, "model")
    expected = [f"model {fixture.name} {fixture.live}" for fixture in fixtures]
    if [line for line in lines if line.startswith("model ")] != expected:
        raise RuntimeError("Rust model did not match fixtures")


def check_fixture_output(
    fixture: GraphFixture, collector: str, lines: list[str],
    suspend: bool = False,
) -> None:
    markers = [
        line for line in lines
        if line.startswith(("dotnet: fixture ", "dotnet: graph "))
    ]
    expected = [
        f"dotnet: fixture '{fixture.name}' constructed",
        f"dotnet: fixture '{fixture.name}' observation declared-only raw-handoff=unsupported",
    ]
    if suspend:
        expected.append(
            f"dotnet: fixture '{fixture.name}' diagnostic suspension completed restart-attempts=1"
        )
    expected += [
        f"dotnet: graph '{fixture.name}' {fixture.edges} {fixture.roots} {fixture.live}",
        f"dotnet: fixture '{fixture.name}' continuation",
    ]
    if markers != expected:
        raise RuntimeError(f"{fixture.name}/{collector}: unexpected fixture records")
    custom_gc_loaded = LOADER_DIAGNOSTIC in lines
    if collector == "zerogc" and not custom_gc_loaded:
        raise RuntimeError(f"{fixture.name}: ZeroGC did not initialize")
    if collector == "stock" and custom_gc_loaded:
        raise RuntimeError(f"{fixture.name}: stock run loaded the custom GC")


def run_fixture(
    sample: Path, fixture: GraphFixture, collector: str,
    environment: dict[str, str], suspend: bool,
) -> None:
    print(f"\n--- {collector}: {fixture.name} ---", flush=True)
    command = ["dotnet", str(sample), fixture.name, fixture.edges, fixture.roots, fixture.live]
    if suspend:
        command.append("--suspend")
    suffix = ".suspend" if suspend else ""
    lines = run_process(command, environment, f"{fixture.name}.{collector}{suffix}",
                        timeout=SCENARIO_TIMEOUT_SECONDS)
    check_fixture_output(fixture, collector, lines, suspend)


def check(
    stock_only: bool = False, scenario: str = "scenarios",
    fixture_name: str | None = None, suspend: bool = False,
) -> None:
    if stock_only and suspend:
        raise RuntimeError("--suspend requires ZeroGC; omit --stock-only")
    path = REPOSITORY_ROOT / f"samples/GraphFixtures/{scenario}.txt"
    fixtures = select_fixtures(path, fixture_name)
    environment = workstation_environment()
    print("\n=== Rust model ===", flush=True)
    check_model(path, fixtures, environment, fixture_name)
    print("\n=== Build C# fixture ===", flush=True)
    sample = build_sample("GraphFixtures", env=environment, warnings_as_errors=True)

    collectors = [("stock", environment)]
    if not stock_only:
        print("\n=== Build Rust GC and C++ shim ===", flush=True)
        shim = build("debug")
        collectors.append(("zerogc", custom_gc_environment(shim, environment)))

    for collector, env in collectors:
        for fixture in fixtures:
            run_fixture(sample, fixture, collector, env, suspend and collector == "zerogc")
    print("\nGraph fixtures passed. Logs: out/graph-fixtures/", flush=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    add_log_level_argument(parser)
    parser.add_argument(
        "--stock-only", action="store_true", help="explicitly skip Windows ZeroGC"
    )
    parser.add_argument(
        "--scenario", default="scenarios", help="fixture file stem (default: scenarios)"
    )
    parser.add_argument("--fixture", help="run only the named fixture, e.g. chain")
    parser.add_argument(
        "--suspend", action="store_true", help="run one empty diagnostic cycle under ZeroGC"
    )
    arguments = parser.parse_args()
    if arguments.log_level is not None:
        os.environ["RUST_GC_LOG_LEVEL"] = arguments.log_level
    try:
        check(arguments.stock_only, arguments.scenario, arguments.fixture, arguments.suspend)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        log(f"error: {error}", is_error=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
