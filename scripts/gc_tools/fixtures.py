"""Read and validate the bounded graph format shared by managed and Rust samples."""

from dataclasses import dataclass
from pathlib import Path

from .constants import MAX_FIXTURE_NODES, MAX_FIXTURE_SCENARIOS


@dataclass(frozen=True)
class GraphFixture:
    name: str
    edges: str
    roots: str
    live: str


def parse_fixture(line: str) -> GraphFixture:
    fields = line.split()
    if len(fields) != 4:
        raise RuntimeError("expected name, edges, roots, and live labels")
    fixture = GraphFixture(*fields)
    links = [link.split(":") for link in fixture.edges.split(",")]
    if not 1 <= len(links) <= MAX_FIXTURE_NODES or any(len(link) != 2 for link in links):
        raise RuntimeError("invalid bounded links")

    labels = {source for source, _ in links}
    if len(labels) != len(links) or any(
        not label.isascii() or not label.isalnum() for label in labels
    ):
        raise RuntimeError("expected distinct ASCII alphanumeric node labels")
    if any(target != "-" and target not in labels for _, target in links):
        raise RuntimeError("unknown edge target")

    for entries in (fixture.roots, fixture.live):
        if entries == "-":
            continue
        values = entries.split(",")
        if len(values) > MAX_FIXTURE_NODES or any(label not in labels for label in values):
            raise RuntimeError("invalid root or live labels")
    if fixture.live != "-" and fixture.live.split(",") != sorted(set(fixture.live.split(","))):
        raise RuntimeError("expected sorted, distinct live labels")
    return fixture


def load_fixtures(path: Path) -> list[GraphFixture]:
    lines = [
        line.strip()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]
    if not 1 <= len(lines) <= MAX_FIXTURE_SCENARIOS:
        raise RuntimeError(f"expected 1..{MAX_FIXTURE_SCENARIOS} scenarios")
    fixtures = [parse_fixture(line) for line in lines]
    if len({fixture.name for fixture in fixtures}) != len(fixtures):
        raise RuntimeError("duplicate scenario")
    return fixtures
