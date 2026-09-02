#!/usr/bin/env python3
"""Bootstrap and build the Windows x64 CoreCLR GC shim."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
SUBMODULE_PATH = Path("external/dotnet-runtime")
LOADER_DIAGNOSTIC = "dotnet-gc-rust: native shim reached Rust"
INTERFACE_SHELL_DIAGNOSTIC = (
    "dotnet-gc-rust: unimplemented method called: CreateGlobalHandleOfType"
)
SERVER_GC_DIAGNOSTIC = (
    "dotnet-gc-rust: unsupported configuration: Server GC is enabled; "
    "only workstation GC is supported"
)
LOADER_SMOKE_OUTPUT = "Hello, World!"
MIRI_TOOLCHAIN = "nightly-2026-08-17"
SYMBOL_CACHE = Path(r"D:\\temp\\symbol-cache")
MICROSOFT_SYMBOL_SERVER = "https://msdl.microsoft.com/download/symbols"


def log(message: str, is_error: bool = False) -> None:
    file = sys.stderr if is_error else sys.stdout
    print(f"build-py: {message}", flush=True, file=file)


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
) -> subprocess.CompletedProcess[str]:
    # change color to green for the command, then reset to default color for the output
    log(f"\033[32m{display_command(command)}\033[0m\n")
    return subprocess.run(
        command,
        cwd=cwd,
        env=env,
        capture_output=capture_output,
        check=check,
        text=True,
    )


def recorded_submodule_commit() -> str:
    result = run(
        ["git", "ls-files", "--stage", "--", SUBMODULE_PATH.as_posix()],
        capture_output=True,
    )
    fields = result.stdout.split()
    if len(fields) < 2 or fields[0] != "160000":
        raise RuntimeError(f"{SUBMODULE_PATH} is not recorded as a Git submodule")
    return fields[1]


def bootstrap() -> None:
    expected_commit = recorded_submodule_commit()
    submodule = REPOSITORY_ROOT / SUBMODULE_PATH
    interface_header = submodule / "src/coreclr/gc/gcinterface.h"
    vm_source = submodule / "src/coreclr/vm/gcheaputilities.cpp"

    if (submodule / ".git").exists():
        current_commit = run(
            ["git", "rev-parse", "HEAD"], cwd=submodule, capture_output=True
        ).stdout.strip()
        sparse_checkout = run(
            ["git", "config", "--bool", "core.sparseCheckout"],
            cwd=submodule,
            capture_output=True,
            check=False,
        )
        if (
            current_commit == expected_commit
            and sparse_checkout.stdout.strip() != "true"
            and interface_header.is_file()
            and vm_source.is_file()
        ):
            log(f"dotnet/runtime is already bootstrapped at {expected_commit[:12]}")
            return

        status = run(
            ["git", "status", "--porcelain"], cwd=submodule, capture_output=True
        )
        if status.stdout.strip():
            raise RuntimeError(
                "dotnet/runtime has local changes; refusing to change its checkout"
            )

    if not (submodule / ".git").exists():
        run(
            [
                "git",
                "-c",
                "core.longpaths=true",
                "submodule",
                "update",
                "--init",
                "--depth",
                "1",
                "--",
                SUBMODULE_PATH.as_posix(),
            ],
        )

    run(["git", "config", "core.longpaths", "true"], cwd=submodule)
    sparse_checkout = run(
        ["git", "config", "--bool", "core.sparseCheckout"],
        cwd=submodule,
        capture_output=True,
        check=False,
    )
    if sparse_checkout.stdout.strip() == "true":
        run(["git", "sparse-checkout", "disable"], cwd=submodule)

    commit_exists = run(
        ["git", "cat-file", "-e", f"{expected_commit}^{{commit}}"],
        cwd=submodule,
        check=False,
    )
    if commit_exists.returncode != 0:
        run(
            ["git", "fetch", "--depth", "1", "origin", expected_commit],
            cwd=submodule,
        )

    run(["git", "checkout", "--detach", expected_commit], cwd=submodule)

    if not interface_header.is_file():
        raise RuntimeError(f"expected CoreCLR header is missing: {interface_header}")
    if not vm_source.is_file():
        raise RuntimeError(f"expected CoreCLR VM source is missing: {vm_source}")

    log(f"Bootstrapped dotnet/runtime at {expected_commit[:12]}")


def locate_visual_studio() -> Path:
    candidates: list[Path] = []
    vswhere = shutil.which("vswhere.exe")
    if vswhere is None:
        installer = Path(os.environ.get("ProgramFiles(x86)", ""))
        if installer:
            candidate = installer / "Microsoft Visual Studio/Installer/vswhere.exe"
            if candidate.is_file():
                vswhere = str(candidate)

    if vswhere is not None:
        result = run(
            [
                vswhere,
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-property",
                "installationPath",
            ],
            capture_output=True,
        )
        installation = result.stdout.strip()
        if installation:
            candidates.append(Path(installation))

    for variable in ("ProgramFiles", "ProgramFiles(x86)"):
        root_text = os.environ.get(variable)
        if not root_text:
            continue
        visual_studio_root = Path(root_text) / "Microsoft Visual Studio"
        candidates.extend(sorted(visual_studio_root.glob("*/*"), reverse=True))

    for candidate in candidates:
        if (candidate / "VC/Tools/MSVC").is_dir():
            return candidate.resolve()
    raise RuntimeError("Visual Studio with the MSVC x64 tools was not found")


def locate_cmake(visual_studio: Path) -> Path:
    from_path = shutil.which("cmake.exe")
    if from_path is not None:
        return Path(from_path)
    bundled = (
        visual_studio
        / "Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin/cmake.exe"
    )
    if bundled.is_file():
        return bundled
    raise RuntimeError("cmake.exe was not found in PATH or Visual Studio")


def locate_dumpbin(visual_studio: Path) -> Path:
    candidates = sorted(
        (visual_studio / "VC/Tools/MSVC").glob("*/bin/Hostx64/x64/dumpbin.exe"),
        reverse=True,
    )
    if not candidates:
        raise RuntimeError("dumpbin.exe was not found in Visual Studio")
    return candidates[0]


def locate_debugging_tools() -> Path:
    candidates: list[Path] = []
    for variable in ("ProgramFiles(x86)", "ProgramFiles"):
        root_text = os.environ.get(variable)
        if not root_text:
            continue
        windows_kits = Path(root_text) / "Windows Kits"
        candidates.extend(
            sorted(windows_kits.glob("*/Debuggers/x64"), reverse=True)
        )

    for candidate in candidates:
        if all(
            (candidate / name).is_file()
            for name in ("dbghelp.dll", "symsrv.dll")
        ):
            return candidate.resolve()
    raise RuntimeError(
        "Windows Debugging Tools with x64 dbghelp.dll and symsrv.dll were not found"
    )


def select_visual_studio_generator(cmake: Path) -> str:
    result = run([str(cmake), "--help"], capture_output=True)
    generators = re.findall(r"Visual Studio \d+ \d+", result.stdout)
    if not generators:
        raise RuntimeError("CMake does not advertise a Visual Studio generator")
    return generators[0]


def build(configuration: str) -> Path:
    bootstrap()
    if os.name != "nt":
        raise RuntimeError("the native shim currently supports Windows only")

    visual_studio = locate_visual_studio()
    cmake = locate_cmake(visual_studio)
    dumpbin = locate_dumpbin(visual_studio)
    generator = select_visual_studio_generator(cmake)
    environment = os.environ.copy()
    cargo_target = REPOSITORY_ROOT / "target"
    environment["CARGO_TARGET_DIR"] = str(cargo_target)

    cargo_command = ["cargo", "build", "--locked", "-p", "gc-rust"]
    if configuration == "release":
        cargo_command.append("--release")
    run(cargo_command, env=environment)

    rust_profile = "release" if configuration == "release" else "debug"
    rust_output = cargo_target / rust_profile
    cmake_configuration = "Release" if configuration == "release" else "Debug"
    build_directory = REPOSITORY_ROOT / "out/build/native-shim" / configuration

    run(
        [
            str(cmake),
            "-S",
            str(REPOSITORY_ROOT / "native-shim"),
            "-B",
            str(build_directory),
            "-G",
            generator,
            "-A",
            "x64",
            f"-DRUST_OUTPUT_DIR={rust_output}",
        ],
        env=environment,
    )
    run(
        [str(cmake), "--build", str(build_directory), "--config", cmake_configuration],
        env=environment,
    )

    shim = build_directory / "stage/dotnet_gc_shim.dll"
    if not shim.is_file():
        raise RuntimeError(f"native shim was not produced: {shim}")
    verify_exports(shim, dumpbin, environment)
    log(f"Built {shim}")
    return shim


def verify_exports(
    shim: Path, dumpbin: Path, environment: dict[str, str]
) -> None:
    result = run(
        [str(dumpbin), "/exports", str(shim)],
        env=environment,
        capture_output=True,
    )
    missing = [
        export
        for export in ("GC_Initialize", "GC_VersionInfo")
        if export not in result.stdout
    ]
    if missing:
        raise RuntimeError(f"native shim is missing exports: {', '.join(missing)}")
    log("Verified exports: GC_Initialize, GC_VersionInfo")


def clean_stock_gc_environment() -> dict[str, str]:
    """Return the environment used by managed checks that must use the stock GC."""
    environment = os.environ.copy()
    for key in list(environment):
        if key.casefold() in {"dotnet_gcpath", "dotnet_gcname"}:
            del environment[key]
    return environment


def verify() -> None:
    """Run the repeatable Windows checks that do not need the nightly Miri toolchain."""
    run(["cargo", "fmt", "--all", "--", "--check"])
    run(
        [
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ]
    )
    run(["cargo", "test", "--workspace", "--locked"])

    environment = clean_stock_gc_environment()
    project = REPOSITORY_ROOT / "samples/LoaderSmoke/LoaderSmoke.csproj"
    run(["dotnet", "build", str(project)], env=environment)
    result = run(
        ["dotnet", "run", "--no-build", "--project", str(project)],
        env=environment,
        capture_output=True,
    )
    if result.stdout.strip() != LOADER_SMOKE_OUTPUT or result.stderr:
        raise RuntimeError(
            "the stock-GC LoaderSmoke output did not match "
            f"{LOADER_SMOKE_OUTPUT!r}"
        )
    log(f"Stock-GC LoaderSmoke output: {LOADER_SMOKE_OUTPUT}")


def miri() -> None:
    """Run the gc-rust tests under the pinned nightly Miri interpreter."""
    run(["cargo", f"+{MIRI_TOOLCHAIN}", "miri", "setup"])
    run(
        [
            "cargo",
            f"+{MIRI_TOOLCHAIN}",
            "miri",
            "test",
            "--locked",
            "-p",
            "gc-rust",
        ]
    )


def smoke(configuration: str, symbol_server: str | None) -> None:
    shim = build(configuration)
    environment = clean_stock_gc_environment()

    if symbol_server is not None:
        SYMBOL_CACHE.mkdir(parents=True, exist_ok=True)
        symbol_server_path = f"srv*{SYMBOL_CACHE}*{symbol_server}"
        existing_symbol_path = environment.get("_NT_SYMBOL_PATH")
        environment["_NT_SYMBOL_PATH"] = os.pathsep.join(
            part
            for part in (existing_symbol_path, symbol_server_path)
            if part
        )
        log(f"Native symbol path: {environment['_NT_SYMBOL_PATH']}")

    project = REPOSITORY_ROOT / "samples/LoaderSmoke/LoaderSmoke.csproj"
    run(["dotnet", "build", str(project)], env=environment)

    sample = REPOSITORY_ROOT / "samples/LoaderSmoke/bin/Debug/net10.0/LoaderSmoke.exe"
    if symbol_server is not None:
        debugging_tools = locate_debugging_tools()
        for library in ("dbghelp.dll", "symsrv.dll"):
            shutil.copy2(debugging_tools / library, sample.parent / library)
        log(f"Using DbgHelp and SymSrv from {debugging_tools}")

    environment["DOTNET_GCPath"] = str(shim)
    environment["DOTNET_GCServer"] = "0"
    environment["PATH"] = f"{shim.parent}{os.pathsep}{environment.get('PATH', '')}"
    result = run(
        [str(sample)],
        env=environment,
        capture_output=True,
        check=False,
    )
    if result.stdout:
        sys.stdout.write(result.stdout)
        sys.stdout.flush()
    if result.stderr:
        sys.stderr.write(result.stderr)
        sys.stderr.flush()

    output = result.stdout + result.stderr
    if (
        result.returncode == 0
        or LOADER_DIAGNOSTIC not in output
        or INTERFACE_SHELL_DIAGNOSTIC not in output
        or "GC initialization failed" in output
    ):
        raise RuntimeError("the loader did not reach the expected interface-shell boundary")
    log("Loader smoke test reached IGCHandleManager::CreateGlobalHandleOfType")

    server_environment = environment.copy()
    server_environment["DOTNET_GCServer"] = "1"
    server_result = run(
        [str(sample)],
        env=server_environment,
        capture_output=True,
        check=False,
    )
    if server_result.stdout:
        sys.stdout.write(server_result.stdout)
        sys.stdout.flush()
    if server_result.stderr:
        sys.stderr.write(server_result.stderr)
        sys.stderr.flush()

    server_output = server_result.stdout + server_result.stderr
    if server_result.returncode == 0 or SERVER_GC_DIAGNOSTIC not in server_output:
        raise RuntimeError("the shim did not reject Server GC as expected")
    log("Loader smoke test rejected Server GC")


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("bootstrap", help="initialize the pinned runtime submodule")
    subparsers.add_parser(
        "verify", help="run formatting, lint, Rust tests, and the stock-GC sample"
    )
    subparsers.add_parser("miri", help="run gc-rust tests under the pinned Miri")

    for command in ("build", "smoke"):
        command_parser = subparsers.add_parser(command)
        command_parser.add_argument(
            "--configuration",
            choices=("debug", "release"),
            default="debug",
        )

        if command == "smoke":
            command_parser.add_argument(
                "--symbol-server",
                metavar="ADDRESS",
                default=MICROSOFT_SYMBOL_SERVER,
                help=(
                    "symbol server address used by DbgHelp; downloaded symbols "
                    f"are cached under {SYMBOL_CACHE}"
                ),
            )
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        if arguments.command == "bootstrap":
            bootstrap()
        elif arguments.command == "verify":
            verify()
        elif arguments.command == "miri":
            miri()
        elif arguments.command == "build":
            build(arguments.configuration)
        else:
            smoke(arguments.configuration, arguments.symbol_server)
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        log(f"error: {error}", is_error=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
