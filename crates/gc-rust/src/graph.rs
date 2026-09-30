use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObjectId(pub u64);

#[derive(Default)]
pub struct Graph {
    objects: HashMap<ObjectId, Vec<ObjectId>>,
    next_id: u64,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_object(&mut self) -> ObjectId {
        let id = ObjectId(self.next_id);
        self.next_id = self.next_id.checked_add(1).expect("object IDs exhausted");
        self.objects.insert(id, Vec::new());
        id
    }

    pub fn add_edge(&mut self, source: ObjectId, target: ObjectId) -> Result<(), GraphError> {
        if !self.objects.contains_key(&source) {
            return Err(GraphError::MissingSource(source));
        }
        if !self.objects.contains_key(&target) {
            return Err(GraphError::MissingTarget { source, target });
        }

        self.objects.get_mut(&source).unwrap().push(target);
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum GraphError {
    MissingRoot(ObjectId),
    MissingSource(ObjectId),
    MissingTarget { source: ObjectId, target: ObjectId },
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct TraceStats {
    pub roots_visited: usize,
    pub objects_discovered: usize,
    pub edges_examined: usize,
    pub max_pending_work: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TraceResult {
    pub reachable: HashSet<ObjectId>,
    pub stats: TraceStats,
}

pub fn trace(graph: &Graph, roots: &[ObjectId]) -> Result<TraceResult, GraphError> {
    for &root in roots {
        if !graph.objects.contains_key(&root) {
            return Err(GraphError::MissingRoot(root));
        }
    }

    for (&source, targets) in &graph.objects {
        for &target in targets {
            if !graph.objects.contains_key(&target) {
                return Err(GraphError::MissingTarget { source, target });
            }
        }
    }

    let mut reachable = HashSet::new();
    let mut pending = Vec::new();
    let mut stats = TraceStats {
        roots_visited: roots.len(),
        ..TraceStats::default()
    };

    for &root in roots {
        if reachable.insert(root) {
            pending.push(root);
        }
    }
    stats.max_pending_work = pending.len();

    while let Some(object) = pending.pop() {
        let targets = &graph.objects[&object];
        stats.edges_examined += targets.len();

        for &target in targets {
            if reachable.insert(target) {
                pending.push(target);
            }
        }
        stats.max_pending_work = stats.max_pending_work.max(pending.len());
    }

    stats.objects_discovered = reachable.len();
    Ok(TraceResult { reachable, stats })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn id(value: u64) -> ObjectId {
        ObjectId(value)
    }

    fn build_graph(entries: &[(u64, &[u64])]) -> Graph {
        let mut graph = Graph::new();
        for &(source, _) in entries {
            assert_eq!(graph.add_object(), id(source));
        }
        for &(source, targets) in entries {
            for &target in targets {
                graph.add_edge(id(source), id(target)).unwrap();
            }
        }
        graph
    }

    #[test]
    fn adds_objects_and_edges() {
        let mut graph = Graph::new();
        let first = graph.add_object();
        let second = graph.add_object();

        assert_ne!(first, second);
        graph.add_edge(first, second).unwrap();
        graph.add_edge(second, second).unwrap();
        assert_eq!(
            trace(&graph, &[first]).unwrap().reachable,
            HashSet::from([first, second])
        );
        assert_eq!(
            graph.add_edge(id(99), first),
            Err(GraphError::MissingSource(id(99)))
        );
        assert_eq!(
            graph.add_edge(first, id(99)),
            Err(GraphError::MissingTarget {
                source: first,
                target: id(99),
            })
        );
    }

    #[test]
    fn traces_cycles_and_disconnected_objects() {
        let graph = build_graph(&[(0, &[1]), (1, &[2]), (2, &[0]), (3, &[3])]);
        let result = trace(&graph, &[id(0)]).unwrap();

        assert_eq!(result.reachable, HashSet::from([id(0), id(1), id(2)]));
        assert_eq!(result.stats.roots_visited, 1);
        assert_eq!(result.stats.objects_discovered, 3);
        assert_eq!(result.stats.edges_examined, 3);
        assert_eq!(result.stats.max_pending_work, 1);
    }

    #[test]
    fn counts_duplicate_roots_and_edges_without_scanning_twice() {
        let graph = build_graph(&[(0, &[0, 1, 1]), (1, &[])]);
        let result = trace(&graph, &[id(0), id(0)]).unwrap();

        assert_eq!(result.reachable, HashSet::from([id(0), id(1)]));
        assert_eq!(result.stats.roots_visited, 2);
        assert_eq!(result.stats.objects_discovered, 2);
        assert_eq!(result.stats.edges_examined, 3);
        assert_eq!(result.stats.max_pending_work, 1);
    }

    #[test]
    fn records_peak_pending_work() {
        let graph = build_graph(&[(0, &[1, 2]), (1, &[]), (2, &[])]);
        let result = trace(&graph, &[id(0)]).unwrap();

        assert_eq!(result.stats.max_pending_work, 2);
    }

    #[test]
    fn handles_empty_graph_and_empty_roots() {
        assert_eq!(
            trace(&Graph::new(), &[]).unwrap().stats,
            TraceStats::default()
        );

        let graph = build_graph(&[(0, &[])]);
        assert!(trace(&graph, &[]).unwrap().reachable.is_empty());
    }

    #[test]
    fn rejects_missing_roots_and_edges_even_when_unreachable() {
        let graph = build_graph(&[(0, &[])]);
        assert_eq!(trace(&graph, &[id(1)]), Err(GraphError::MissingRoot(id(1))));

        let mut graph = build_graph(&[(0, &[]), (1, &[])]);
        graph.objects.get_mut(&id(1)).unwrap().push(id(2));
        assert_eq!(
            trace(&graph, &[id(0)]),
            Err(GraphError::MissingTarget {
                source: id(1),
                target: id(2),
            })
        );
    }

    #[test]
    fn traces_a_deep_chain_without_recursion() {
        const LENGTH: u64 = 100_000;
        let mut graph = Graph::new();
        for _ in 0..LENGTH {
            graph.add_object();
        }
        for index in 0..LENGTH - 1 {
            graph.add_edge(id(index), id(index + 1)).unwrap();
        }

        let result = trace(&graph, &[id(0)]).unwrap();
        assert_eq!(result.reachable.len(), LENGTH as usize);
        assert_eq!(result.stats.edges_examined, LENGTH as usize - 1);
        assert_eq!(result.stats.max_pending_work, 1);
    }

    fn bfs(graph: &Graph, roots: &[ObjectId]) -> HashSet<ObjectId> {
        let mut queue: VecDeque<_> = roots.iter().copied().collect();
        let mut visited = HashSet::new();

        while let Some(object) = queue.pop_front() {
            if visited.insert(object) {
                queue.extend(graph.objects[&object].iter().copied());
            }
        }

        visited
    }

    #[test]
    fn agrees_with_bfs_on_generated_graphs() {
        let mut state = 1_u64;
        let mut next = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            state >> 32
        };

        for size in 0..32_u64 {
            for _ in 0..20 {
                let mut graph = Graph::new();
                for _ in 0..size {
                    graph.add_object();
                }
                for source in 0..size {
                    for _ in 0..next() % 5 {
                        graph.add_edge(id(source), id(next() % size)).unwrap();
                    }
                }
                let roots: Vec<_> = (0..(next() % 5))
                    .filter(|_| size > 0)
                    .map(|_| id(next() % size))
                    .collect();

                assert_eq!(
                    trace(&graph, &roots).unwrap().reachable,
                    bfs(&graph, &roots)
                );
            }
        }
    }
}
