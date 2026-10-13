# Graph reachability

Run the standalone Rust model from the repository root:

```sh
cargo run -p gc-rust --example graph_reachability
```

The example builds four objects through `Graph::add_object` and links them
through `Graph::add_edge`. The returned IDs identify objects within that graph.
The root list is supplied separately to `trace`.

Initially there are two disconnected cycles:

```text
root <-> child       detached_a <-> detached_b
```

With `root` as the only root, exactly `root` and `child` are reachable.
After adding `child -> detached_a`, all four objects are reachable. Each run
prints the references, reachability, and traversal statistics. Both runs leave
all four objects stored in the graph.

`Graph::objects` lists stored IDs; `Graph::references` reads an object's outgoing
references. Neither exposes mutable storage. Missing endpoints are rejected by
`add_edge`, and `trace` checks every root and edge before returning a result.

This is the model required by mission 05. Missions 07-10 connect shared managed
fixtures to native object reading and tracing. Missions 15-16 add model and
runtime handle semantics.
An object's references are its entire representation at this stage.

## Model collection

Run mission 06's two-phase collection example:

```sh
cargo run -p gc-rust --example graph_collection
```

The same two disconnected cycles initially contain four objects. Marking from
`root` finds two live objects without deleting anything; sweep removes the
other two. Allocation then resumes with a fresh ID. A second collection with
no roots removes all three remaining objects.

`Graph::collect(&roots)` performs both phases. To inspect marking first, use
`Graph::begin_collection().mark(&roots)`, inspect `Marked::trace_result`, then
call `Marked::sweep`. The phase values borrow the graph exclusively, preventing
allocation, edge mutation, or another collection while that borrow is used.
Only successful marking produces a value with a `sweep` method, which consumes
the value so it cannot be reused. These restrictions are checked by compile-fail
doctests, rather than runtime phase checks.

Dropping either phase abandons collection without mutation. A marking error
also releases the borrow and leaves objects, edges, and the ID counter unchanged,
available for inspection. An invalid graph remains invalid; subsequent marking
continues to reject it. Collection counts reconcile as `before = live + reclaimed`
and `after = live`. IDs are not reused, so a stale root cannot identify a new
object. Collection deletes map entries only; it does not reclaim managed memory.
