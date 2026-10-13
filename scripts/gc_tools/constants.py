"""Repository paths and the contracts used by the managed checks."""

from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
SUBMODULE_PATH = Path("external/dotnet-runtime")
TARGET_FRAMEWORK = "net10.0"
SCENARIO_TIMEOUT_SECONDS = 30
MIRI_TOOLCHAIN = "nightly-2026-08-17"

LOADER_DIAGNOSTIC = "dotnet-gc-rust: native shim reached Rust"
INTERFACE_SHELL_DIAGNOSTIC = "dotnet-gc-rust: unimplemented method called:"
SERVER_GC_DIAGNOSTIC = (
    "dotnet-gc-rust: unsupported configuration: Server GC is enabled; "
    "only workstation GC is supported"
)
LOADER_SMOKE_OUTPUT = "Hello, World!"

SCENARIOS = ("smoke", "small", "large", "threads", "pin", "finalize", "exhaust")
UNSUPPORTED_METHODS = {
    "large": "GetLOHThreshold",
    "threads": "FixAllocContext",
    "finalize": "RegisterForFinalization",
}
DEFAULT_MATRIX_LIMIT_BYTES = 64 * 1024 * 1024
EXHAUST_LIMIT_BYTES = 1024 * 1024
MAX_FIXTURE_SCENARIOS = 16
MAX_FIXTURE_NODES = 16

SYMBOL_CACHE = Path(r"D:\temp\symbol-cache")
MICROSOFT_SYMBOL_SERVER = "https://msdl.microsoft.com/download/symbols"
