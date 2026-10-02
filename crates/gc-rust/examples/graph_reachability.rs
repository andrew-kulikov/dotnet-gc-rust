use gc_rust::graph::{Graph, GraphError, ObjectId, trace};

fn main() -> Result<(), GraphError> {
    let mut graph = Graph::new();
    let root = graph.add_object();
    let child = graph.add_object();
    let detached_a = graph.add_object();
    let detached_b = graph.add_object();

    graph.add_edge(root, child)?;
    graph.add_edge(child, root)?;
    graph.add_edge(detached_a, detached_b)?;
    graph.add_edge(detached_b, detached_a)?;

    let names = [
        (root, "root"),
        (child, "child"),
        (detached_a, "detached_a"),
        (detached_b, "detached_b"),
    ];

    println!("Two separate cycles:");
    report(&graph, &[root], &names)?;

    graph.add_edge(child, detached_a)?;

    println!("\nAfter adding child -> detached_a:");
    report(&graph, &[root], &names)?;
    Ok(())
}

fn report(graph: &Graph, roots: &[ObjectId], names: &[(ObjectId, &str)]) -> Result<(), GraphError> {
    let result = trace(graph, roots)?;

    for &(object, name) in names {
        let references = graph.references(object).unwrap();
        let targets: Vec<_> = references
            .iter()
            .map(|target| names.iter().find(|(id, _)| id == target).unwrap().1)
            .collect();
        let status = if result.reachable.contains(&object) {
            "reachable"
        } else {
            "unreachable"
        };
        println!("  {name} -> {targets:?}: {status}");
    }

    println!("Objects still stored: {}", graph.objects().count());
    println!("Stats: {:?}", result.stats);
    Ok(())
}
