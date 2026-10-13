"""Check custom-GC initialization and Server GC rejection."""

import os
import shutil

from .constants import (
    INTERFACE_SHELL_DIAGNOSTIC,
    LOADER_DIAGNOSTIC,
    LOADER_SMOKE_OUTPUT,
    SERVER_GC_DIAGNOSTIC,
    SYMBOL_CACHE,
)
from .utils import (
    build_sample, clean_stock_gc_environment, custom_gc_environment, log, print_output, run,
)
from .toolchain import locate_debugging_tools
from .build import build


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

    sample = build_sample("LoaderSmoke", env=environment).with_suffix(".exe")
    if symbol_server is not None:
        debugging_tools = locate_debugging_tools()
        for library in ("dbghelp.dll", "symsrv.dll"):
            shutil.copy2(debugging_tools / library, sample.parent / library)
        log(f"Using DbgHelp and SymSrv from {debugging_tools}")

    environment = custom_gc_environment(shim, environment)
    result = run(
        [str(sample)],
        env=environment,
        capture_output=True,
        check=False,
    )
    print_output(result)

    output = result.stdout + result.stderr
    if result.returncode != 0:
        raise RuntimeError(
            f"the loader exited with code {result.returncode}; expected zero"
        )
    if (
        LOADER_DIAGNOSTIC not in output
        or LOADER_SMOKE_OUTPUT not in result.stdout.splitlines()
        or "GC initialization failed" in output
        or INTERFACE_SHELL_DIAGNOSTIC in output
    ):
        raise RuntimeError("the loader failed initialization or exited unexpectedly")

    log("Loader smoke test completed successfully")

    server_environment = environment.copy()
    server_environment["DOTNET_GCServer"] = "1"
    server_result = run(
        [str(sample)],
        env=server_environment,
        capture_output=True,
        check=False,
    )
    server_output = server_result.stdout + server_result.stderr
    if server_result.returncode == 0 or SERVER_GC_DIAGNOSTIC not in server_output:
        print_output(server_result)
        raise RuntimeError("the shim did not reject Server GC as expected")
    log("Loader smoke test rejected Server GC")
