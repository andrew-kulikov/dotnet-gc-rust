"""Locate Windows build tools and check the pinned CoreCLR source."""

import os
from pathlib import Path
import re
import shutil

from .constants import REPOSITORY_ROOT, SUBMODULE_PATH
from .utils import run


def recorded_submodule_commit() -> str:
    result = run(
        ["git", "ls-files", "--stage", "--", SUBMODULE_PATH.as_posix()],
        capture_output=True,
    )
    fields = result.stdout.split()
    if len(fields) < 2 or fields[0] != "160000":
        raise RuntimeError(f"{SUBMODULE_PATH} is not recorded as a Git submodule")
    return fields[1]


def check_runtime_submodule() -> None:
    expected_commit = recorded_submodule_commit()
    submodule = REPOSITORY_ROOT / SUBMODULE_PATH
    interface_header = submodule / "src/coreclr/gc/gcinterface.h"
    update_command = (
        "git -c core.longpaths=true submodule update --init --depth 1 -- "
        f"{SUBMODULE_PATH.as_posix()}"
    )
    if not (submodule / ".git").exists() or not interface_header.is_file():
        raise RuntimeError(f"CoreCLR source is missing. Run: {update_command}")
    current_commit = run(
        ["git", "rev-parse", "HEAD"], cwd=submodule, capture_output=True
    ).stdout.strip()
    if current_commit != expected_commit:
        raise RuntimeError(
            f"dotnet/runtime is at {current_commit[:12]}, expected "
            f"{expected_commit[:12]}. Run: {update_command}"
        )


def locate_visual_studio() -> Path:
    candidates: list[Path] = []
    vswhere = shutil.which("vswhere.exe")
    if vswhere is None:
        installer = os.environ.get("ProgramFiles(x86)")
        if installer:
            candidate = Path(installer) / "Microsoft Visual Studio/Installer/vswhere.exe"
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
        if all((candidate / name).is_file() for name in ("dbghelp.dll", "symsrv.dll")):
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
