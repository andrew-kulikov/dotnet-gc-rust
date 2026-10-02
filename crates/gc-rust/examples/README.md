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

This is the model required by mission 05. Mission 06 adds reclamation to it,
missions 07-10 connect shared managed fixtures to native object reading and
tracing. Missions 15-16 add model and runtime handle semantics.
An object's references are its entire representation at this stage.
