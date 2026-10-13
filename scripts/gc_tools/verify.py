"""Check Rust code and run LoaderSmoke with the stock collector."""

from .constants import LOADER_SMOKE_OUTPUT
from .utils import build_sample, clean_stock_gc_environment, log, run


def verify() -> None:
    """Run the repeatable Windows checks that do not need the nightly Miri toolchain."""
    run(["cargo", "fmt", "--all", "--", "--check"])
    run(
        [
            "cargo", "clippy", "--workspace", "--all-targets", "--locked",
            "--", "-D", "warnings",
        ]
    )
    run(["cargo", "test", "--workspace", "--locked"])

    environment = clean_stock_gc_environment()
    sample = build_sample("LoaderSmoke", env=environment)
    result = run(
        ["dotnet", str(sample)],
        env=environment,
        capture_output=True,
    )
    if result.stdout.strip() != LOADER_SMOKE_OUTPUT or result.stderr:
        raise RuntimeError(
            "the stock-GC LoaderSmoke output did not match "
            f"{LOADER_SMOKE_OUTPUT!r}"
        )
    log(f"Stock-GC LoaderSmoke output: {LOADER_SMOKE_OUTPUT}")
