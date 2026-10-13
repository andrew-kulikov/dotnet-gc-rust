"""Run the collector's tests under the pinned Miri interpreter."""

from .constants import MIRI_TOOLCHAIN
from .utils import run


def miri() -> None:
    run(["cargo", f"+{MIRI_TOOLCHAIN}", "miri", "setup"])
    run(["cargo", f"+{MIRI_TOOLCHAIN}", "miri", "test", "--locked", "-p", "gc-rust"])
