#!/usr/bin/env python3
"""Build and check the Windows x64 CoreCLR GC shim."""

import argparse
import os
from pathlib import Path
import subprocess

from gc_tools.build import build
from gc_tools.matrix import matrix
from gc_tools.miri import miri
from gc_tools.smoke import smoke
from gc_tools.verify import verify
from gc_tools.constants import MICROSOFT_SYMBOL_SERVER, SYMBOL_CACHE
from gc_tools.utils import add_log_level_argument, log


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    add_log_level_argument(parser)
    subparsers = parser.add_subparsers(dest="command", required=True)

    # verify
    subparsers.add_parser(
        "verify",
        help="run formatting, lint, Rust tests, and the stock-GC sample"
    )

    # miri
    subparsers.add_parser(
        "miri",
        help="run gc-rust tests under the pinned Miri"
    )

    # matrix
    matrix_parser = subparsers.add_parser(
        "matrix",
        help="run bounded scenarios under both GCs"
    )
    matrix_parser.add_argument(
        "--report-dir",
        type=Path,
        metavar="DIRECTORY",
        help="write each scenario's stdout and stderr to separate files",
    )

    # build
    build_parser = subparsers.add_parser("build")
    build_parser.add_argument(
        "--configuration",
        choices=("debug", "release"),
        default="debug"
    )

    # smoke
    smoke_parser = subparsers.add_parser("smoke")
    smoke_parser.add_argument(
        "--configuration",
        choices=("debug", "release"),
        default="debug"
    )
    smoke_parser.add_argument(
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
    if arguments.log_level is not None:
        os.environ["RUST_GC_LOG_LEVEL"] = arguments.log_level
    try:
        if arguments.command == "verify":
            verify()
        elif arguments.command == "miri":
            miri()
        elif arguments.command == "matrix":
            matrix(arguments.report_dir)
        elif arguments.command == "build":
            build(arguments.configuration)
        elif arguments.command == "smoke":
            smoke(arguments.configuration, arguments.symbol_server)
    except (OSError, RuntimeError, subprocess.SubprocessError) as error:
        log(f"error: {error}", is_error=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
