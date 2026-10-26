use gc_rust::graph::Graph;
use std::{collections::HashMap, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).ok_or("expected a fixture file")?;
    let selected = std::env::args().nth(2);
    let mut matched = false;
    let fixtures = fs::read_to_string(path)?;
    for line in fixtures.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        let [name, edges, roots, expected] = fields.as_slice() else {
            return Err("expected name, edges, roots, and live labels".into());
        };
        if selected
            .as_deref()
            .is_some_and(|selected| selected != *name)
        {
            continue;
        }
        matched = true;
        check_fixture(name, edges, roots, expected)?;
    }
    if !matched {
        return Err("no matching fixtures".into());
    }
    Ok(())
}

fn check_fixture(
    name: &str,
    edges: &str,
    root_labels: &str,
    expected_live: &str,
) -> Result<(), Box<dyn Error>> {
    let links: Vec<_> = edges
        .split(',')
        .map(|link| link.split_once(':').ok_or("expected label:next"))
        .collect::<Result<_, _>>()?;

    // Allocate every object before resolving links, including forward references.
    let mut graph = Graph::new();
    let object_ids: HashMap<_, _> = links
        .iter()
        .map(|(label, _)| (*label, graph.add_object()))
        .collect();
    if object_ids.len() != links.len() {
        return Err("duplicate node label".into());
    }

    let resolve = |label: &str| object_ids.get(label).copied().ok_or("unknown node label");
    for &(source, target) in &links {
        if target != "-" {
            graph
                .add_edge(resolve(source)?, resolve(target)?)
                .map_err(|error| format!("{error:?}"))?;
        }
    }
    let roots = if root_labels == "-" {
        Vec::new()
    } else {
        root_labels
            .split(',')
            .map(resolve)
            .collect::<Result<_, _>>()?
    };

    let marked = graph
        .begin_collection()
        .mark(&roots)
        .map_err(|error| format!("{error:?}"))?;
    let reachable = &marked.trace_result().reachable;
    let mut live_labels: Vec<_> = object_ids
        .iter()
        .filter(|(_, id)| reachable.contains(id))
        .map(|(&label, _)| label)
        .collect();
    // HashMap iteration is unordered; the oracle and transcript use sorted labels.
    live_labels.sort_unstable();
    let actual_live = if live_labels.is_empty() {
        "-".to_owned()
    } else {
        live_labels.join(",")
    };
    if actual_live != expected_live {
        return Err(format!("{name}: expected {expected_live}, observed {actual_live}").into());
    }

    let stats = marked.sweep();
    assert_eq!(stats.allocated_before, links.len());
    assert_eq!(stats.allocated_after, graph.objects().count());
    println!("model {name} {actual_live}");
    Ok(())
}
