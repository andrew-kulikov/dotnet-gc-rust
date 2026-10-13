"""Build the Rust library and native shim, then verify its exports."""

import os
from pathlib import Path

from .constants import REPOSITORY_ROOT
from .utils import log, run
from .toolchain import (
    check_runtime_submodule,
    locate_cmake,
    locate_dumpbin,
    locate_visual_studio,
    select_visual_studio_generator,
)


def build(configuration: str) -> Path:
    if os.name != "nt":
        raise RuntimeError("the native shim currently supports Windows only")
    check_runtime_submodule()

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

    rust_output = cargo_target / configuration
    cmake_configuration = "Release" if configuration == "release" else "Debug"
    build_directory = REPOSITORY_ROOT / "out/build/native-shim" / configuration

    run(
        [
            str(cmake),
            "-S", str(REPOSITORY_ROOT / "native-shim"),
            "-B", str(build_directory),
            "-G", generator,
            "-A", "x64",
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


def verify_exports(shim: Path, dumpbin: Path, environment: dict[str, str]) -> None:
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
