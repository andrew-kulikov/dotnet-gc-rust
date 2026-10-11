use gc_rust::graph::{Graph, GraphError};

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

    let marked = graph.begin_collection().mark(&[root])?;
    println!("Mark: {:?}", marked.trace_result().stats);
    println!("Sweep: {:?}", marked.sweep());
    assert_eq!(graph.objects().count(), 2);
    assert_eq!(graph.references(root), Some([child].as_slice()));
    assert!(graph.references(detached_a).is_none());
    assert!(graph.references(detached_b).is_none());

    // Allocation resumes; dropping every root lets the next collection reclaim all.
    let new_object = graph.add_object();
    assert_ne!(new_object, detached_a);
    assert_ne!(new_object, detached_b);
    println!("Without roots: {:?}", graph.collect(&[])?);
    assert_eq!(graph.objects().count(), 0);
    Ok(())
}
