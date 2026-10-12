use gc_rust::graph::Graph;
use std::{collections::HashMap, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).ok_or("expected a fixture file")?;
    for line in fs::read_to_string(path)?.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        let [name, edges, roots, expected] = fields.as_slice() else {
            return Err("expected name, edges, roots, and live labels".into());
        };
        let links: Vec<_> = edges
            .split(',')
            .map(|link| link.split_once(':').ok_or("expected label:next"))
            .collect::<Result<_, _>>()?;
        let mut graph = Graph::new();
        let ids: HashMap<_, _> = links
            .iter()
            .map(|(label, _)| (*label, graph.add_object()))
            .collect();
        if ids.len() != links.len() {
            return Err("duplicate node label".into());
        }
        let resolve = |label: &str| ids.get(label).copied().ok_or("unknown node label");
        for &(source, target) in &links {
            if target != "-" {
                graph
                    .add_edge(resolve(source)?, resolve(target)?)
                    .map_err(|error| format!("{error:?}"))?;
            }
        }
        let roots = if *roots == "-" {
            Vec::new()
        } else {
            roots.split(',').map(resolve).collect::<Result<_, _>>()?
        };
        let marked = graph
            .begin_collection()
            .mark(&roots)
            .map_err(|error| format!("{error:?}"))?;
        let mut live: Vec<_> = ids
            .iter()
            .filter(|(_, id)| marked.trace_result().reachable.contains(id))
            .map(|(&label, _)| label)
            .collect();
        live.sort_unstable();
        let live = if live.is_empty() {
            "-".to_owned()
        } else {
            live.join(",")
        };
        if live != *expected {
            return Err(format!("{name}: expected {expected}, observed {live}").into());
        }
        let stats = marked.sweep();
        assert_eq!(stats.allocated_before, links.len());
        assert_eq!(stats.allocated_after, graph.objects().count());
        println!("model {name} {live}");
    }
    Ok(())
}
