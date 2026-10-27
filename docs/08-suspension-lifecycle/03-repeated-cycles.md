# 3. Repeated cycles with two mutators

[Mission overview](README.md) · [Previous: the first empty cycle](02-empty-cycle.md)

This records the initial empty-operation experiment. [Chapter 4](04-handoff-checkpoint.md)
extends the same command with handle handoff and an injected operation failure;
its logs replace the initial experiment's logs on rerun.

The initial `python scripts/graph_fixtures.py --fixture chain --suspend` baseline
passed before changes: one restart attempt and managed continuation.

`--suspend-repeat` adds eight cycles on the main thread while a second managed
thread changes a reference in its own preallocated two-node graph. Each cycle
requires an atomic progress counter to change before entry and again after
return. The worker has a ten-second lifetime bound; each progress wait and join
has a five-second bound. The process retains the runner's 30-second timeout.
The owner holds no worker lock across native entry. Cleanup requests worker exit
and joins it in `finally`.

The first experiment exposed missing dependent-handle callbacks during formatting.
[Chapter 5](05-dependent-handles.md) resolves this; the fixture now uses ordinary
interpolation, including cycle numbers.

Command: `python scripts/graph_fixtures.py --fixture chain --suspend-repeat`.
Observed result: exit 0, eight successful cycles with one restart attempt each,
worker progress before/after each, worker exit, graph match, and continuation.
ZeroGC also called `FixAllocContext` during worker cleanup. Logs are under
`out/graph-fixtures/chain.zerogc.suspend-repeat.{stdout,stderr}.log`.

This evidence establishes repeated runtime suspension and continuation with two
application mutators. Progress samples do not independently prove the worker
was stationary inside the empty window or that a particular EE wait branch ran.
The pinned `SuspendEE` contract supplies that guarantee; native event tests cover
the completion condition independently.

Next action: validate owned normal-handle handoff, limit borrowed references to
the stopped operation, then include injected post-suspend failure in the full
checkpoint. No object fields or runtime roots are inspected here.
