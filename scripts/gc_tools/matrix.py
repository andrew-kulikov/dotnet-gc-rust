"""Check isolated LoaderSmoke scenarios under the stock collector and ZeroGC."""

from pathlib import Path
import re
import subprocess

from .constants import (
    DEFAULT_MATRIX_LIMIT_BYTES,
    EXHAUST_LIMIT_BYTES,
    INTERFACE_SHELL_DIAGNOSTIC,
    LOADER_SMOKE_OUTPUT,
    REPOSITORY_ROOT,
    SCENARIOS,
    UNSUPPORTED_METHODS,
)
from .utils import (
    build_sample, custom_gc_environment, log, run_scenario, workstation_environment,
)
from .build import build


def check_counters(output: str, expected_limit: int, *, failed: bool) -> tuple[int, int]:
    matches = re.findall(
        r"ZeroGC counters: requests=(\d+) successful=(\d+) "
        r"requested_bytes=(\d+) owned_bytes=(\d+) limit=(\d+) "
        r"ranges=(\d+) registry_bytes=(\d+) disjoint=(true|false) gaps=(\d+)",
        output,
    )
    if not matches:
        raise RuntimeError("ZeroGC did not report allocation counters")
    fields = matches[-1]
    requests, successful, requested, owned, limit, ranges, registry_bytes = map(int, fields[:7])
    disjoint = fields[7] == "true"
    gaps = int(fields[8])
    if not (
        requests >= successful > 0
        and requested > 0
        and owned <= limit == expected_limit
        and ranges == successful
        and registry_bytes >= ranges * 16
        and disjoint
        and gaps > 0
        and output.count("ZeroGC native range:") >= 2
        and "ZeroGC heap walk: unavailable" in output
        and requests == successful + int(failed)
    ):
        raise RuntimeError(f"ZeroGC counters did not reconcile: {fields}")
    return successful, owned


def success_line(scenario: str) -> str:
    return LOADER_SMOKE_OUTPUT if scenario == "smoke" else f"scenario {scenario} ok"


def check_custom_result(
    scenario: str, result: subprocess.CompletedProcess[str], expected_limit: int
) -> tuple[int, int]:
    output = result.stdout + result.stderr
    methods = re.findall(rf"{re.escape(INTERFACE_SHELL_DIAGNOSTIC)} (\w+)", output)
    if scenario == "exhaust":
        valid = (
            result.returncode != 0
            and "ZeroGC: allocation limit exceeded" in output
            and "dotnet-gc-rust: IGCHeap::Alloc failed" in output
            and not methods
        )
    elif scenario in UNSUPPORTED_METHODS:
        valid = (
            result.returncode != 0
            and bool(methods)
            and methods[-1] == UNSUPPORTED_METHODS[scenario]
        )
    else:
        valid = (
            result.returncode == 0
            and success_line(scenario) in result.stdout.splitlines()
            and not methods
        )
    if not valid:
        raise RuntimeError(
            f"ZeroGC {scenario}: unexpected defect ({result.returncode}):\n{output}"
        )
    return check_counters(output, expected_limit, failed=scenario == "exhaust")


def matrix(report_dir: Path | None = None) -> None:
    if report_dir is not None:
        report_dir = (REPOSITORY_ROOT / report_dir).resolve()
        report_dir.mkdir(parents=True, exist_ok=True)

    shim = build("debug")
    environment = workstation_environment()
    sample = build_sample("LoaderSmoke", env=environment).with_suffix(".exe")
    custom_environment = custom_gc_environment(shim, environment)

    for scenario in SCENARIOS:
        command = [str(sample)] if scenario == "smoke" else [str(sample), scenario]
        stock = run_scenario(
            command,
            env=environment,
            report_prefix=report_dir / f"{scenario}.stock" if report_dir else None,
        )
        if stock.returncode != 0 or success_line(scenario) not in stock.stdout.splitlines():
            raise RuntimeError(
                f"stock GC {scenario} failed ({stock.returncode}):\n"
                f"{stock.stdout}\n{stock.stderr}"
            )

        scenario_environment = custom_environment.copy()
        expected_limit = DEFAULT_MATRIX_LIMIT_BYTES
        attempts = 1
        if scenario == "exhaust":
            expected_limit = EXHAUST_LIMIT_BYTES
            scenario_environment["ZERO_GC_LIMIT_BYTES"] = str(expected_limit)
            attempts = 2

        counts = []
        for attempt in range(1, attempts + 1):
            suffix = f".{attempt}" if attempts > 1 else ""
            label = f"{scenario}.zerogc{suffix}"
            custom = run_scenario(
                command,
                env=scenario_environment,
                report_prefix=report_dir / label if report_dir else None,
            )
            counts.append(check_custom_result(scenario, custom, expected_limit))

        if scenario == "exhaust":
            if counts[0] != counts[1]:
                raise RuntimeError(f"exhaustion was not reproducible: {counts}")
            outcome = "deterministic exhaustion at IGCHeap::Alloc"
        elif scenario in UNSUPPORTED_METHODS:
            outcome = f"unsupported by {UNSUPPORTED_METHODS[scenario]}"
        else:
            outcome = "works"
        log(f"{scenario}: stock GC works; ZeroGC {outcome}")

    if report_dir is not None:
        log(f"Matrix logs: {report_dir}")
