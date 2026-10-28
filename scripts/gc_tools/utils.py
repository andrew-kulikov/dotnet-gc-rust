"""Shared process, reporting, and managed-sample helpers."""

import argparse
from collections.abc import Mapping
import os
from pathlib import Path
import shlex
import subprocess
import sys
from typing import TextIO

from .constants import REPOSITORY_ROOT, SCENARIO_TIMEOUT_SECONDS, TARGET_FRAMEWORK


def add_log_level_argument(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--log-level",
        type=str.lower,
        choices=("error", "warn", "info", "debug", "trace"),
        help=(
            "set Rust collector log verbosity for child processes "
            "(default: inherited RUST_GC_LOG_LEVEL, otherwise info)"
        ),
    )


def _colorize(message: str, color: int, stream: TextIO) -> str:
    if stream.isatty():
        return f"\033[{color}m{message}\033[0m"
    return message


def log(message: str, *, is_error: bool = False) -> None:
    stream = sys.stderr if is_error else sys.stdout
    output = f"build-py: {message}"
    if is_error:
        output = _colorize(output, 31, stream)
    print(output, file=stream, flush=True)


def display_command(command: list[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    return shlex.join(command)


def run(
    command: list[str],
    *,
    cwd: Path = REPOSITORY_ROOT,
    env: dict[str, str] | None = None,
    capture_output: bool = False,
    check: bool = True,
    timeout: float | None = None,
) -> subprocess.CompletedProcess[str]:
    log(f"{_colorize(display_command(command), 32, sys.stdout)}\n")
    return subprocess.run(
        command,
        cwd=cwd,
        env=env,
        capture_output=capture_output,
        check=check,
        text=True,
        encoding="utf-8",
        timeout=timeout,
    )


def run_scenario(
    command: list[str],
    *,
    env: dict[str, str],
    report_prefix: Path | None = None,
    timeout: float | None = SCENARIO_TIMEOUT_SECONDS,
) -> subprocess.CompletedProcess[str]:
    """Capture a bounded scenario, saving both streams even when it fails."""
    result = run(
        command,
        env=env,
        capture_output=True,
        check=False,
        timeout=timeout,
    )
    if report_prefix is not None:
        report_prefix.parent.mkdir(parents=True, exist_ok=True)
        for stream in ("stdout", "stderr"):
            report_prefix.with_name(f"{report_prefix.name}.{stream}.log").write_text(
                getattr(result, stream), encoding="utf-8"
            )
    return result


def print_output(result: subprocess.CompletedProcess[str]) -> None:
    for output, stream in ((result.stdout, sys.stdout), (result.stderr, sys.stderr)):
        if output:
            # Rust's tree output is UTF-8, including when stdout is redirected.
            stream.reconfigure(encoding="utf-8")
            if stream is sys.stderr:
                output = _colorize(output, 31, stream)
            stream.write(output)
            stream.flush()


def _without_variables(environment: Mapping[str, str], *names: str) -> dict[str, str]:
    removed = {name.casefold() for name in names}
    return {key: value for key, value in environment.items() if key.casefold() not in removed}


def clean_stock_gc_environment() -> dict[str, str]:
    """Remove custom collector selectors, including mixed-case inherited names."""
    return _without_variables(
        os.environ, "DOTNET_GCPath", "DOTNET_GCName", "COMPlus_GCPath", "COMPlus_GCName"
    )


def workstation_environment() -> dict[str, str]:
    environment = _without_variables(
        clean_stock_gc_environment(), "ZERO_GC_LIMIT_BYTES", "DOTNET_GCServer"
    )
    environment["DOTNET_GCServer"] = "0"
    return environment


def custom_gc_environment(shim: Path, environment: dict[str, str]) -> dict[str, str]:
    custom = _without_variables(environment, "DOTNET_GCPath", "DOTNET_GCServer")
    custom["DOTNET_GCPath"] = str(shim)
    custom["DOTNET_GCServer"] = "0"
    custom["PATH"] = f"{shim.parent}{os.pathsep}{custom.get('PATH', '')}"
    return custom


def build_sample(
    name: str, *, env: dict[str, str], warnings_as_errors: bool = False
) -> Path:
    """Build a sample in Debug and return its managed assembly path."""
    project = REPOSITORY_ROOT / "samples" / name / f"{name}.csproj"
    command = ["dotnet", "build", str(project)]
    if warnings_as_errors:
        command.append("-warnaserror")
    run(command, env=env)
    return project.parent / "bin/Debug" / TARGET_FRAMEWORK / f"{name}.dll"
